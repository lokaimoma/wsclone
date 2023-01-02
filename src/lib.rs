use crate::errors::WscError;
use std::future::Future;
use tracing::instrument;

mod download;
mod errors;
mod link;
mod resource;
mod session;

#[derive(Debug)]
pub struct DownloadRule {
    /// Maximum size for static files to download
    pub max_static_file_size: u64,
    pub download_static_resource_with_unknown_size: bool,
    /// Progress update interval in millisecond
    pub progress_update_interval: u64,
    /// Max levels of pages to download. Default is 0, which means download
    /// only the initial page and it's resources.
    pub max_level: u8,
    pub black_list_urls: Vec<String>,
    pub allowed_file_extensions: Vec<String>,
}

#[derive(Debug)]
pub enum Update {
    MessageUpdate(Message),
    ProgressUpdate(Progress),
}

#[derive(Debug)]
pub struct Message {
    pub session_id: String,
    pub content: String,
    pub resource_name: String,
    pub is_error: bool,
}

#[derive(Debug)]
pub struct Progress {
    pub bytes_written: u64,
    pub file_size: u64,
    pub resource_name: String,
    pub session_id: String,
}

#[instrument]
pub async fn init_download<F>(
    session_id: &str,
    link: &str,
    dest_dir: &str,
    rule: DownloadRule,
    on_update: fn(Update) -> F,
) -> Result<(), WscError>
where
    F: Future<Output = ()>,
{
    Ok(())
}
