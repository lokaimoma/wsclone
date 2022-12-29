use std::path::PathBuf;
use reqwest::{Client, header};
use tracing::{event, instrument, Level};
use url::Url;
use phf::phf_map;
use reqwest::header::HeaderMap;
use chrono::Utc;


use crate::errors::WscError;
use crate::session::Session;

const DEFAULT_INDEX_FILE_NAME: &str = "index.html";
const DEFAULT_SUB_PAGES_DIRECTORY_NAME: &str = "sub_pages";
const DEFAULT_RESOURCES_DIRECTORY_NAME: &str = "static";

pub enum ResourceType {
    Page,
    StaticFile
}

pub enum DownloadState {
    Queued,
    Downloading,
    Paused
}

pub struct Resource {
    pub link: String,
    pub destination: PathBuf,
    pub type_: ResourceType,
    pub content_length: u64,
    pub bytes_written: u64,
    pub state: DownloadState,
    pub can_resume: bool,
}


impl Resource {
    #[instrument]
    pub async fn new(link: Url, session: &Session, client: &Client) -> Result<Self, WscError> {
        let response = match client.head(link.to_string()).send().await {
            Ok(r) => r,
            Err(e) => {
                event!(Level::ERROR, "Failed to fetch resource at {}", link.to_string());
                event!(Level::ERROR, "{}", e);
                return Err(WscError::ErrorDownloadingResource(e.to_string()))
            }
        };

        let can_resume = response.headers().contains_key(header::ACCEPT_RANGES);
        let content_length: u64 = match response.headers().get(header::CONTENT_LENGTH) {
            Some(value) => value.to_str().unwrap().parse::<u64>().unwrap_or(0),
            None => 0
        };
        let mut destination = PathBuf::from(&session.destination_directory);
        let type_: ResourceType;

        if session.index_url == link.to_string() {
            type_ = ResourceType::Page;
            destination.push(DEFAULT_INDEX_FILE_NAME)
        }else {
            let file_name = get_file_name(link.as_ref(), response.headers());
            if file_name.contains(".html") || file_name.contains(".xhtml") || file_name.contains(".htm") {
                type_ = ResourceType::Page;
                destination.push(DEFAULT_SUB_PAGES_DIRECTORY_NAME);
            }else {
                type_ = ResourceType::StaticFile;
                destination.push(DEFAULT_RESOURCES_DIRECTORY_NAME);
            }
            destination.push(file_name)
        };

        Ok(Self {
            link: link.to_string(),
            bytes_written: 0,
            state: DownloadState::Queued,
            type_,
            destination,
            can_resume,
            content_length
        })
    }
}

fn get_file_name(mut link: &str, response_header: &HeaderMap) -> String {
    if link.contains('?') {
        if let Some(idx) = link.rfind('?') {
            link = &link[0..idx];
        }
    }
    if let Some(idx) = link.rfind('/') {
        link = &link[idx..link.len()];
    }
    let extension = match response_header.get(header::CONTENT_TYPE) {
        Some(value) => MIME_TYPES.get(value.to_str().unwrap()).unwrap_or(&"").to_owned(),
        None => ""
    };

    if !link.is_empty() && !extension.is_empty() {
        return if link.contains(extension) {
            link.into()
        } else if extension == ".html" {
            format!("page-{time}{ext}", time = Utc::now().timestamp(), ext = extension)
        } else {
            format!("{link}-{time}{ext}", link = link, time = Utc::now().timestamp(), ext = extension)
        }
    }

    return match response_header.get(header::CONTENT_DISPOSITION) {
        None => format!("file-{time}{ext}", time=Utc::now().timestamp(), ext=extension),
        Some(value) => {
            let mut value = value.to_str().unwrap_or("");
            let mut file_name: String;
            if value.contains("filename*=") {
                value = &value[value.find("filename*=").unwrap()..value.len()];
                file_name = value.replace("filename*=", "");
                if let Some(idx) = file_name.rfind('\'') {
                    if idx+1 < file_name.len() {
                        file_name = file_name[idx + 1..file_name.len()].to_owned();
                    }
                }
            }else if value.contains("filename=") {
                value = &value[value.find("filename=").unwrap()..value.len()];
                file_name = value.replace("filename=", "");
                file_name = file_name.replacen('\"', "", 2);
            }else {
                file_name = format!("file-{time}{ext}", time=Utc::now().timestamp(), ext=extension);
            }
            file_name
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

