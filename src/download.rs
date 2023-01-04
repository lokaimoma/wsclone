use crate::errors::WscError;
use crate::Update::{MessageUpdate, ProgressUpdate};
use crate::{DownloadRule, Message, Progress, Update};
use chrono::Utc;
use phf::phf_map;
use reqwest::header::HeaderMap;
use reqwest::{header, Client};
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::time::Instant;
use url::Url;

#[derive(Debug)]
pub struct DownloadItem {
    pub link: Url,
    pub destination_dir: PathBuf,
}

#[tracing::instrument]
/// Takes care of downloading a file. The returned optional string is the path to the downloaded file
pub async fn download_file<F>(
    session_id: String,
    mut dld_item: DownloadItem,
    client: &Arc<Client>,
    rule: &DownloadRule,
    on_update: fn(Update) -> F,
    file_name: Option<String>,
) -> Result<Option<String>, WscError>
where
    F: Future + Send + 'static,
{
    if rule.black_list_urls.contains(&dld_item.link.to_string()) {
        return Ok(None);
    }

    if !dld_item.destination_dir.exists() {
        return Err(WscError::DestinationDirectoryDoesNotExist(
            dld_item.destination_dir.to_string_lossy().to_string(),
        ));
    }

    let mut response = match client.get(dld_item.link.to_string()).send().await {
        Err(e) => {
            tracing::error!("Error getting file from {}", dld_item.link.to_string());
            tracing::error!("{}", e);
            return Err(WscError::ErrorDownloadingResource(
                dld_item.link.to_string(),
            ));
        }
        Ok(r) => {
            if !r.status().is_success() {
                tracing::error!("Error status code received : {}", r.status());
                return Err(WscError::ErrorDownloadingResource(
                    dld_item.link.to_string(),
                ));
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
        .create_new(true)
        .open(dld_item.destination_dir.as_path())
        .await
    {
        Err(e) => {
            tracing::error!(
                "Error opening/creating file {}",
                dld_item.destination_dir.to_str().unwrap()
            );
            tracing::error!("{}", e);
            on_update(MessageUpdate(Message {
                session_id,
                resource_name: f_name,
                is_error: true,
                content: "Error opening destination file".into(),
            }))
            .await;
            return Err(WscError::FileOperationError(
                dld_item.destination_dir.to_string_lossy().to_string(),
                format!("{} | {}", e, e.kind()),
            ));
        }
        Ok(f) => f,
    };

    let f_size = match headers.get(header::CONTENT_LENGTH) {
        None => 0u64,
        Some(s) => s.to_str().unwrap().parse::<u64>().unwrap_or(0u64),
    };

    if f_size > rule.max_static_file_size {
        return Ok(None);
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
            on_update(MessageUpdate(Message {
                session_id,
                resource_name: f_name,
                is_error: true,
                content: "Network error".into(),
            }))
            .await;
            tracing::error!("{}", e);
            return Err(WscError::ErrorDownloadingResource(e.to_string()));
        }
        Ok(bytes) => bytes,
    } {
        if let Err(e) = dest_file.write_all(&chunks).await {
            tracing::error!(
                "Error writing to destination file {}",
                dld_item.destination_dir.to_str().unwrap()
            );
            tracing::error!("{} | {}", e, e.kind());
            on_update(MessageUpdate(Message {
                session_id,
                resource_name: f_name,
                is_error: true,
                content: "Error writing to file".into(),
            }))
            .await;
            return Err(WscError::FileOperationError(
                dld_item.destination_dir.to_string_lossy().to_string(),
                format!("{} | {}", e, e.kind()),
            ));
        };
        bytes_written += chunks.len();
        if Instant::now().duration_since(last_update_time) > progress_update_interval {
            (on_update)(ProgressUpdate(Progress {
                bytes_written: bytes_written as u64,
                file_size: f_size,
                resource_name: f_name.to_owned(),
                session_id: session_id.to_owned(),
            }))
            .await;
            last_update_time = Instant::now();
        }
    }
    // destination_dir has been updated previously to point to the destination file
    tracing::debug!(
        "Download completed for {}, file @ {}",
        &dld_item.link,
        dld_item.destination_dir.to_str().unwrap()
    );
    Ok(Some(dld_item.destination_dir.to_string_lossy().to_string()))
}

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
