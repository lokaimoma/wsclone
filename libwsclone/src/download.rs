use crate::errors::WscError;
use crate::Update::{MessageUpdate, ProgressUpdate};
use crate::{DownloadRule, Message, Progress, Update};
use chrono::Utc;
use phf::phf_map;
use reqwest::header::HeaderMap;
use reqwest::{header, Client};

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::Sender;
use tokio::time::Instant;
use url::Url;

#[derive(Debug)]
pub struct DownloadItem {
    pub link: Url,
    pub destination_dir: PathBuf,
}

/// Takes care of downloading a file. The returned optional string is the path to the downloaded file
#[tracing::instrument]
pub async fn download_file(
    session_id: String,
    mut dld_item: DownloadItem,
    client: &Arc<Client>,
    rule: &DownloadRule,
    update_tx: Sender<Update>,
    file_name: Option<String>,
) -> Result<Option<String>, WscError> {
    let link_str = dld_item.link.to_string();
    for link in rule.black_list_urls.iter() {
        if link_str.contains(link) {
            return Ok(None);
        }
    }
    drop(link_str);

    if !dld_item.destination_dir.exists() {
        tracing::error!("Destination directory does not exist. Aborting...");
        return Err(WscError::DestinationDirectoryDoesNotExist(
            dld_item.destination_dir.to_string_lossy().to_string(),
        ));
    }

    let mut response = match client.get(dld_item.link.to_string()).send().await {
        Err(e) => {
            tracing::error!("Error downloading file from {}", dld_item.link.to_string());
            tracing::error!("{}", e);
            // File name is always passed for first page.
            // If it's the first page, then we want to
            // abort the whole download irrespective of
            // the abort rule. Otherwise follow the rule.
            if !rule.abort_on_download_error || file_name.is_some() {
                return Ok(None);
            }
            return Err(WscError::NetworkError(dld_item.link.to_string()));
        }
        Ok(r) => {
            if !r.status().is_success() {
                tracing::error!(
                    "Error status code received : {} |{}|",
                    r.status(),
                    dld_item.link
                );
                return if rule.abort_on_download_error {
                    Err(WscError::ErrorStatusCode {
                        status_code: r.status().to_string(),
                        url: dld_item.link.to_string(),
                    })
                } else {
                    if (update_tx
                        .send(MessageUpdate(Message {
                            session_id: session_id.clone(),
                            resource_name: dld_item.link.to_string(),
                            is_error: true,
                            content: "Error downloading resource".into(),
                        }))
                        .await)
                        .is_err()
                    {};
                    Ok(None)
                };
            }
            r
        }
    };

    let headers = response.headers();
    let f_name: String;
    if file_name.is_none() {
        let f_ext = get_file_extension(&dld_item, headers);
        tracing::debug!(
            "File extension for {} is {}",
            dld_item.link.to_string(),
            f_ext
        );

        f_name = get_file_name(&dld_item, headers, f_ext);
        tracing::debug!("File name for {} is {}", dld_item.link.to_string(), &f_name);
    } else {
        f_name = file_name.unwrap();
    }

    dld_item.destination_dir.push(&f_name);

    let mut dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .open(dld_item.destination_dir.as_path())
        .await
    {
        Err(e) => {
            tracing::error!(
                "Error opening/creating file {}",
                dld_item.destination_dir.to_str().unwrap()
            );
            tracing::error!("{} | {}", e, e.kind());
            if (update_tx
                .send(MessageUpdate(Message {
                    session_id,
                    resource_name: f_name,
                    is_error: true,
                    content: "Error opening destination file".into(),
                }))
                .await)
                .is_err()
            {};
            return Err(WscError::FileOperationError {
                file_name: dld_item.destination_dir.to_string_lossy().to_string(),
                message: format!("{} | {}", e, e.kind()),
            });
        }
        Ok(f) => f,
    };

    let f_size = match headers.get(header::CONTENT_LENGTH) {
        None => 0u64,
        Some(s) => s.to_str().unwrap().parse::<u64>().unwrap_or(0u64),
    };

    if (f_size > 0 && f_size > rule.max_static_file_size)
        || (f_size == 0 && !rule.download_static_resource_with_unknown_size)
    {
        return Ok(None);
    }

    let dest_f_size = match dest_file.metadata().await {
        Ok(m) => m.len(),
        Err(e) => {
            tracing::debug!(
                "Error getting file size for {} @ {}\nError : {}",
                dld_item.destination_dir.to_string_lossy().to_string(),
                dld_item.link.to_string(),
                e
            );
            0
        }
    };

    if dest_f_size > 0 && f_size != 0 && dest_file.metadata().await.unwrap().len() >= f_size {
        let f_name = dld_item.destination_dir.to_string_lossy().to_string();
        tracing::debug!(
            "File : |{}| from |{}| has already been downloaded.",
            f_name,
            dld_item.link.to_string()
        );
        if (update_tx
            .send(ProgressUpdate(Progress {
                bytes_written: f_size,
                file_size: f_size,
                resource_name: f_name.clone(),
                session_id,
            }))
            .await)
            .is_err()
        {};
        return Ok(Some(f_name));
    }

    let progress_update_interval = Duration::from_millis(rule.progress_update_interval);
    let mut last_update_time = Instant::now() - progress_update_interval;
    let mut bytes_written = 0;

    while let Some(chunks) = match response.chunk().await {
        Err(e) => {
            tracing::error!(
                "Error downloading resource from {}",
                dld_item.link.to_string()
            );
            tracing::error!("{}", e);
            if e.is_connect() {
                if (update_tx
                    .send(MessageUpdate(Message {
                        session_id,
                        resource_name: f_name,
                        is_error: true,
                        content: "Network error".into(),
                    }))
                    .await)
                    .is_err()
                {};
                return Err(WscError::NetworkError(e.to_string()));
            } else if e.is_status() {
                let status = e.status().unwrap();
                if (update_tx
                    .send(MessageUpdate(Message {
                        session_id,
                        resource_name: f_name,
                        is_error: true,
                        content: format!("Error status code : {}", status),
                    }))
                    .await)
                    .is_err()
                {};
                if rule.abort_on_download_error {
                    return Err(WscError::ErrorStatusCode {
                        status_code: status.to_string(),
                        url: dld_item.link.to_string(),
                    });
                }
            }
            return Ok(None);
        }
        Ok(bytes) => bytes,
    } {
        if let Err(e) = dest_file.write_all(&chunks).await {
            tracing::error!(
                "Error writing to destination file {}",
                dld_item.destination_dir.to_str().unwrap()
            );
            tracing::error!("{} | {}", e, e.kind());
            if (update_tx
                .send(MessageUpdate(Message {
                    session_id,
                    resource_name: f_name,
                    is_error: true,
                    content: "Error writing to file".into(),
                }))
                .await)
                .is_err()
            {};
            return Err(WscError::FileOperationError {
                file_name: dld_item.destination_dir.to_string_lossy().to_string(),
                message: format!("{} | {}", e, e.kind()),
            });
        };
        bytes_written += chunks.len();
        if Instant::now().duration_since(last_update_time) > progress_update_interval {
            if let Err(e) = update_tx.try_send(ProgressUpdate(Progress {
                bytes_written: bytes_written as u64,
                file_size: f_size,
                resource_name: f_name.to_owned(),
                session_id: session_id.to_owned(),
            })) {
                if let TrySendError::Closed(_) = e {
                    return Err(WscError::ChannelClosed);
                }
            } else {
                last_update_time = Instant::now();
            };
        }
    }
    // destination_dir has been updated previously to point to the destination file
    tracing::debug!(
        "Download completed for {}, file @ {}",
        &dld_item.link,
        dld_item.destination_dir.to_str().unwrap()
    );
    if (update_tx
        .send(ProgressUpdate(Progress {
            bytes_written: bytes_written as u64,
            file_size: if f_size == 0 {
                bytes_written as u64
            } else {
                f_size
            },
            resource_name: f_name,
            session_id,
        }))
        .await)
        .is_err()
    {};
    Ok(Some(dld_item.destination_dir.to_string_lossy().to_string()))
}

