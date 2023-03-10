use crate::download::{download_file, DownloadItem};
use crate::errors::WscError;
use crate::link::{get_anchor_links, get_static_resource_links};
use crate::session::{LinkInfo, Session};
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::{fs, spawn};
use tracing::instrument;
use url::Url;

mod download;
mod errors;
mod link;
mod session;

#[derive(Debug, Clone)]
pub struct DownloadRule {
    /// Maximum size for static files to download
    pub max_static_file_size: u64,
    pub download_static_resource_with_unknown_size: bool,
    /// Progress update interval in millisecond
    pub progress_update_interval: u64,
    /// Max levels of pages to download. Minimum is 0, which means download
    /// only the initial page and it's resources.
    pub max_level: u8,
    pub black_list_urls: Vec<String>,
    /// Abort download if any resource other than the first page encounters an error.
    pub abort_on_download_error: bool,
}

#[derive(Debug)]
pub enum Update {
    MessageUpdate(Message),
    ProgressUpdate(Progress),
}

impl Update {
    pub fn get_resource_name(&self) -> &str {
        match self {
            Update::MessageUpdate(msg) => &msg.resource_name,
            Update::ProgressUpdate(prog) => &prog.resource_name,
        }
    }

    pub fn get_file_size(&self) -> Option<u64> {
        match self {
            Update::ProgressUpdate(prog) => Some(prog.file_size),
            _ => None,
        }
    }

    pub fn get_bytes_written(&self) -> Option<u64> {
        match self {
            Update::ProgressUpdate(prog) => Some(prog.bytes_written),
            _ => None,
        }
    }

