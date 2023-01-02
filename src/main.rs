use libwsclone::download::{init_session_download, DownloadRule};
use libwsclone::session::Session;

#[tokio::main]
async fn main() {
    let format = tracing_subscriber::fmt::format().pretty();
    tracing_subscriber::fmt()
        .with_env_filter("libwsclone=debug")
        .event_format(format)
        .init();
    let mut session = Session {
        title: "".to_string(),
        index_url: "https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html"
            .to_string(),
        destination_directory: "./downloads".to_string(),
        processed_pages: Default::default(),
        processed_resource_links: Default::default(),
    };
    init_session_download(
        &mut session,
        DownloadRule {
            max_static_resource_download_size: 500000,
            progress_update_interval: 10000,
            should_download_resource_with_unknown_size: false,
            max_level: 1,
        },
    )
    .await
    .unwrap();
}
