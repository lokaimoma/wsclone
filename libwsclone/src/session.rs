use std::collections::HashMap;
use url::Url;

#[derive(Debug)]
pub struct Session {
    pub initial_url: Url,
    pub session_id: String,
    /// A url string to file destination map of all processed pages
    pub processed_pages: HashMap<String, String>,
    /// A url string to file destination map of all processed static resources
    pub processed_static_files: HashMap<String, String>,
}
