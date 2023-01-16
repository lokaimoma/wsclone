use scraper::{Html, Selector};
use std::collections::HashSet;
use tracing::{event, instrument, Level};
use url::{ParseError, Url};

#[instrument]
/// Get the full link to a sub-page or file, given a page's full url.
fn get_full_link(link: &str, page_url: &Url) -> Option<Url> {
    if link.is_empty() {
        return None;
    }
    match Url::parse(link) {
        Ok(url) => Some(url),
        Err(e)
            if e == ParseError::EmptyHost
                || e == ParseError::RelativeUrlWithoutBase
                || e == ParseError::RelativeUrlWithCannotBeABaseBase =>
        {
            Some(page_url.join(link).unwrap())
        }
        Err(e) => {
            event!(Level::ERROR, "Failed to get full link for {}", link);
            event!(Level::ERROR, "{}", e);
            None
        }
    }
}

/// Gets all valid anchor tag links. Each tuple,
/// has as first element, the link found in the page and the second
/// element is a parsed URL object of that link in relation with the
/// current page's url.
/// E.g (/hello.html, https://www.example.com/hello.html as a Url object, href)  
pub fn get_anchor_links(html_string: &str, page_url: Url) -> HashSet<(String, Url)> {
    let html_document = Html::parse_document(html_string);
    let anchor_tag_selector = Selector::parse(
        r##"
        a[href]:not([download]):not([href="javascript:void(0)"]):not([href*="#"])
        "##,
    )
    .unwrap();
    html_document
        .select(&anchor_tag_selector)
        .filter(|element| {
            let value = element.value().attr("href").unwrap_or("");
            !value.is_empty() && !value.contains('#')
        })
        .map(|element| element.value().attr("href").unwrap())
        .map(|relative_link| {
            (
                relative_link.to_owned(),
                get_full_link(relative_link, &page_url),
            )
        })
        .filter(|(_, link)| link.is_some())
        .map(|(relative_link, full_link)| {
            let f_link = full_link.unwrap();
            tracing::debug!("Full link for {} => {}", relative_link, &f_link);
            (relative_link, f_link)
        })
        .collect::<_>()
}

/// Gets all valid static web resource file links. Each tuple,
/// has as first element, the link found in the page and the second
/// element is a parsed URL object of that link in relation with the
/// current page's url and the element attribute that provided
/// the relative link. E.g: href, src,etc.
/// E.g (/hello.js, https://www.example.com/hello.js as a Url object, src)
pub fn get_static_resource_links(
    html_string: &str,
    page_url: Url,
) -> HashSet<(String, Url, String)> {
    let html_document = Html::parse_document(html_string);
    let css_tag_selector = Selector::parse(r#"link[href][rel="stylesheet"]"#).unwrap();
    let js_tag_selector = Selector::parse("script[src]").unwrap();
    let img_tag_selector =
        Selector::parse(r###"img[src]:not([src^="data"]):not([src^="blob"])"###).unwrap();
    html_document
        .select(&css_tag_selector)
        .chain(html_document.select(&js_tag_selector))
        .chain(html_document.select(&img_tag_selector))
        .map(|element| {
            return if let Some(href) = element.value().attr("href") {
                (href, "href")
            } else if let Some(src) = element.value().attr("src") {
                (src, "src")
            } else {
                ("", "")
            };
        })
        .map(|(relative_link, attrib)| {
            let full_link = get_full_link(relative_link, &page_url);
            (relative_link.to_string(), full_link, attrib)
        })
        .filter(|(_, full_link, _)| full_link.is_some())
        .map(|(relative_link, full_link, attrib)| {
            let f_link = full_link.unwrap();
            tracing::debug!("Full link for {} => {}", relative_link, &f_link);
            (relative_link, f_link, attrib.to_string())
        })
        .collect::<_>()
}
