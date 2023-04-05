use tauri::{plugin::Plugin, Invoke, Manager, Runtime, State};

use crate::state::AppState;
use ws_common::command::Command;

pub struct WSClonePlugin<R: Runtime> {
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync + 'static>,
}

#[tauri::command]
fn handle_command<'a>(_app_state: State<'a, AppState>, json_payload: Command) {
    println!("Hello command received");
    println!("{:?}", json_payload);
}

impl<R: Runtime> WSClonePlugin<R> {
    pub fn new() -> Self {
        Self {
            invoke_handler: Box::new(tauri::generate_handler![handle_command]),
        }
    }
}


impl<R: Runtime> Plugin<R> for WSClonePlugin<R> {
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

    fn extend_api(&mut self, invoke: Invoke<R>) {
        (self.invoke_handler)(invoke)
    }
}
