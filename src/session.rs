use crate::resource::Resource;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Session {
    pub session_id: String,
    /// A url string to file destination map of all processed pages
    pub processed_pages: HashMap<String, String>,
    /// A url string to file destination map of all processed static resources
    pub processed_static_files: HashMap<String, String>pub ,
}
