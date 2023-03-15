use tauri::{Runtime, plugin::Plugin};

pub struct WSClonePlugin;


impl<R: Runtime> Plugin<R> for WSClonePlugin {
    fn name(&self) -> &'static str {
        "wsclone"
    }

    fn initialize(&mut self, app: &tauri::AppHandle<R>, config: serde_json::Value) -> tauri::plugin::Result<()> {
        // get path to daemon and load
       todo!()
    }
}
