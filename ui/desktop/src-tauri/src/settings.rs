use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub max_static_file_size: usize,
    pub clones_dir: String,
    pub download_static_res_with_unknown_size: bool,
    pub abort_on_download_error: bool
}


impl Settings {
    
}