    pub fn get_message(&self) -> Option<&str> {
        match self {
            Update::MessageUpdate(msg) => Some(msg.content.as_str()),
            _ => None,
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            Update::MessageUpdate(msg) => msg.is_error,
            _ => true,
        }
    }
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

#[derive(Debug, Clone)]
struct DownloadProp {
    session_id: String,
    dest_dir: String,
    rule: DownloadRule,
    file_name: Option<String>,
    session: Arc<RwLock<Session>>,
    client: Arc<Client>,
}

#[instrument]
pub async fn init_download(
    session_id: &str,
    link: &str,
    dest_dir: &str,
    mut rule: DownloadRule,
    update_tx: Sender<Update>,
) -> Result<(), WscError> {
    let initial_url = if let Ok(u) = Url::parse(link) {
        u
    } else {
        return Err(WscError::InvalidUrl(link.to_string()));
    };

    if let Err(e) = fs::create_dir_all(dest_dir).await {
        tracing::error!("Failed to create destination directory\nError : {}", e);
        return Err(WscError::ErrorCreatingDestinationDirectory(e.to_string()));
    };

    let client = Arc::new(Client::builder()
        .user_agent(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36"
        ).build().unwrap());

    let session_lock = Arc::new(RwLock::new(Session {
        initial_url,
        session_id: session_id.to_string(),
        processed_static_files: Default::default(),
        processed_pages: Default::default(),
    }));

    let mut a_href_links: Vec<(String, Url)> = match download_page_with_static_resources(
        update_tx.clone(),
        rule.max_level > 0,
        link,
        &Url::parse(link).unwrap(),
        DownloadProp {
            session_id: session_id.to_string(),
            dest_dir: dest_dir.to_string(),
            rule: DownloadRule {
                black_list_urls: rule.black_list_urls.clone(),
                max_level: rule.max_level,
                abort_on_download_error: true,
                max_static_file_size: rule.max_static_file_size,
                download_static_resource_with_unknown_size: true,
                progress_update_interval: rule.progress_update_interval,
            },
            file_name: Some("index.html".to_string()),
            session: session_lock.clone(),
            client: client.clone(),
        },
    )
    .await
    {
        Err(e) => return Err(e),
        Ok(some_links) => {
            if some_links.is_some() {
                some_links.unwrap()
            } else {
                Vec::new()
            }
        }
    };

    while rule.max_level > 0 {
        let more_pages = rule.max_level - 1 > 0;
        let mut new_pages: Vec<(String, Url)> = Vec::new();
        for (raw_link, pg_url) in a_href_links.iter() {
            if let Some(mut pages) = download_page_with_static_resources(
                update_tx.clone(),
                more_pages,
                raw_link,
                pg_url,
                DownloadProp {
                    rule: rule.clone(),
                    file_name: None,
                    session_id: session_id.to_string(),
                    dest_dir: dest_dir.to_string(),
                    client: client.clone(),
                    session: session_lock.clone(),
                },
            )
            .await?
            {
                new_pages.append(&mut pages);
            };
        }
        a_href_links.clear();
        a_href_links.append(&mut new_pages);
        rule.max_level -= 1;
    }

    let mut raw_links: Vec<String> = Vec::new();
    let mut res_f_loc: Vec<String> = Vec::new();

    let session = session_lock.read().await;

    for (_, link_info) in session
        .processed_static_files
        .iter()
        .chain(session.processed_pages.iter())
    {
        raw_links.push(format!(
            r#"{attribute}="{relative_link}""#,
            attribute = link_info.element_attribute,
            relative_link = link_info.relative_link
        ));
        res_f_loc.push(format!(
            r#"{attribute}="{file_path}""#,
            attribute = link_info.element_attribute,
            file_path = link_info.file_path
        ));
    }

    for (_, link_info) in session_lock.read().await.processed_pages.iter() {
        link_page_to_static_resources(&link_info.file_path, &raw_links, &res_f_loc).await?;
    }
    tracing::debug!("Session completed");
    Ok(())
}

#[tracing::instrument]
async fn download_page_with_static_resources(
    update_tx: Sender<Update>,
    more_pages: bool,
    relative_link: &str,
    full_link: &Url,
    prop: DownloadProp,
) -> Result<Option<Vec<(String, Url)>>, WscError> {
    // Check if current page and initial page belong to the same host.
    if let Some(initial_page_host) = prop.session.read().await.initial_url.host() {
        if let Some(host) = full_link.host() {
            if initial_page_host.to_string() != host.to_string() {
                tracing::debug!("Skipping {}", full_link.to_string());
                return Ok(None);
            }
        }
    }

    let mut pages: Option<Vec<(String, Url)>> = None;

    match download_file(
        prop.session_id.to_string(),
        DownloadItem {
            link: full_link.to_owned(),
            destination_dir: PathBuf::from(&prop.dest_dir),
        },
        &prop.client,
        &prop.rule,
        update_tx.clone(),
        prop.file_name.clone(),
    )
    .await
    {
        Err(e) => return Err(e),
        Ok(res) => {
            if res.is_some() {
                let page_f_path = res.unwrap();
                prop.session.write().await.processed_pages.insert(
                    full_link.to_string(),
                    LinkInfo {
                        relative_link: relative_link.to_string(),
                        file_path: page_f_path.to_string(),
                        element_attribute: "href".to_string(),
                    },
                );
                let static_res_links: Vec<(String, Url, String)> =
                    match fs::read_to_string(&page_f_path).await {
                        // This might mostly be a UTF-8 error and rarely a read operation error
                        // We only abort if it's the initial page.
                        Err(e) => {
                            tracing::error!("Error reading file {}\nError : {}", page_f_path, e);
                            update_tx
                                .send(Update::MessageUpdate(Message {
                                    session_id: prop.session_id.clone(),
                                    content: format!("Error reading file for resource links. {e}"),
                                    resource_name: "".to_string(),
                                    is_error: false,
                                }))
                                .await
                                .unwrap();
                            // This is valid for only the initial page
                            if prop.file_name.is_some() {
                                return Err(WscError::FileOperationError {
                                    file_name: page_f_path,
                                    message: format!("{} | {}", e, e.kind()),
                                });
                            }
                            return Ok(None);
                        }
                        Ok(html) => {
                            let dest_dir = &prop.dest_dir;
                            let session = prop.session.read().await;
                            if more_pages {
                                // If a page has already been downloaded and all links replaced, the
                                // links to the static resources will point to their local files. Which
                                // we don't want to try downloading (404). Hence the filtering.
                                pages = Some(
                                    get_anchor_links(&html, full_link.to_owned())
                                        .into_iter()
                                        .filter(|(relative_link, url)| {
                                            !relative_link.contains(dest_dir)
                                                && !session
                                                    .processed_pages
                                                    .contains_key(&url.to_string())
                                        })
                                        .collect(),
                                );
                            }
                            get_static_resource_links(&html, full_link.to_owned())
                                .into_iter()
                                .filter(|(relative_link, url, _)| {
                                    !relative_link.contains(dest_dir)
                                        && !session
                                            .processed_static_files
                                            .contains_key(&url.to_string())
                                })
                                .collect()
                        }
                    };
                let mut dld_tasks: Vec<JoinHandle<Option<WscError>>> = Vec::new();
                for (raw_link, parsed_link, attrib) in static_res_links {
                    let task = download_static_resource(
                        update_tx.clone(),
                        raw_link,
                        parsed_link,
                        attrib,
                        prop.clone(),
                    );
                    dld_tasks.push(task);
                }

                for task in dld_tasks {
                    match task.await {
                        Ok(opt_error) => {
                            if opt_error.is_some() {
                                let err = opt_error.unwrap();
                                if matches!(err, WscError::DestinationDirectoryDoesNotExist(_))
                                    || matches!(err, WscError::NetworkError(_))
                                    || (matches!(
                                        err,
                                        WscError::ErrorStatusCode {
                                            status_code: _,
                                            url: _
                                        }
                                    ) && prop.rule.abort_on_download_error)
                                {
                                    return Err(err);
                                } else {
                                    tracing::warn!("An error occurred but not network error. Continuing download...\nError : {}", err);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Download thread panicked\nError : {}", e);
                            return Err(WscError::UnknownError(e.to_string()));
                        }
                    }
                }
            }
        }
    }
    Ok(pages)
}

fn download_static_resource(
    update_tx: Sender<Update>,
    relative_link: String,
    full_link: Url,
    attribute: String,
    mut prop: DownloadProp,
) -> JoinHandle<Option<WscError>> {
    prop.file_name = None;
    spawn(async move {
        return match download_file(
            prop.session_id,
            DownloadItem {
                link: full_link.clone(),
                destination_dir: PathBuf::from(prop.dest_dir),
            },
            &prop.client,
            &prop.rule,
            update_tx,
            None,
        )
        .await
        {
            Ok(opt_f_path) => {
                if let Some(f_path) = opt_f_path {
                    prop.session.write().await.processed_static_files.insert(
                        full_link.to_string(),
                        LinkInfo {
                            relative_link,
                            file_path: f_path,
                            element_attribute: attribute,
                        },
                    );
                }
                return None;
            }
            Err(e) => Some(e),
        };
    })
}

#[tracing::instrument]
async fn link_page_to_static_resources(
    page_file_path: &str,
    raw_links: &[String],
    res_f_loc: &[String],
) -> Result<(), WscError> {
    let html_string = match fs::read_to_string(&page_file_path).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(
                "Error reading file {}\nError : {} | {}",
                page_file_path,
                e,
                e.kind()
            );
            return Err(WscError::FileOperationError {
                file_name: page_file_path.into(),
                message: format!("{} | {}", e, e.kind()),
            });
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
        .create(true)
        .write(true)
        .truncate(true)
        .open(&page_file_path)
        .await
    {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(
                "Error opening file : {}\nError : {} | {}",
                page_file_path,
                e,
                e.kind()
            );
            return Err(WscError::FileOperationError {
                file_name: page_file_path.into(),
                message: format!("{} | {}", e, e.kind()),
            });
        }
    };

    if let Err(e) = file.write_all(&final_html_bytes).await {
        tracing::error!("Error writing to file : {}\nError : {}", page_file_path, e);
        return Err(WscError::FileOperationError {
            file_name: page_file_path.into(),
            message: format!("{} | {}", e, e.kind()),
        });
    }

    Ok(())
}
