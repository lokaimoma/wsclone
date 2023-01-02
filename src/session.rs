use crate::resource::Resource;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Session {
    pub title: String,
    /// The first page's url.
    pub index_url: String,
    /// The destination for the downloaded files. This path must exist.
    pub destination_directory: String,
    pub resources: HashMap<String, Resource>,
}

pub struct SessionUpdate {
    pub session_url: String,
    pub resource_url: String,
    pub message: Option<String>,
    pub bytes_written: Option<u64>,
    pub total_size: Option<u64>,
}

impl Session {
    async fn relink_resources() {}
}
