use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandType {
    Clone,
    CloneStatus,
    DaemonStatus,
    GetClones,
    AbortClone,
    QueueClone,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command<T> {
    #[serde(rename(serialize = "type"))]
    pub type_: CommandType,
    pub props: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloneProp {
    pub session_id: String,
    pub link: String,
    pub dest_dir: String,
    pub max_static_file_size: u64,
    pub download_static_resource_with_unknown_size: bool,
    /// Progress update interval in millisecond
    pub progress_update_interval: u64,
    /// Max levels of pages to download. Default is 0, which means download
    /// only the initial page and it's resources.
    pub max_level: u8,
    pub black_list_urls: Vec<String>,
    /// Abort download if any resource other than the first page encounters an error.
    pub abort_on_download_error: bool,
}

/// The following structs takes as argument the session id of a clone.
#[derive(Debug, Serialize, Deserialize)]
pub struct CloneStatusProp(String);
#[derive(Debug, Serialize, Deserialize)]
pub struct AbortClone(String);
