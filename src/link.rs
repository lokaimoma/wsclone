use crate::errors::WscError;
use scraper::{Html, Selector};
use tracing::{event, instrument, Level};
use url::{ParseError, Url};

#[instrument]
/// Get the full link to a sub-page or file, given a page's full url.
fn get_full_link(link: &str, page_url: &Url) -> Option<String> {
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

/// Gets all valid anchor tag links. The returned vector of tuples,
/// has as first element, the link found in the page and the second
/// element is a parsed URL object of that link.
/// E.g (/hello.html, https://www.example.com/hello.html as a Url object)  
pub fn get_anchor_links(html_string: &str, page_url: Url) -> Vec<(String, Url)> {
    let html_document = Html::parse_document(html_string);
    let anchor_tag_selector = Selector::parse(
        r##"a[href]:not([download]):not([href="javascript:void(0)"]):not([href~="#"])"##,
    )
    .unwrap();
    html_document
        .select(&anchor_tag_selector)
        .filter_map(|element| get_full_link(element.value().attr("href").unwrap_or(""), &page_url))
        .map(|link| (link.to_owned(), Url::parse(&link).unwrap()))
        .collect::<Vec<(String, Url)>>()
}

/// Gets all valid anchor css and javascript file links. The returned vector of
/// tuples, has as first element, the link found in the page and the second
/// element is a parsed URL object of that link.
/// E.g (/hello.js, https://www.example.com/hello.js as a Url object)
pub fn get_static_resource_links(html_string: &str, page_url: Url) -> Vec<Url> {
    let html_document = Html::parse_document(html_string);
    let css_tag_selector = Selector::parse(r#"link[href][rel="stylesheet"]"#).unwrap();
    let js_tag_selector = Selector::parse("script[src]").unwrap();
    html_document
        .select(&css_tag_selector)
        .chain(html_document.select(&js_tag_selector))
        .map(|element| {
            return if let Some(href) = element.value().attr("href") {
                href
            } else if let Some(src) = element.value().attr("src") {
                src
            } else {
                ""
            };
        })
        .filter_map(|link| get_full_link(link, &page_url))
        .map(|link| Url::parse(&link).unwrap())
        .collect::<Vec<Url>>()
}
