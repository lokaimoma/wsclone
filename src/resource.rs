use chrono::Utc;
use phf::phf_map;
use reqwest::header::HeaderMap;
use reqwest::{header, Client};
use std::io::SeekFrom;

use lazy_static::lazy_static;
use scraper::{Html, Selector};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::{
    fs::{read_to_string, OpenOptions},
    time::Instant,
};
use tracing::{event, instrument, Level};
use url::Url;

use crate::errors::WscError;
use crate::link::get_full_link;
use crate::session::Session;

const DEFAULT_INDEX_FILE_NAME: &str = "index.html";
const DEFAULT_SUB_PAGES_DIRECTORY_NAME: &str = "sub_pages";
const DEFAULT_RESOURCES_DIRECTORY_NAME: &str = "static";
lazy_static! {
    static ref CSS_TAG_SELECTOR: Selector =
        Selector::parse(r#"link[href][rel="stylesheet"]"#).unwrap();
    static ref JS_TAG_SELECTOR: Selector = Selector::parse("script[src]").unwrap();
    static ref ANCHOR_TAG_SELECTOR: Selector = Selector::parse(
        r##"a[href]:not([download]):not([href="javascript:void(0)"]):not([href~="#"])"##
    )
    .unwrap();
}

#[derive(Debug, PartialEq)]
pub enum ResourceType {
    Page,
    StaticFile,
}

#[derive(Debug)]
pub enum DownloadState {
    Queued,
    Downloading,
    Paused,
    Completed,
}

#[derive(Debug)]
pub struct Resource {
    pub link: Url,
    pub destination: PathBuf,
    pub type_: ResourceType,
    pub content_length: u64,
    pub bytes_written: u64,
    pub state: DownloadState,
    pub can_resume: bool,
}

impl Resource {
    #[instrument]
    pub async fn new(link: Url, session: &Session, client: Arc<Client>) -> Result<Self, WscError> {
        event!(Level::DEBUG, "[Querying resource info] => {}", link);
        let response = match client.head(link.to_string()).send().await {
            Ok(r) => {
                if r.status().is_server_error() || r.status().is_client_error() {
                    event!(
                        Level::ERROR,
                        "Failed to fetch resource info from {}, status code : {}",
                        link,
                        r.status()
                    );
                    return Err(WscError::ErrorFetchingResourceInfo(format!(
                        "Failed to fetch resource info from {}, status code : {}",
                        link,
                        r.status()
                    )));
                }
                r
            }
            Err(e) => {
                event!(
                    Level::ERROR,
                    "Failed to fetch resource info from {}",
                    link.to_string()
                );
                event!(Level::ERROR, "{}", e);
                return Err(WscError::ErrorFetchingResourceInfo(e.to_string()));
            }
        };

        let can_resume = response.headers().contains_key(header::ACCEPT_RANGES);
        let content_length: u64 = match response.headers().get(header::CONTENT_LENGTH) {
            Some(value) => value.to_str().unwrap().parse::<u64>().unwrap_or(0),
            None => 0,
        };
        let mut destination = PathBuf::from(&session.destination_directory);
        let type_: ResourceType;

        if session.index_url == link.to_string() {
            type_ = ResourceType::Page;
            destination.push(DEFAULT_INDEX_FILE_NAME)
        } else {
            let file_name = get_file_name(link.as_ref(), response.headers());
            if file_name.contains(".html")
                || file_name.contains(".xhtml")
                || file_name.contains(".htm")
            {
                type_ = ResourceType::Page;
                destination.push(DEFAULT_SUB_PAGES_DIRECTORY_NAME);
            } else {
                type_ = ResourceType::StaticFile;
                destination.push(DEFAULT_RESOURCES_DIRECTORY_NAME);
            }
            destination.push(file_name)
        };
        event!(
            Level::DEBUG,
            "Location for resource = {}",
            destination.to_string_lossy()
        );

        Ok(Self {
            bytes_written: 0,
            state: DownloadState::Queued,
            link,
            type_,
            destination,
            can_resume,
            content_length,
        })
    }

    #[instrument]
    /// Function that downloads a given resource. The on_progress_update callback
    /// takes in a u64 value as attribute which equals the number of bytes written.
    pub async fn download(
        &mut self,
        client: Arc<Client>,
        progress_update_interval: Duration,
        on_progress_update: fn(u64) -> (),
    ) -> Result<(), WscError> {
        let mut dest_file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.destination.as_path())
            .await
        {
            Err(e) => {
                event!(
                    Level::ERROR,
                    "Failed to open the file {:?}",
                    &self.destination
                );
                event!(Level::ERROR, "{} | {}", e, e.kind());
                return Err(WscError::FailedToOpenResourceFile(
                    self.destination.to_string_lossy().to_string(),
                    e.to_string(),
                ));
            }
            Ok(f) => f,
        };

        let file_size = dest_file.metadata().await.unwrap().len();

        if file_size >= self.content_length && self.content_length != 0 {
            self.state = DownloadState::Completed;
            (on_progress_update)(self.content_length);
        }

        let mut request_build = client.get(self.link.to_string());

        if self.can_resume {
            request_build = request_build.header(
                header::RANGE,
                format!(
                    "bytes={start}-{end}",
                    start = file_size,
                    end = self.content_length
                ),
            );
        } else {
            dest_file.seek(SeekFrom::Start(0)).await.unwrap();
        }

        let mut response = match request_build.send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    event!(
                        Level::ERROR,
                        "Got status code {} whiles fetching {}",
                        response.status(),
                        self.link
                    );
                    return Err(WscError::ErrorDownloadingResource(
                        "Wrong status code returned".into(),
                    ));
                }
                response
            }
            Err(e) => {
                event!(
                    Level::ERROR,
                    "Failed to fetch resource from {}",
                    self.link.to_string()
                );
                event!(Level::ERROR, "{}", e);
                return Err(WscError::ErrorDownloadingResource(e.to_string()));
            }
        };
        event!(Level::DEBUG, "Downloading {}", self.link);
        self.state = DownloadState::Downloading;
        let mut bytes_written = 0;
        let mut last_progress_time = Instant::now() - progress_update_interval;
        while let Some(chunk) = match response.chunk().await {
            Ok(b) => b,
            Err(e) => {
                event!(
                    Level::ERROR,
                    "Failed to download full resource from {}",
                    self.link
                );
                event!(Level::ERROR, "{}", e);
                return Err(WscError::ErrorDownloadingResource(e.to_string()));
            }
        } {
            if let Err(e) = dest_file.write_all(&chunk).await {
                event!(
                    Level::ERROR,
                    "Failed to write bytes to file {}",
                    self.destination.to_string_lossy()
                );
                event!(Level::ERROR, "{}", e);
                on_progress_update(bytes_written);
                return Err(WscError::ErrorWritingToFile(
                    self.destination.to_string_lossy().to_string(),
                    e.to_string(),
                ));
            } else {
                bytes_written += chunk.len() as u64;
                if Instant::now().duration_since(last_progress_time) > progress_update_interval {
                    on_progress_update(bytes_written);
                    last_progress_time = Instant::now();
                }
            };
        }

        self.state = DownloadState::Completed;
        event!(Level::DEBUG, "Download Completed for {}", self.link);
        Ok(())
    }

    #[instrument]
    pub async fn get_resource_links(&self) -> Result<Vec<String>, WscError> {
        let mut result: Vec<String> = Vec::new();

        if self.type_ == ResourceType::Page {
            let html = match read_to_string(&self.destination).await {
                Ok(html) => html,
                Err(e) => {
                    event!(
                        Level::ERROR,
                        "Failed to read html file: {}",
                        self.destination.to_string_lossy().to_string()
                    );
                    event!(Level::ERROR, "{}", e);
                    return Err(WscError::ErrorReadingHtmlFile(
                        self.destination.to_string_lossy().to_string(),
                        e.to_string(),
                    ));
                }
            };
            let html_document = Html::parse_document(&html);
            let css_elements = html_document.select(&CSS_TAG_SELECTOR);
            let script_elements = html_document.select(&JS_TAG_SELECTOR);
            let anchor_elements = html_document.select(&ANCHOR_TAG_SELECTOR);

            css_elements
                .chain(script_elements)
                .chain(anchor_elements)
                .into_iter()
                .for_each(|element| {
                    if let Some(href) = element.value().attr("href") {
                        if let Some(link) = get_full_link(href, &self.link) {
                            result.push(link);
                        };
                    } else if let Some(src) = element.value().attr("src") {
                        if let Some(link) = get_full_link(src, &self.link) {
                            result.push(link)
                        }
                    }
                })
        }

        Ok(result)
    }
}

