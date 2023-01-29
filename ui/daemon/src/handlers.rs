use crate::state::DaemonState;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::sync::RwLock;
use ws_common::command::{AbortCloneProp, CloneProp, Command, CommandType};
use ws_common::ipc_helpers;
use ws_common::response;

pub async fn handle_connection<T>(mut stream: T, app_state: Arc<RwLock<DaemonState>>)
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
            let payload = response::Ok("Alive".to_string());
            let payload =
                ipc_helpers::payload_to_bytes(&serde_json::to_string(&payload).unwrap()).unwrap();
            stream.write_all(&payload).unwrap();
        }
        CommandType::QueueClone => {
            let clone_prop: CloneProp = match serde_json::from_str(&cmd.props) {
                Ok(v) => v,
                Err(e) => {
                    send_err(stream, format!("Invalid clone prop payload : {}", e)).await;
                    return;
                }
            };
            let app_state = app_state.write().await;
            app_state.queued_links.push(clone_prop);
            let response = response::Ok("Clone task queued successfully".to_string());
            let response = serde_json::to_string(&response).unwrap();
            let response = ipc_helpers::payload_to_bytes(&response).unwrap();
            stream.write_all(&response).await.unwrap();
        }
        CommandType::AbortClone => handle_abort_clone(&mut stream, app_state, cmd).await,
        _ => send_err(&mut stream, "Command not implemented yet".to_string()).await,
    }
}

async fn handle_abort_clone<T>(
    mut stream: &mut T,
    app_state: Arc<RwLock<DaemonState>>,
    cmd: Command,
) where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    let abort_clone_prop: AbortCloneProp = match serde_json::from_str(&cmd.props) {
        Ok(v) => v,
        Err(e) => {
            send_err(
                stream,
                format!("Error parsing props as an AbortCloneProp : {}", e),
            )
            .await;
            return;
        }
    };

    let app_state = app_state.write().await;
    if app_state.current_session_id.is_some()
        && app_state.current_session_id.unwrap() == abort_clone_prop.0
    {
        if let Some(session_thread) = &app_state.current_session_thread {
            session_thread.abort();
            app_state.current_session_thread = None;
            app_state.current_session_updates = None;
        }
        app_state.current_session_id = None;
    }
    let response = response::Ok("Abort successful".to_string());
    let response = serde_json::to_string(&response).unwrap();
    let response = ipc_helpers::payload_to_bytes(&response).unwrap();
    stream.write_all(&response).await.unwrap();
}

async fn send_err<T>(stream: &mut T, msg: String)
where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    let err = response::Err { msg };
    let bytes = ipc_helpers::payload_to_bytes(&serde_json::to_string(&err).unwrap()).unwrap();
    stream.write_all(&bytes).await.unwrap();
}
