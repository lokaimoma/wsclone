use crate::download::{download_file, DownloadItem};
use crate::errors::WscError;
use crate::link::get_static_resource_links;
use crate::session::Session;
use std::future::Future;
use std::path::PathBuf;
use tokio::fs;
use tracing::instrument;
use url::Url;

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
    let mut session = Session {
        session_id: session_id.to_string(),
        processed_static_files: Default::default(),
        processed_pages: Default::default(),
    };
    if let Err(e) = fs::create_dir_all(dest_dir).await {
        tracing::error!("Failed to create destination directory");
        tracing::error!("{}", e);
        return Err(WscError::ErrorCreatingDestinationDirectory(e.to_string()));
    };
    let client = reqwest::Client::new();
    let index_file_path = download_file(
        session_id.to_string(),
        DownloadItem {
            link: Url::parse(link).unwrap(),
            destination_dir: PathBuf::from(dest_dir),
        },
        &client,
        &rule,
        on_update,
        Some("index.html".into()),
    )
    .await?
    .unwrap();
    session
        .processed_pages
        .insert(link.to_string(), index_file_path);

    Ok(())
}