#[instrument]
fn get_file_name(mut link: &str, response_header: &HeaderMap) -> String {
    let original_link = link.to_owned();
    event!(Level::DEBUG, "Getting file name for {}", link);
    if link.contains('?') {
        if let Some(idx) = link.rfind('?') {
            link = &link[0..idx];
        }
    }
    if let Some(idx) = link.rfind('/') {
        link = &link[idx + 1..link.len()];
    }
    let extension = match response_header.get(header::CONTENT_TYPE) {
        Some(value) => {
            let mut val = value.to_str().unwrap();
            // Removing charset if present (Eg. text/html; charset=utf-8)
            if val.contains(';') {
                val = &val[0..val.find(';').unwrap()];
            }
            MIME_TYPES
                .get(val)
                .unwrap_or_else(|| {
                    event!(
                        Level::WARN,
                        "Content-Type: |{}| didn't much any of the mime types. For {}",
                        val,
                        &original_link
                    );
                    &""
                })
                .to_owned()
        }
        None => {
            event!(Level::WARN, "No content type found for {}", &original_link);
            ""
        }
    };

    if !link.is_empty() && !extension.is_empty() {
        return if link.contains(extension) {
            link.into()
        } else if extension == ".html" {
            format!(
                "page-{time}{ext}",
                time = Utc::now().timestamp(),
                ext = extension
            )
        } else {
            format!(
                "{link}-{time}{ext}",
                link = link,
                time = Utc::now().timestamp(),
                ext = extension
            )
        };
    } else if !link.is_empty() && link.contains('.') && link.len() > 1 {
        return link.to_string();
    } else if !extension.is_empty() {
        return if extension == ".html" {
            format!(
                "page-{time}{ext}",
                time = Utc::now().timestamp(),
                ext = extension
            )
        } else {
            format!(
                "static-{time}{ext}",
                time = Utc::now().timestamp(),
                ext = extension
            )
        };
    }

    return match response_header.get(header::CONTENT_DISPOSITION) {
        None => {
            event!(
                Level::WARN,
                "Failed to get file name for {}. File name: {}. Using generic name.",
                link,
                &original_link
            );
            format!(
                "file-{time}{ext}",
                time = Utc::now().timestamp(),
                ext = extension
            )
        }
        Some(value) => {
            let mut value = value.to_str().unwrap_or("");
            let mut file_name: String;
            if value.contains("filename*=") {
                value = &value[value.find("filename*=").unwrap()..value.len()];
                file_name = value.replace("filename*=", "");
                if let Some(idx) = file_name.rfind('\'') {
                    if idx + 1 < file_name.len() {
                        file_name = file_name[idx + 1..file_name.len()].to_owned();
                    }
                }
            } else if value.contains("filename=") {
                value = &value[value.find("filename=").unwrap()..value.len()];
                file_name = value.replace("filename=", "");
                file_name = file_name.replacen('\"', "", 2);
            } else {
                event!(
                    Level::WARN,
                    "Failed to get file name for {}. File name: {}. Using generic name.",
                    link,
                    &original_link
                );
                file_name = format!(
                    "file-{time}{ext}",
                    time = Utc::now().timestamp(),
                    ext = extension
                );
            }
            file_name
        }
    };
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

#[cfg(test)]
mod test_get_file_name {
    use super::get_file_name;
    use reqwest::header;

    #[test]
    fn get_file_name_from_url() {
        let link: &str = "http://somelink.com/hello.js?skdjs=324&skdsjk=skfj";
        let file_name: &str = "hello.js";
        let mut header_map = header::HeaderMap::new();
        header_map.insert(
            header::CONTENT_TYPE,
            "text/javascript; charset=utf-8".parse().unwrap(),
        );

        assert_eq!(get_file_name(link, &header_map), file_name);
    }

    #[test]
    fn get_file_name_from_url_no_charset() {
        let link: &str = "http://somelink.com/hello.js?skdjs=324&skdsjk=skfj";
        let file_name: &str = "hello.js";
        let mut header_map = header::HeaderMap::new();
        header_map.insert(header::CONTENT_TYPE, "text/javascript;".parse().unwrap());

        assert_eq!(get_file_name(link, &header_map), file_name);
    }

    #[test]
    fn get_file_name_from_url_no_content_type() {
        let link: &str = "http://somelink.com/hello.js?skdjs=324&skdsjk=skfj";
        let file_name: &str = "hello.js";
        let header_map = header::HeaderMap::new();

        assert_eq!(get_file_name(link, &header_map), file_name);
    }

    #[test]
    fn get_file_name_no_name_in_url() {
        let link: &str = "http://somelink.com/";
        let mut header_map = header::HeaderMap::new();
        header_map.insert(
            header::CONTENT_TYPE,
            "text/javascript; charset=utf-8".parse().unwrap(),
        );

        assert!(get_file_name(link, &header_map).contains("static"));
    }

    #[test]
    fn get_file_name_no_name_in_url_2() {
        let link: &str = "http://somelink.com/";
        let mut header_map = header::HeaderMap::new();
        header_map.insert(
            header::CONTENT_TYPE,
            "text/html; charset=utf-8".parse().unwrap(),
        );

        assert!(get_file_name(link, &header_map).contains("page"));
    }

    #[test]
    fn get_file_name_from_content_disposition() {
        let link: &str = "http://somelink.com/";
        let file_name: &str = "hello.js";
        let mut header_map = header::HeaderMap::new();
        header_map.insert(
            header::CONTENT_DISPOSITION,
            r#"attachment; filename="hello.js""#.parse().unwrap(),
        );

        assert_eq!(get_file_name(link, &header_map), file_name);
    }

    #[test]
    fn get_file_name_from_content_disposition_filename_rfc_5987_style() {
        let link: &str = "http://somelink.com/";
        let file_name: &str = "hello.js";
        let mut header_map = header::HeaderMap::new();
        header_map.insert(
            header::CONTENT_DISPOSITION,
            r#"attachment; filename="hello.js"; filename*=UTF-8'en'hello.js"#
                .parse()
                .unwrap(),
        );

        assert_eq!(get_file_name(link, &header_map), file_name);
    }
}
