use crate::errors::WscError;
use crate::resource::Resource;
use crate::session::Session;
use futures::TryFutureExt;
use reqwest::Client;
use std::sync::Arc;

use tokio::time::Duration;
use tracing::{event, instrument, Level};
use url::Url;

#[derive(Debug)]
pub struct DownloadRule {
    /// Maximum size for static files to download
    pub max_static_resource_download_size: u64,
    pub should_download_resource_with_unknown_size: bool,
    /// Progress update interval in millisecond
    pub progress_update_interval: u64,
    /// Max levels of pages to download. Default is 1, which means will download the initial page
    /// and all the links to other pages it finds on the initial page then it stops.
    pub max_level: u8,
}

#[instrument]
pub async fn init_session_download(
    session: &mut Session,
    rule: DownloadRule,
) -> Result<(), WscError> {
    let index_url: Url = match Url::parse(&session.index_url) {
        Ok(u) => u,
        Err(e) => {
            event!(
                Level::ERROR,
                "Error parsing index url: {}",
                &session.index_url
            );
            return Err(WscError::ErrorParsingIndexUrl(e.to_string()));
        }
    };

    let client = Arc::from(Client::new());
    let mut index_page = Resource::new(index_url, session, client.clone()).await?;
    index_page
        .download(
            client.clone(),
            Duration::from_millis(rule.progress_update_interval),
            |_| {},
        )
        .await?;
    let res_links = index_page
        .get_resource_links()
        .and_then(|links| async {
            res_links_to_resources(links, &rule, session, client.clone()).await
        })
        .await?;
    for mut res in res_links {
        if (res
            .download(
                client.clone(),
                Duration::from_millis(rule.progress_update_interval),
                |_| {},
            )
            .await)
            .is_err()
        {};
    }
    Ok(())
}

#[instrument]
async fn res_links_to_resources(
    res_links: Vec<String>,
    rule: &DownloadRule,
    session: &mut Session,
    client: Arc<Client>,
) -> Result<Vec<Resource>, WscError> {
    event!(Level::DEBUG, "Total links received {}", res_links.len());
    let mut resources: Vec<Resource> = Vec::new();
    for link in res_links {
        if link.contains('#') {
            continue;
        }
        let res = match Resource::new(Url::parse(&link).unwrap(), session, client.clone()).await {
            Ok(r) => r,
            Err(_) => continue,
        };
        if (res.content_length != 0 && res.content_length < rule.max_static_resource_download_size)
            || (res.content_length == 0 && rule.should_download_resource_with_unknown_size)
        {
            resources.push(res);
        }
    }
    event!(
        Level::DEBUG,
        "Total links processed for download {}",
        resources.len()
    );
    Ok(resources)
}
