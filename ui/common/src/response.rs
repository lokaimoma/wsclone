use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Failure {
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Success {
    pub msg: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MessageContent {
    pub message: String,
    #[serde(rename = "isError")]
    pub is_error: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUpdate {
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "bytesWritten")]
    pub bytes_written: u64,
    #[serde(rename = "fileSize")]
    pub file_size: Option<u64>,
    pub message: Option<MessageContent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloneStatusResponse {
    pub completed: bool,
    pub updates: Vec<FileUpdate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetClonesResponse {
    pub clones: Vec<CloneInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloneInfo {
    pub title: String,
    //pub size: u64,
    //pub file_count: u64,
    //pub pages: u64,
    //pub errors_occurred: bool,
}
