use crate::cli::DaemonCli;
use crate::state::{DaemonState, FileStatus};
use clap::Parser;
use libwsclone::Update;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tokio::sync::RwLock;
use ws_common::response::MessageContent;

mod cli;
mod error;
mod handlers;
mod state;

const MAX_CHANNEL_BUFFER_SIZE: usize = 100;
const MAX_RECEIVER_SLEEP_SECONDS: u64 = 1;

#[tokio::main]
async fn main() {
    let daemon_cli = DaemonCli::parse();
    if !daemon_cli.clones_dir.is_dir() {
        std::io::stderr()
            .write(b"Clones directory might not exist or you do not have permission to access it")
            .unwrap();
        return;
    }
    let (tx, mut rx) = channel::<Update>(MAX_CHANNEL_BUFFER_SIZE);
    let state = Arc::new(RwLock::new(DaemonState {
        queued_links: Vec::new(),
        tx,
        current_session_id: None,
        current_session_thread: None,
        current_session_updates: None,
        clones_dir: daemon_cli.clones_dir.clone(),
    }));

    let state_clone = state.clone();
    tokio::spawn(async move {
        while let Some(update) = rx.recv().await {
            let mut state = state_clone.write().await;
            if state.current_session_updates.is_some() {
                if state
                    .current_session_updates
                    .as_ref()
                    .unwrap()
                    .contains_key(update.get_resource_name())
                {
                    let f_update = state
                        .current_session_updates
                        .as_mut()
                        .unwrap()
                        .get_mut(update.get_resource_name())
                        .unwrap();

                    *f_update = FileStatus {
                        bytes_written: update.get_bytes_written().unwrap_or(f_update.bytes_written),
                        f_size: update.get_file_size(),
                        message: if update.get_message().is_some() {
                            Some(MessageContent {
                                message: update.get_message().unwrap().to_string(),
                                is_error: update.is_error(),
                            })
                        } else {
                            f_update.message.clone()
                        },
                    }
                } else {
                    state.current_session_updates.as_mut().unwrap().insert(
                        update.get_resource_name().to_string(),
                        FileStatus {
                            bytes_written: update.get_bytes_written().unwrap_or(0),
                            f_size: update.get_file_size(),
                            message: if update.get_message().is_some() {
                                Some(MessageContent {
                                    message: update.get_message().unwrap().to_string(),
                                    is_error: update.is_error(),
                                })
                            } else {
                                None
                            },
                        },
                    );
                }
            }
            drop(state);
            tokio::time::sleep(Duration::from_secs(MAX_RECEIVER_SLEEP_SECONDS)).await;
        }
    });

    daemon_cli.run_server(state).await.unwrap();
}
