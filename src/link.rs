use crate::errors::WscError;
use futures::FutureExt;
use scraper::{Html, Selector};
use tracing::{event, instrument, Level};
use url::{ParseError, Url};

#[instrument]
/// Get the full link to a sub-page or file, given a page's full url.
pub fn get_full_link(link: &str, page_url: &Url) -> Option<String> {
    if link.is_empty() {
        return None;
    }
    match Url::parse(link) {
        Ok(_) => Some(link.into()),
        Err(e)
            if e == ParseError::EmptyHost
                || e == ParseError::RelativeUrlWithoutBase
                || e == ParseError::RelativeUrlWithCannotBeABaseBase =>
        {
            Some(page_url.join(link).unwrap().to_string())
        }
        Err(e) => {
            event!(Level::ERROR, "Failed to get full link for {}", link);
            event!(Level::ERROR, "{}", e);
            None
        }
    }
}

#[tracing::instrument]
pub fn get_anchor_links(html_string: &str, page_url: Url) -> Result<Vec<Url>, WscError> {
    let html_document = Html::parse_document(html_string);
    let anchor_tag_selector = Selector::parse(
        r##"a[href]:not([download]):not([href="javascript:void(0)"]):not([href~="#"])"##,
    )
    .unwrap();
    Ok(html_document
        .select(&anchor_tag_selector)
        .filter_map(|element| get_full_link(element.value().attr("href").unwrap_or(""), &page_url))
        .map(|link| Url::parse(&link).unwrap())
        .collect::<Vec<Url>>())
}
