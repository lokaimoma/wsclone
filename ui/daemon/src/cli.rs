use crate::error::{Error, Result};
use crate::handlers::handle_connection;
use crate::state::DaemonState;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(not(target_family = "unix"))]
use tokio::net::TcpListener;
#[cfg(target_family = "unix")]
use tokio::net::UnixListener;
use tokio::sync::RwLock;

#[derive(Parser, Debug)]
#[command(about = "WsClone daemon", version)]
pub struct DaemonCli {
    #[cfg(not(target_family = "unix"))]
    pub port: u8,
    #[cfg(not(target_family = "unix"))]
    #[arg(default_value_t = String::from("localhost"))]
    pub host: String,
    #[cfg(target_family = "unix")]
    pub socket_file_path: PathBuf,
    pub clones_dir: PathBuf,
}

impl DaemonCli {
    #[cfg(target_family = "unix")]
    #[tracing::instrument]
    pub async fn run_server(&self, state: Arc<RwLock<DaemonState>>) -> Result<()> {
        if let Err(e) = std::fs::remove_file(self.socket_file_path.as_path()) {
            if !e.to_string().contains("No such file or directory") {
                return Err(Error::IOError(format!(
                    "{e} : {}\nFile : {}",
                    e.kind(),
                    self.socket_file_path.to_string_lossy()
                )));
            }
        };
        let listener = self.get_unix_socket_listener()?;
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(s) => s,
                Err(e) => {
                    return Err(Error::SocketError(format!("{} | {}", e, e.kind())));
                }
            };
            let state = state.clone();
            tokio::spawn(async move {
                handle_connection(stream, state).await;
            });
        }
    }

    #[cfg(not(target_family = "unix"))]
    #[tracing::instrument]
    pub async fn run_server(&self, state: Arc<RwLock<DaemonState>>) -> Result<()> {
        let listener = self.get_tcp_socket_listener().await?;
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(s) => s,
                Err(e) => {
                    return Err(Error::SocketError(format!("{} | {}", e, e.kind())));
                }
            };
            let state = state.clone();
            tokio::spawn(async move {
                handle_connection(stream, state).await;
            });
        }
    }

    #[cfg(target_family = "unix")]
    fn get_unix_socket_listener(&self) -> Result<UnixListener> {
        let listener = match UnixListener::bind(self.socket_file_path.as_path()) {
            Ok(l) => l,
            Err(e) => {
                tracing::error!(
                    msg = "Error creating UNIX socket",
                    error = e.to_string(),
                    error_kind = e.kind().to_string()
                );
                return Err(Error::SocketError(format!("{} | {}", e, e.kind())));
            }
        };
        Ok(listener)
    }

    #[cfg(not(target_family = "unix"))]
    async fn get_tcp_socket_listener(&self) -> Result<TcpListener> {
        let listener = match TcpListener::bind(format!("{}:{}", self.host, self.port)).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!(
                    "Error creating TCP socket",
                    error = e
                    error_kind = e.kind()
                );
                return Err(Error::SocketError(format!("{} | {}", e, e.kind())));
            }
        };
        Ok(listener)
    }
}
