use url::{ParseError, Url};
use tracing::{event, instrument, Level};

#[instrument]
/// Get the full link to a sub-page or file, given a page's full url.
pub fn get_full_link(link: &str, page_url: Url) -> Option<String> {
    match Url::parse(link) {
        Ok(_) => Some(link.into()),
        Err(e) if e == ParseError::EmptyHost || e== ParseError::RelativeUrlWithoutBase ||
            e == ParseError::RelativeUrlWithCannotBeABaseBase =>
            Some(page_url.join(link).unwrap().to_string()),
        Err(e) => {
            event!(Level::ERROR, "Failed to get full link for {}", link);
            event!(Level::ERROR, "{}", e);
            None
        }
    }
}
