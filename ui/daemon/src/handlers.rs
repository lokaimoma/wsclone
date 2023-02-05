use crate::state::DaemonState;
use libwsclone::DownloadRule;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::sync::RwLock;
use ws_common::command::{AbortCloneProp, CloneProp, CloneStatusProp, Command, CommandType};
use ws_common::ipc_helpers;
use ws_common::response;
use ws_common::response::{CloneInfo, CloneStatusResponse, FileUpdate, GetClonesResponse};

#[tracing::instrument]
pub async fn handle_connection<T>(mut stream: T, daemon_state: Arc<RwLock<DaemonState>>)
where
    T: AsyncRead + AsyncWrite + Send + Unpin + Debug,
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
                format!("Error parsing payload to command type : {e}"),
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
                    send_err(&mut stream, format!("Invalid clone prop payload : {e}")).await;
                    return;
                }
            };
            let mut app_state = daemon_state.write().await;
            app_state.queued_links.push(clone_prop);
            drop(app_state);
            let response = response::Ok("Clone task queued successfully".to_string());
            let response = serde_json::to_string(&response).unwrap();
            let response = ipc_helpers::payload_to_bytes(&response).unwrap();
            stream.write_all(&response).await.unwrap();
        }
        CommandType::Clone => handle_clone(stream, daemon_state, cmd).await,
        CommandType::AbortClone => handle_abort_clone(&mut stream, daemon_state, cmd).await,
        CommandType::CloneStatus => {
            let prop: CloneStatusProp = match serde_json::from_str(&cmd.props) {
                Ok(v) => v,
                Err(e) => {
                    send_err(&mut stream, format!("Invalid clone status prop : {e}")).await;
                    return;
                }
            };
            let mut state = daemon_state.write().await;
            if state.current_session_id.is_some()
                && state.current_session_id.as_ref().unwrap() != &prop.session_id
            {
                send_err(&mut stream, "Incorrect session id".to_string()).await;
                return;
            }
            if state.current_session_updates.is_some() {
                let updates = state.current_session_updates.clone();
                state.current_session_updates = Some(HashMap::new());
                drop(state);
                let updates = updates.unwrap();
                let updates = updates
                    .into_iter()
                    .map(|(file_name, status)| FileUpdate {
                        file_name,
                        bytes_written: status.bytes_written,
                        file_size: status.f_size,
                        message: status.message,
                    })
                    .collect();
                let response = CloneStatusResponse { updates };
                let response = match serde_json::to_string(&response) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!(msg = "Error serializing updates", error = e.to_string());
                        send_err(
                            &mut stream,
                            format!("Error serializing updates to string : {e}"),
                        )
                        .await;
                        return;
                    }
                };
                let response = ipc_helpers::payload_to_bytes(&response).unwrap();
                stream.write_all(&response).await.unwrap();
            }
        }
        CommandType::GetClones => {
            let dir = daemon_state.read().await.clones_dir.clone();
            // later the results will be cached into the app state
            let clones: Vec<CloneInfo> = match tokio::fs::read_dir(dir).await {
                Err(e) => {
                    send_err(
                        &mut stream,
                        format!("Error reading clones directory entries : {e}"),
                    )
                    .await;
                    return;
                }
                Ok(mut read_dir) => {
                    // other info will be taken later. Taking only dir name for now
                    let mut res: Vec<String> = Vec::new();
                    while let Ok(Some(entry)) = read_dir.next_entry().await {
                        if let Ok(f_type) = entry.file_type().await {
                            if f_type.is_dir() {
                                res.push(entry.file_name().to_string_lossy().to_string());
                            }
                        }
                    }
                    res.into_iter()
                        .map(|entry| CloneInfo { title: entry })
                        .collect()
                }
            };
            let response = GetClonesResponse { clones };
            let response = match serde_json::to_string(&response) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(
                        msg = "Error serializing clones information",
                        error = e.to_string()
                    );
                    send_err(
                        &mut stream,
                        format!("Error serializing clones information : {e}"),
                    )
                    .await;
                    return;
                }
            };
            let response = ipc_helpers::payload_to_bytes(&response).unwrap();
            stream.write_all(&response).await.unwrap();
        }
    }
}

async fn handle_clone<T>(mut stream: T, daemon_state: Arc<RwLock<DaemonState>>, cmd: Command)
where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    let mut app_state = daemon_state.write().await;
    if app_state.current_session_thread.is_some() {
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
            send_err(stream, format!("Invalid clone prop payload : {e}")).await;
            return;
        }
    };
    app_state.current_session_id = Some(clone_prop.session_id.clone());
    app_state.current_session_updates = Some(HashMap::new());
    let tx = app_state.tx.clone();
    let mut dest_dir = app_state.clones_dir.clone();
    dest_dir.push(&clone_prop.dir_name);
    if let Err(e) = tokio::fs::create_dir_all(dest_dir.as_path()).await {
        tracing::error!(
            msg = "Error creating destination directory for clone",
            error = e.to_string(),
            error_kind = e.kind().to_string(),
            clone_dir_name = clone_prop.dir_name
        );
        send_err(
            &mut stream,
            format!(
                "Error creating destination directory {} : {e}",
                dest_dir.to_string_lossy()
            ),
        )
        .await;
        return;
    }
    let dest_dir = dest_dir.to_string_lossy().to_string();
    let daemon_state = daemon_state.clone();
    let handle = tokio::spawn(async move {
        libwsclone::init_download(
            &clone_prop.session_id,
            &clone_prop.link,
            &dest_dir,
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
        daemon_state.write().await.current_session_thread = None;
    });
    app_state.current_session_thread = Some(handle);
    drop(app_state);
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
                format!("Error parsing props as an AbortCloneProp : {e}"),
            )
            .await;
            return;
        }
    };

    let mut app_state = app_state.write().await;
    if app_state.current_session_id.is_some()
        && app_state.current_session_id.as_ref().unwrap() == &abort_clone_prop.session_id
    {
        if let Some(session_thread) = &app_state.current_session_thread {
            session_thread.abort();
            app_state.current_session_thread = None;
            app_state.current_session_updates = None;
        }
        app_state.current_session_id = None;
    }
    drop(app_state);
    let response = response::Ok("Abort successful".to_string());
    let response = serde_json::to_string(&response).unwrap();
    let response = ipc_helpers::payload_to_bytes(&response).unwrap();
    stream.write_all(&response).await.unwrap();
}

async fn send_err<T>(mut stream: T, msg: String)
where
    T: AsyncWrite + Send + Unpin,
{
    let err = response::Err(msg);
    let bytes = ipc_helpers::payload_to_bytes(&serde_json::to_string(&err).unwrap()).unwrap();
    stream.write_all(&bytes).await.unwrap();
}
