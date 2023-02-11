use libwsclone::Update;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use ws_common::command::CloneProp;
use ws_common::response::MessageContent;

#[derive(Debug)]
pub struct DaemonState {
    pub queued_links: Vec<CloneProp>,
    pub tx: Sender<Update>,
    pub current_session_id: Option<String>,
    pub current_session_thread: Option<JoinHandle<()>>,
    /// An optional map of file names to their current download status
    pub current_session_updates: Option<HashMap<String, FileStatus>>,
    pub clones_dir: PathBuf,
}

impl std::fmt::Display for DaemonState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DaemonState_abridged {{current_session_id_is_some = {},current_session_thread_is_some={},current_session_updates_is_some={}}}",self.current_session_id.is_some(), self.current_session_thread.is_some(), self.current_session_updates.is_some())
    }
}

#[derive(Clone, Debug)]
pub struct FileStatus {
    pub bytes_written: u64,
    pub f_size: Option<u64>,
    pub message: Option<MessageContent>,
}
