use directories::ProjectDirs;
use std::fs;
#[cfg(not(target_family = "unix"))]
use std::net::TcpStream;
#[cfg(target_family = "unix")]
use std::os::unix::net::UnixStream;
use tauri::api::process::{Command, CommandChild};

#[cfg(target_family = "unix")]
const DEFAULT_SOCKET_F_NAME: &str = "wsclone.sock";
#[cfg(not(target_family = "unix"))]
const DEFAULT_PORT: &str = "5070";

pub struct AppState {
    pub daemon: CommandChild,
    #[cfg(target_family = "unix")]
    pub daemon_client: Option<UnixStream>,
    #[cfg(not(target_family = "unix"))]
    pub daemon_client: Option<TcpStream>,
}

impl AppState {
    #[cfg(target_family = "unix")]
    pub fn new() -> Result<Self, String> {
        let dirs = ProjectDirs::from("com", "koc", "wsclone")
            .expect("Project directories for wsclone not found");

        let data_dir = dirs.data_dir().to_path_buf();
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).expect("Failed to create data directory for wsclone");
        }

        let cmd = Command::new_sidecar("daemon").unwrap();
        let cmd = cmd.args([
            data_dir
                .join(DEFAULT_SOCKET_F_NAME)
                .to_string_lossy()
                .to_string(),
            data_dir.join("Downloads").to_string_lossy().to_string(),
        ]);

        let daemon;

        match cmd.spawn() {
            Err(e) => return Err(format!("[ERROR] Failed to run daemon : {e}")),
            Ok((_, child)) => {
                daemon = child;
                println!("[INFO] Daemon created successfully");
            }
        }

        Ok(Self {
            daemon,
            daemon_client: None,
        })
    }

    #[cfg(not(target_family = "unix"))]
    pub fn new1() -> Result<Self, String> {
        let dirs = ProjectDirs::from("com", "koc", "wsclone")
            .expect("Project directoes for wsclone not found");

        let data_dir = dirs.data_dir().to_path_buf();
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).expect("Failed to create data directory for wsclone");
        }

        let cmd = Command::new_sidecar("daemon").unwrap();
        let cmd = cmd.args([
            DEFAULT_PORT.to_string(),
            data_dir.join("Downloads").to_string_lossy().to_string(),
        ]);

        let daemon;

        match cmd.spawn() {
            Err(e) => return Err(format!("[ERROR] Failed to run daemon : {e}")),
            Ok((_, child)) => {
                daemon = child;
            }
        }

        Ok(Self { daemon, daemon_client: None})
    }
}
