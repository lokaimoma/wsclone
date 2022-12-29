use std::collections::{HashSet, HashMap};
use std::path::PathBuf;


#[derive(Debug)]
pub struct Session {
    pub title: String,
    /// The first page's url.
    pub index_url: String,
    pub destination_directory: PathBuf,
    /// A page url to file path map of the pages we've downloaded
    pub processed_pages: HashMap<String, String>,
    /// A resource link to file path map of the resources we've downloaded
    pub processed_resource_links: HashMap<String, String>,
}

impl Session {
    async fn relink_pages(){}
    async fn relink_ressource_files(){}
}