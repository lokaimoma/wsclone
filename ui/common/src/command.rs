use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandType {
    Clone,
    HealthCheck,
    GetClones,
    AbortClone,
    QueueClone,
    CloneStatus,
}

#[derive(Debug, Deserialize)]
pub struct Command {
    #[serde(rename(deserialize = "type"))]
    pub type_: CommandType,
    pub props: String,
}

#[derive(Debug, Deserialize)]
pub struct CloneProp {
    #[serde(rename(deserialize = "sessionId"))]
    pub session_id: String,
    pub link: String,
    #[serde(rename(deserialize = "dirName"))]
    pub dir_name: String,
    #[serde(rename(deserialize = "maxStaticFileSize"))]
    pub max_static_file_size: u64,
    #[serde(rename(deserialize = "downloadStaticResourceWithUnknownSize"))]
    pub download_static_resource_with_unknown_size: bool,
    /// Progress update interval in millisecond
    #[serde(rename(deserialize = "progressUpdateInterval"))]
    pub progress_update_interval: u64,
    /// Max levels of pages to download. Default is 0, which means download
    /// only the initial page and it's resources.
    #[serde(rename(deserialize = "maxLevel"))]
    pub max_level: u8,
    #[serde(rename(deserialize = "blackListUrls"))]
    pub black_list_urls: Vec<String>,
    /// Abort download if any resource other than the first page encounters an error.
    #[serde(rename(deserialize = "abortOnDownloadError"))]
    pub abort_on_download_error: bool,
}

/// The following structs takes as argument the session id of a clone.
#[derive(Debug, Deserialize)]
pub struct AbortCloneProp(pub String);
#[derive(Debug, Deserialize)]
pub struct CloneStatusProp(pub String);
