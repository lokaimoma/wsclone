use std::collections::HashMap;
use url::Url;

#[derive(Debug)]
pub struct LinkInfo {
    /// Relative links from html and css files. This might be an absolute link in some cases
    pub relative_link: String,
    /// File path to the downloaded file
    pub file_path: String,
    /// The element attribute that provided the relative link. E.g href, src, etc
    pub(crate) element_attribute: String,
}

#[derive(Debug)]
pub struct Session {
    pub initial_url: Url,
    pub session_id: String,
    /// A url string to file destination map of all processed pages
    pub processed_pages: HashMap<String, LinkInfo>,
    /// A url string to file destination map of all processed static resources
    pub processed_static_files: HashMap<String, LinkInfo>,
}
