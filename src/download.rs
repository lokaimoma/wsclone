use crate::errors::WscError;
use crate::resource::Resource;
use crate::session::Session;
use reqwest::Client;
use std::sync::Arc;
use futures::TryFutureExt;
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
}

#[instrument]
pub async fn init_session_download(session: &Session, rule: DownloadRule) -> Result<(), WscError> {
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
            |bytes_written| {},
        )
        .await?;
    let res_links = index_page.get_resource_links().and_then(|links| async {
        res_links_to_resources(links, &rule, session, client.clone()).await
    }).await?;

    Ok(())
}

async fn res_links_to_resources(
    res_links: Vec<String>,
    rule: &DownloadRule,
    session: &Session,
    client: Arc<Client>,
) -> Result<Vec<Resource>, WscError> {
    let mut resources: Vec<Resource> = Vec::new();
    for link in res_links {
        let res = Resource::new(Url::parse(&link).unwrap(), session, client.clone()).await?;
        if (res.content_length != 0 && res.content_length < rule.max_static_resource_download_size)
            || (res.content_length == 0 && rule.should_download_resource_with_unknown_size)
        {
            resources.push(res);
        }
    }
    Ok(resources)
}
