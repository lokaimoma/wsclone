use crate::state::DaemonState;
use libwsclone::DownloadRule;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::sync::RwLock;
use ws_common::command::{AbortCloneProp, CloneProp, CloneStatusProp, Command, CommandType};
use ws_common::response;
use ws_common::response::{CloneInfo, CloneStatusResponse, FileUpdate, GetClonesResponse};
use ws_common::{ipc_helpers, Payload};

#[tracing::instrument]
pub async fn handle_connection<T>(mut stream: T, daemon_state: Arc<RwLock<DaemonState>>)
where
    T: AsyncRead + AsyncWrite + Send + Unpin + Debug,
{
    let mut keep_alive = true;
    while keep_alive {
        let cmd = match ipc_helpers::get_payload_content(&mut stream).await {
            Ok(r) => r,
            Err(e) => return send_err(&mut stream, e.to_string()).await,
        };

        let cmd = match Command::from_str(&cmd) {
            Ok(c) => c,
            Err(e) => return send_err(stream, e.to_string()).await,
        };

        keep_alive = cmd.keep_alive;

        match cmd.type_ {
            CommandType::HealthCheck => {
                let msg = Some("Alive".to_string());
                let payload = match (response::Success { msg }).to_bytes() {
                    Ok(v) => v,
                    Err(e) => return send_err(stream, e.to_string()).await,
                };
                stream.write_all(&payload).await.unwrap();
            }
            CommandType::Clone => handle_clone(&mut stream, &daemon_state, cmd).await,
            CommandType::AbortClone => handle_abort_clone(&mut stream, &daemon_state, cmd).await,
            CommandType::CloneStatus => {
                handle_get_clone_status(&mut stream, &daemon_state, &cmd).await
            }
            CommandType::GetClones => handle_get_clones(&mut stream, &daemon_state).await,
        }
    }
}

async fn handle_get_clone_status<T>(
    stream: &mut T,
    daemon_state: &Arc<RwLock<DaemonState>>,
    cmd: &Command,
) where
    T: AsyncWrite + Send + Unpin,
{
    let prop = match CloneStatusProp::from_str(&cmd.props) {
        Ok(v) => v,
        Err(e) => return send_err(stream, e.to_string()).await,
    };
    let mut state = daemon_state.write().await;
    if state.current_session_id.is_some()
        && state.current_session_id.as_ref().unwrap() != &prop.session_id
    {
        return send_err(stream, "Incorrect session id".to_string()).await;
    }
    if state.current_session_updates.is_some() {
        let updates = state.current_session_updates.clone();
        if state.current_session_thread.is_some() {
            state.current_session_updates = Some(HashMap::new());
            tracing::debug!(msg = "Fetching updates. Cloning not completed yet.")
        } else {
            state.current_session_thread = None;
            state.current_session_id = None;
            state.current_session_updates = None;
            tracing::debug!(
                msg = "Cloning completed. Resetting state to default.",
                state = state.to_string()
            )
        }
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
        let response = match (CloneStatusResponse { updates }).to_bytes() {
            Ok(b) => b,
            Err(e) => return send_err(stream, e.to_string()).await,
        };
        stream.write_all(&response).await.unwrap();
    }
}

async fn handle_get_clones<T>(stream: &mut T, daemon_state: &Arc<RwLock<DaemonState>>)
where
    T: AsyncWrite + Send + Unpin,
{
    let dir = daemon_state.read().await.clones_dir.clone();
    // later the results will be cached into the app state
    let clones: Vec<CloneInfo> = match tokio::fs::read_dir(dir).await {
        Err(e) => return send_err(stream, e.to_string()).await,
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
    let response = match (GetClonesResponse { clones }).to_bytes() {
        Ok(v) => v,
        Err(e) => return send_err(stream, e.to_string()).await,
    };
    stream.write_all(&response).await.unwrap();
}

async fn handle_clone<T>(stream: &mut T, daemon_state: &Arc<RwLock<DaemonState>>, cmd: Command)
where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    let mut app_state = daemon_state.write().await;
    if app_state.current_session_thread.is_some() {
        let response = response::Failure {
            msg: "A clone task is already running in the background".to_string(),
        };
        let response = serde_json::to_string(&response).unwrap();
        let response = ipc_helpers::payload_to_bytes(&response).unwrap();
        stream.write_all(&response).await.unwrap();
        return;
    }
    let clone_prop = match CloneProp::from_str(&cmd.props) {
        Ok(v) => v,
        Err(e) => return send_err(stream, e.to_string()).await,
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
        return send_err(
            stream,
            format!(
                "Error creating destination directory {} : {e}",
                dest_dir.to_string_lossy()
            ),
        )
        .await;
    }
    let dest_dir = dest_dir.to_string_lossy().to_string();
    let daemon_state = daemon_state.clone();
    let handle = tokio::spawn(async move {
        let res = libwsclone::init_download(
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
        .await;
        daemon_state.write().await.current_session_thread = None;
        tracing::debug!(
            msg = "Download session terminated",
            error_occured = res.is_err()
        );
    });
    app_state.current_session_thread = Some(handle);
    drop(app_state);
    let response = match (response::Success {
        msg: Some("Clone started successfully".to_string()),
    })
    .to_bytes()
    {
        Ok(v) => v,
        Err(e) => return send_err(stream, e.to_string()).await,
    };
    stream.write_all(&response).await.unwrap();
}

async fn handle_abort_clone<T>(mut stream: T, app_state: &Arc<RwLock<DaemonState>>, cmd: Command)
where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    let abort_clone_prop = match AbortCloneProp::from_str(&cmd.props) {
        Ok(v) => v,
        Err(e) => return send_err(stream, e.to_string()).await,
    };

    let mut app_state = app_state.write().await;
    if app_state.current_session_thread.is_some()
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
    let response = match (response::Success {
        msg: Some("Abort successful".to_string()),
    })
    .to_bytes()
    {
        Ok(v) => v,
        Err(e) => return send_err(stream, e.to_string()).await,
    };
    stream.write_all(&response).await.unwrap();
}

async fn send_err<T>(mut stream: T, msg: String)
where
    T: AsyncWrite + Send + Unpin,
{
    let err = response::Failure { msg }.to_bytes().unwrap();
    stream.write_all(&err).await.unwrap();
}
