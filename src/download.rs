use crate::errors::WscError;
use crate::resource::Resource;
use crate::session::Session;
use reqwest::Client;
use std::sync::Arc;
use tracing::{event, instrument, Level};
use url::Url;

#[instrument]
async fn init_session_download(session: &Session) -> Result<(), WscError> {
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
    let index_page_res = Resource::new(index_url, session, client.clone());
    
    Ok(())
}
