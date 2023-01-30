use libwsclone::Update;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use ws_common::command::CloneProp;

pub struct DaemonState {
    pub(crate) queued_links: Vec<CloneProp>,
    pub tx: Sender<Update>,
    pub current_session_id: Option<String>,
    pub current_session_thread: Option<JoinHandle<()>>,
    /// An optional map of file names to their current download status
    pub current_session_updates: Option<HashMap<String, FileStatus>>,
}

pub struct FileStatus {
    pub(crate) bytes_written: u64,
    pub(crate) f_size: u64,
    pub(crate) message: Option<MessageContent>,
}

#[derive(Debug, Clone)]
pub struct MessageContent {
    pub(crate) message: String,
    pub(crate) is_error: bool,
}
