use tauri::{plugin::Plugin, Manager, Runtime};

use crate::state::AppState;

pub struct WSClonePlugin;

impl<R: Runtime> Plugin<R> for WSClonePlugin {
    fn name(&self) -> &'static str {
        "wsclone"
    }

    fn initialize(
        &mut self,
        app: &tauri::AppHandle<R>,
        _config: serde_json::Value,
    ) -> tauri::plugin::Result<()> {
        let app_state = AppState::new().unwrap();
        app.manage(app_state);
        Ok(())
    }
}
