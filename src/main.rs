use libwsclone::{init_download, DownloadRule, Update};

async fn print(update: Update) {
    print!("{:?}", update);
}

#[tokio::main]
async fn main() {
    let format = tracing_subscriber::fmt::format().pretty();
    tracing_subscriber::fmt()
        .with_env_filter("libwsclone=warn")
        .event_format(format)
        .init();
    init_download(
        "clio123",
        "https://docs.rs/tokio/latest/tokio/fs/struct.OpenOptions.html",
        "/media/local_disk/windows_back_up/projects/rust/myowns/wsclone/downloads",
        DownloadRule {
            max_level: 2,
            max_static_file_size: 10000000,
            progress_update_interval: 5000,
            download_static_resource_with_unknown_size: false,
            black_list_urls: Vec::new(),
        },
        print,
    )
    .await
    .unwrap();
}
