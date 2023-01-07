use libwsclone::{init_download, DownloadRule, Update};

async fn print(update: Update) {
    print!("{:?}", update);
}

#[tokio::main]
async fn main() {
    let format = tracing_subscriber::fmt::format().pretty();
    tracing_subscriber::fmt()
        .with_env_filter("libwsclone=debug")
        .event_format(format)
        .init();
    init_download(
        "clio123",
        "https://jakewharton.com/",
        "/media/local_disk/windows_back_up/projects/rust/myowns/wsclone/downloads",
        DownloadRule {
            max_level: 1,
            max_static_file_size: 10000000,
            progress_update_interval: 5000,
            download_static_resource_with_unknown_size: false,
            black_list_urls: Vec::new(),
            abort_on_error_status: false,
        },
        print,
    )
    .await
    .unwrap();
}