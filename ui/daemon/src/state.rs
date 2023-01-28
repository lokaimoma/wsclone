use libwsclone::Update;
use tokio::sync::mpsc::Sender;

pub struct DaemonState {
    queued_link: Vec<String>,
    pub tx: Sender<Update>,
}
