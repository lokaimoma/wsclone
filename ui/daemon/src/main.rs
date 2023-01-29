use crate::cli::DaemonCli;
use crate::state::{DaemonState, FileStatus, MessageContent};
use clap::Parser;
use libwsclone::Update;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::RwLock;

mod cli;
mod error;
mod handlers;
mod state;

const MAX_CHANNEL_BUFFER_SIZE: usize = 100;
const MAX_RECEIVER_SLEEP_SECONDS: u64 = 1;

#[tokio::main]
async fn main() {
    let daemon_cli = DaemonCli::parse();
    let (tx, mut rx) = channel::<Update>(MAX_CHANNEL_BUFFER_SIZE);
    let state = Arc::new(RwLock::new(DaemonState {
        queued_links: Vec::new(),
        tx,
        current_session_id: None,
        current_session_thread: None,
        current_session_updates: None,
    }));
    tokio::spawn(async move {
        let state = state.clone();
        while let Some(update) = rx.recv().await {
            let state = state.write().await;
            if state.current_session_updates.is_some() {
                if state
                    .current_session_updates
                    .unwrap()
                    .contains_key(update.get_resource_name())
                {
                    let f_update = state
                        .current_session_updates
                        .unwrap()
                        .get_mut(update.get_resource_name())
                        .unwrap();

                    *f_update = FileStatus {
                        bytes_written: update.get_bytes_written().unwrap_or(f_update.bytes_written),
                        f_size: update.get_file_size().unwrap_or(f_update.f_size),
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
                    state.current_session_updates.unwrap().insert(
                        update.get_resource_name().to_string(),
                        FileStatus {
                            bytes_written: update.get_bytes_written().unwrap_or(0),
                            f_size: update.get_file_size().unwrap_or(0),
                            message: if update.get_message().is_some() {
                                Some(MessageContent {
                                    message: update.get_message().unwrap().to_string(),
                                    is_error: update.is_error(),
                                })
                            } else {
                                None
                            },
                        },
                    )
                }
            }
            drop(state);
            tokio::time::sleep(Duration::from_secs(MAX_RECEIVER_SLEEP_SECONDS)).await;
        }
    });
    daemon_cli.run_server(state).await.unwrap();
}
