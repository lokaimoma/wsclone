use crate::download::{download_file, DownloadItem};
use crate::errors::WscError;
use crate::link::get_static_resource_links;
use crate::session::Session;
use reqwest::Client;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::{fs, spawn};
use tracing::instrument;
use url::Url;

mod download;
mod errors;
mod link;
mod resource;
mod session;

#[derive(Debug, Clone)]
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
    F: Future + Send + 'static,
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
    let client = Arc::new(Client::new());
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
        .insert(link.to_string(), index_file_path.to_owned());

    let static_res_links = match fs::read_to_string(&index_file_path).await {
        Err(e) => {
            tracing::error!("Error reading file {}", index_file_path);
            tracing::error!("{}", e);
            return Err(WscError::FileOperationError(
                index_file_path,
                format!("{} | {}", e, e.kind()),
            ));
        }
        Ok(html) => get_static_resource_links(&html, Url::parse(link).unwrap()),
    };

    let session_lock = Arc::new(RwLock::new(session));
    let mut dld_tasks: Vec<JoinHandle<Option<WscError>>> = Vec::new();

    for (raw_link, parsed_link) in static_res_links {
        if session_lock
            .read()
            .await
            .processed_static_files
            .contains_key(&raw_link)
        {
            continue;
        }
        let sess_id = session_id.to_string();
        let dl_rule = rule.clone();
        let dl_dir = dest_dir.to_string();
        let cli = client.clone();
        let session = session_lock.clone();
        let task = download_static_resource(
            on_update,
            raw_link,
            parsed_link,
            sess_id,
            &dl_rule,
            dl_dir,
            &cli,
            session,
        );
        dld_tasks.push(task);
    }

    for task in dld_tasks {
        match task.await {
            Ok(opt_error) => {
                if opt_error.is_some() {
                    let err = opt_error.unwrap();
                    if matches!(err, WscError::DestinationDirectoryDoesNotExist(_))
                        || matches!(err, WscError::ErrorDownloadingResource(_))
                    {
                        return Err(err);
                    } else {
                        tracing::warn!(
                            "An error occurred but not network error. Continuing download..."
                        );
                        tracing::error!("{:?}", err);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Download thread panicked");
                tracing::error!("{}", e);
                return Err(WscError::UnknownError(e.to_string()));
            }
        }
    }

    // process_other pages here before moving below

    let mut raw_links: Vec<String> = Vec::new();
    let mut res_f_loc: Vec<String> = Vec::new();

    for (r_link, r_loc) in session_lock
        .read()
        .await
        .processed_static_files
        .iter()
        .chain(session_lock.read().await.processed_pages.iter())
    {
        raw_links.push(r_link.to_string());
        res_f_loc.push(r_loc.to_string());
    }

    for (_, p_loc) in session_lock.read().await.processed_pages.iter() {
        link_page_to_static_resources(p_loc, &raw_links, &res_f_loc).await?;
    }

    Ok(())
}

async fn link_page_to_static_resources(
    page_file_path: &str,
    raw_links: &Vec<String>,
    res_f_loc: &Vec<String>,
) -> Result<(), WscError> {
    let html_string = match fs::read_to_string(&page_file_path).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Error reading file {}", page_file_path);
            tracing::error!("{} | {}", e, e.kind());
            return Err(WscError::FileOperationError(
                page_file_path.into(),
                format!("{} | {}", e, e.kind()),
            ));
        }
    };

    let html_string_bytes = html_string.as_bytes();

    let mut final_html_bytes = Vec::new();

    let ac = aho_corasick::AhoCorasick::new(raw_links);

    if let Err(e) = ac.stream_replace_all(html_string_bytes, &mut final_html_bytes, res_f_loc) {
        // Shouldn't happen though
        tracing::error!("AHOCORASICK ERROR : {}", e);
        return Err(WscError::UnknownError(e.to_string()));
    };

    let mut file = match fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&page_file_path)
        .await
    {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("Error opening file : {}", page_file_path);
            tracing::error!("{}", e);
            return Err(WscError::FileOperationError(
                page_file_path.into(),
                format!("{} | {}", e, e.kind()),
            ));
        }
    };

    if let Err(e) = file.write_all(&final_html_bytes).await {
        tracing::error!("Error writing to file : {}", page_file_path);
        tracing::error!("{}", e);
        return Err(WscError::FileOperationError(
            page_file_path.into(),
            format!("{} | {}", e, e.kind()),
        ));
    }

    Ok(())
}

fn download_static_resource<F>(
    on_update: fn(Update) -> F,
    raw_link: String,
    parsed_link: Url,
    sess_id: String,
    dl_rule: &DownloadRule,
    dl_dir: String,
    cli: &Arc<Client>,
    session: Arc<RwLock<Session>>,
) -> JoinHandle<Option<WscError>>
where
    F: Future + Send + 'static,
{
    spawn(async move {
        return match download_file(
            sess_id,
            DownloadItem {
                link: parsed_link,
                destination_dir: PathBuf::from(dl_dir),
            },
            &cli,
            &dl_rule,
            on_update,
            None,
        )
        .await
        {
            Ok(opt_f_path) => {
                if opt_f_path.is_some() {
                    session
                        .write()
                        .await
                        .processed_static_files
                        .insert(raw_link, opt_f_path.unwrap());
                }
                return None;
            }
            Err(e) => Some(e),
        };
    })
}
