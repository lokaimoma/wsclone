use crate::error::{Error, Result};
use clap::Parser;
use std::path::PathBuf;
use tokio::net::{TcpListener, UnixListener};

#[derive(Parser, Debug)]
#[command(about = "WsClone daemon", version)]
pub struct DaemonCli {
    port: u8,
    #[arg(default_value_t = String::from("localhost"))]
    host: String,
    #[cfg(target_family = "unix")]
    socket_file_path: PathBuf,
}

impl DaemonCli {
    #[cfg(target_family = "unix")]
    pub async fn get_unix_socket_listener(&self) -> Result<UnixListener> {
        let listener = match UnixListener::bind(self.socket_file_path.as_path()) {
            Ok(l) => l,
            Err(e) => {
                return Err(Error::SocketError(format!("{} | {}", e, e.kind())));
            }
        };
        Ok(listener)
    }

    pub async fn get_tcp_socket_listener(&self) -> Result<TcpListener> {
        let listener = match TcpListener::bind(format!("{}:{}", self.host, self.port)).await {
            Ok(l) => l,
            Err(e) => {
                return Err(Error::SocketError(format!("{} | {}", e, e.kind())));
            }
        };
        Ok(listener)
    }
}
