use crate::command_handlers::handle_connection;
use crate::error::{Error, Result};
use clap::Parser;
use std::path::PathBuf;
#[cfg(not(target_family = "unix"))]
use tokio::net::TcpListener;
#[cfg(target_family = "unix")]
use tokio::net::UnixListener;

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
}

impl DaemonCli {
    #[cfg(target_family = "unix")]
    pub async fn run_server(&self) -> Result<()> {
        let listener = self.get_unix_socket_listener().unwrap();
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(s) => s,
                Err(e) => {
                    return Err(Error::SocketError(format!("{} | {}", e, e.kind())));
                }
            };

            tokio::spawn(async move {
                handle_connection(stream).await;
            });
        }
    }

    #[cfg(not(target_family = "unix"))]
    pub async fn run_server(&self) -> Result<()> {
        let listener = self.get_tcp_socket_listener().await?;
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(s) => s,
                Err(e) => {
                    return Err(Error::SocketError(format!("{} | {}", e, e.kind())));
                }
            };

            tokio::spawn(async move {
                handle_connection(stream).await;
            });
        }
    }

    #[cfg(target_family = "unix")]
    fn get_unix_socket_listener(&self) -> Result<UnixListener> {
        let listener = match UnixListener::bind(self.socket_file_path.as_path()) {
            Ok(l) => l,
            Err(e) => {
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
                return Err(Error::SocketError(format!("{} | {}", e, e.kind())));
            }
        };
        Ok(listener)
    }
}
