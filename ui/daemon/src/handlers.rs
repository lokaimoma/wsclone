use crate::state::DaemonState;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::sync::RwLock;
use ws_common::command::{Command, CommandType};
use ws_common::ipc_helpers;
use ws_common::response;
use ws_common::response::HealthCheck;

pub async fn handle_connection<T>(mut stream: T, arc: Arc<RwLock<DaemonState>>)
where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    let cmd = match ipc_helpers::get_payload_content(&mut stream).await {
        Ok(r) => r,
        Err(e) => {
            send_err(&mut stream, e.to_string()).await;
            return;
        }
    };

    let cmd: Command = match serde_json::from_str(&cmd) {
        Ok(r) => r,
        Err(e) => {
            send_err(
                &mut stream,
                format!("Error parsing payload to command type : {}", e),
            )
            .await;
            return;
        }
    };

    match cmd.type_ {
        CommandType::HealthCheck => {
            let payload = HealthCheck("Alive".to_string());
            let payload =
                ipc_helpers::payload_to_bytes(&serde_json::to_string(&payload).unwrap()).unwrap();
            stream.write_all(&payload).unwrap();
            return;
        }
        CommandType::AbortClone => {
            todo!()
        }
        CommandType::Clone => {
            todo!()
        }
        CommandType::CloneStatus => {
            todo!()
        }
        CommandType::GetClones => {
            todo!()
        }
        CommandType::QueueClone => {
            todo!()
        }
    }

    send_err(&mut stream, "Command not implemented yet".to_string()).await;
}

async fn send_err<T>(stream: &mut T, msg: String)
where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    let err = response::Err { msg };
    let bytes = ipc_helpers::payload_to_bytes(&serde_json::to_string(&err).unwrap()).unwrap();
    stream.write_all(&bytes).await.unwrap();
}
