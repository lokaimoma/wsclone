use crate::state::DaemonState;
use libwsclone::DownloadRule;
use std::collections::HashMap;
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
            stream.write_all(&payload).await.unwrap();
        }
        CommandType::QueueClone => {
            let clone_prop: CloneProp = match serde_json::from_str(&cmd.props) {
                Ok(v) => v,
                Err(e) => {
                    send_err(&mut stream, format!("Invalid clone prop payload : {}", e)).await;
                    return;
                }
            };
            let mut app_state = app_state.write().await;
            app_state.queued_links.push(clone_prop);
            let response = response::Ok("Clone task queued successfully".to_string());
            let response = serde_json::to_string(&response).unwrap();
            let response = ipc_helpers::payload_to_bytes(&response).unwrap();
            stream.write_all(&response).await.unwrap();
        }
        CommandType::Clone => handle_clone(stream, app_state, cmd).await,
        CommandType::AbortClone => handle_abort_clone(&mut stream, app_state, cmd).await,
        _ => send_err(&mut stream, "Command not implemented yet".to_string()).await,
    }
}

async fn handle_clone<T>(mut stream: T, app_state: Arc<RwLock<DaemonState>>, cmd: Command)
where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    let mut app_state = app_state.write().await;
    if app_state.current_session_id.is_some() {
        let response =
            response::Err("A clone task is already running in the background".to_string());
        let response = serde_json::to_string(&response).unwrap();
        let response = ipc_helpers::payload_to_bytes(&response).unwrap();
        stream.write_all(&response).await.unwrap();
        return;
    }
    let clone_prop: CloneProp = match serde_json::from_str(&cmd.props) {
        Ok(v) => v,
        Err(e) => {
            send_err(stream, format!("Invalid clone prop payload : {}", e)).await;
            return;
        }
    };
    app_state.current_session_id = Some(clone_prop.session_id.clone());
    app_state.current_session_updates = Some(HashMap::new());
    let tx = app_state.tx.clone();
    let handle = tokio::spawn(async move {
        libwsclone::init_download(
            &clone_prop.session_id,
            &clone_prop.link,
            &clone_prop.dest_dir,
            DownloadRule {
                max_static_file_size: clone_prop.max_static_file_size,
                download_static_resource_with_unknown_size: clone_prop
                    .download_static_resource_with_unknown_size,
                progress_update_interval: clone_prop.progress_update_interval,
                max_level: clone_prop.max_level,
                black_list_urls: clone_prop.black_list_urls,
                abort_on_download_error: clone_prop.abort_on_download_error,
            },
            tx,
        )
        .await
        .unwrap();
    });
    app_state.current_session_thread = Some(handle);
    let response = response::Ok("Clone started successfully".to_string());
    let response = serde_json::to_string(&response).unwrap();
    let response = ipc_helpers::payload_to_bytes(&response).unwrap();
    stream.write_all(&response).await.unwrap();
}

async fn handle_abort_clone<T>(mut stream: T, app_state: Arc<RwLock<DaemonState>>, cmd: Command)
where
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

    let mut app_state = app_state.write().await;
    if app_state.current_session_id.is_some()
        && app_state.current_session_id.as_ref().unwrap() == &abort_clone_prop.0
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

async fn send_err<T>(mut stream: T, msg: String)
where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    let err = response::Err(msg);
    let bytes = ipc_helpers::payload_to_bytes(&serde_json::to_string(&err).unwrap()).unwrap();
    stream.write_all(&bytes).await.unwrap();
}
