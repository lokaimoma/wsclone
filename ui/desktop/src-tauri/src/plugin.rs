use directories::ProjectDirs;
use std::fs;
use tauri::{
    api::process::{Command, CommandEvent},
    plugin::Plugin,
    Runtime,
};

#[cfg(target_family = "unix")]
const DEFAULT_SOCKET_F_NAME: &str = "wsclone.sock";

pub struct WSClonePlugin;

impl<R: Runtime> Plugin<R> for WSClonePlugin {
    fn name(&self) -> &'static str {
        "wsclone"
    }

    fn initialize(
        &mut self,
        _app: &tauri::AppHandle<R>,
        _config: serde_json::Value,
    ) -> tauri::plugin::Result<()> {
        let dirs = ProjectDirs::from("com", "koc", "wsclone")
            .expect("Project directoes for wsclone not found");

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
            data_dir.join("Downloads/").to_string_lossy().to_string(),
        ]);
        if let Ok((mut rx, _child)) = cmd.spawn() {
            println!("Daemon launched successfully");

            tauri::async_runtime::spawn(async move {
                while let Some(ev) = rx.recv().await {
                    if let CommandEvent::Stderr(line) = ev {
                        println!("[DAEMON] {line}");
                    } else if let CommandEvent::Stdout(line) = ev {
                        println!("[DAEMON] {line}");
                    }
                }
            });
        } else {
            eprintln!("Error launching daemon");
        };

        Ok(())
    }
}
