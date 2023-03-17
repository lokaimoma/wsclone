use tauri::{
    api::process::{Command, CommandEvent},
    plugin::Plugin,
    Runtime,
};

pub struct WSClonePlugin;

impl<R: Runtime> Plugin<R> for WSClonePlugin {
    fn name(&self) -> &'static str {
        "wsclone"
    }

    fn initialize(
        &mut self,
        app: &tauri::AppHandle<R>,
        config: serde_json::Value,
    ) -> tauri::plugin::Result<()> {
        println!("Plugin initialization started....");
        let cmd = Command::new_sidecar("daemon").unwrap();
        let cmd = cmd.args(["./wsclone.sock", "./downloads/"]);
        if let Ok((mut rx, child)) = cmd.spawn() {
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
