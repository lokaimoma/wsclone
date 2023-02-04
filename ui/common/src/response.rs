use serde::Serialize;

#[derive(Serialize)]
pub struct Err(pub String);

#[derive(Serialize)]
pub struct Ok(pub String);

#[derive(Clone, Serialize)]
pub struct MessageContent {
    pub message: String,
    #[serde(rename = "isError")]
    pub is_error: bool,
}

#[derive(Serialize)]
pub struct FileUpdate {
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "bytesWritten")]
    pub bytes_written: u64,
    #[serde(rename = "fileSize")]
    pub file_size: Option<u64>,
    pub message: Option<MessageContent>,
}

#[derive(Serialize)]
pub struct CloneStatusResponse {
    pub updates: Vec<FileUpdate>,
}

#[derive(Serialize)]
pub struct GetClonesResponse {
    pub clones: Vec<CloneInfo>,
}

#[derive(Serialize)]
pub struct CloneInfo {
    pub title: String,
    //pub size: u64,
    //pub file_count: u64,
    //pub pages: u64,
    //pub errors_occurred: bool,
}
