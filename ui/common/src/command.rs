use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandType {
    Clone,
    HealthCheck,
    GetClones,
    AbortClone,
    CloneStatus,
}

fn default_prop() -> String {
    "".into()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Command {
    #[serde(rename(deserialize = "type"))]
    pub type_: CommandType,
    #[serde(default = "default_prop")]
    pub props: String,
    #[serde(default, rename(deserialize = "keepAlive"))]
    pub keep_alive: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloneProp {
    pub session_id: String,
    pub link: String,
    pub dir_name: String,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct AbortCloneProp {
    #[serde(rename(deserialize = "sessionId"))]
    pub session_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CloneStatusProp {
    #[serde(rename(deserialize = "sessionId"))]
    pub session_id: String,
}
