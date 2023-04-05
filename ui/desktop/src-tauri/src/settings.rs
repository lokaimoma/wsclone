use directories::ProjectDirs;
use serde::Serialize;
use std::fs;

const SETTINGS_F_NAME:&str = "wsclone.config.json";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub max_static_file_size: usize,
    pub clones_dir: String,
    pub download_static_res_with_unknown_size: bool,
    pub abort_on_download_error: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let dirs = ProjectDirs::from("com", "koc", "wsclone")
            .expect("Project directories for wsclone not found");
        let config_dir = dirs.config_dir().to_path_buf();
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).expect("Failed to create config directory for wsclone");
        }
        let settings_file = config_dir.join(SETTINGS_F_NAME);
        
        todo!()
    }
}