#[tracing::instrument]
fn get_file_name(dld_item: &DownloadItem, headers: &HeaderMap, f_ext: &str) -> String {
    let mut file_name = dld_item.link.to_string();
    if file_name.contains('?') {
        file_name = file_name[0..file_name.find('?').unwrap()].parse().unwrap();
    }
    let slash_idx = file_name.rfind('/').unwrap();

    if slash_idx != file_name.len() - 1 {
        file_name = file_name[slash_idx + 1..file_name.len()].parse().unwrap();
    } else {
        file_name = "".into();
    }
    if file_name.is_empty() {
        file_name = match headers.get(header::CONTENT_DISPOSITION) {
            None => {
                tracing::warn!(
                    "File name can't be determined, using generic name. {}",
                    dld_item.link.to_string()
                );
                format!("file-{time}{ext}", time = Utc::now().time(), ext = f_ext)
            }
            Some(cd) => {
                let mut val: String = cd.to_str().unwrap().parse().unwrap();
                // If content disposition uses RFC_5987 style
                if val.contains("filename*") {
                    val = val[val.rfind('\'').unwrap() + 1..file_name.len()]
                        .parse()
                        .unwrap();
                } else if val.contains("filename=") {
                    val = val[val.rfind('=').unwrap() + 1..val.len()].parse().unwrap();
                    if val.contains('\"') {
                        val = val[1..val.len() - 1].parse().unwrap();
                    }
                } else {
                    tracing::warn!(
                        "File name can't be determined, using generic name. {}",
                        dld_item.link.to_string()
                    );
                    val = format!("file-{time}{ext}", time = Utc::now().time(), ext = f_ext);
                }
                val
            }
        }
    }
    file_name
}

#[tracing::instrument]
fn get_file_extension<'a>(dld_item: &'a DownloadItem, headers: &HeaderMap) -> &'a str {
    match headers.get(header::CONTENT_TYPE) {
        None => {
            tracing::warn!(
                "File extension can not be determined for {}",
                dld_item.link.to_string()
            );
            ""
        }
        Some(ct) => {
            let mut val = ct.to_str().unwrap().to_lowercase();
            // Remove charset if present (E.g text/html; charset=utf-8))
            if val.contains(';') {
                val = val[0..val.find(';').unwrap()].parse().unwrap();
            }
            match MIME_TYPES.get(&val) {
                None => {
                    tracing::warn!(
                        "File extension can not be determined for {}",
                        dld_item.link.to_string()
                    );
                    ""
                }
                Some(ext) => ext,
            }
        }
    }
}

static MIME_TYPES: phf::Map<&'static str, &str> = phf_map! {
    "text/html" => ".html",
    "image/jpeg" => ".jpg",
    "text/javascript" => ".js",
    "application/json" => ".json",
    "audio/mpeg" => ".mp3",
    "video/mp4" => ".mp4",
    "video/mpeg" => ".mpeg",
    "audio/ogg" => ".oga",
    "video/ogg" => ".ogv",
    "font/otf" => ".otf",
    "image/png" => ".png",
    "application/pdf" => ".pdf",
    "application/vnd.ms-powerpoint" => ".ppt",
    "application/xhtml+xml" => ".xhtml",
    "text/css" => ".css",
    "image/gif" => ".gif",
    "application/javascript" => ".js"
};
