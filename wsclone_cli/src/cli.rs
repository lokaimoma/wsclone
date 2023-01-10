use chrono::Utc;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use libwsclone::{init_download, DownloadRule, Update};
use std::collections::HashMap;
use tokio::sync::mpsc::channel;
use url::Url;

const PROGRESS_UPDATE_INTERVAL: u64 = 1000;
const MAX_BUFFER_SIZE: usize = 100;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "An offline browser utility",
    long_about = "An offline browser utility for downloading website(s) for offline viewing."
)]
pub struct Cli {
    url: Url,
    output_directory: String,
    #[arg(default_value = "10000000", help = "Max file size in bytes.", long)]
    max_file_size: u64,
    #[arg(default_value = "1", long)]
    max_level: u8,
    #[arg(
        help = "Abort download if any resource other than the first page encounters an error.",
        long
    )]
    abort_on_download_error: Option<bool>,
    #[arg(
        help = "Download files even if their size can't be determined. Defaults to true.",
        long
    )]
    download_files_with_unknown_size: Option<bool>,
    #[arg(
        help = "A space separated list of texts/urls. All links will be checked if they contain \
        the text or the url, links that match the check won't be downloaded."
    )]
    blacklist_urls: Vec<String>,
}

pub async fn download(cli: Cli) {
    log::info!("Initializing download....");
    let mut pb_files: HashMap<String, ProgressBar> = HashMap::new();
    let (tx, mut rx) = channel::<Update>(MAX_BUFFER_SIZE);
    tokio::spawn(async move {
        match init_download(
            &format!("Session-{}", Utc::now().timestamp()),
            cli.url.as_ref(),
            &cli.output_directory,
            DownloadRule {
                max_level: cli.max_level,
                abort_on_download_error: cli.abort_on_download_error.unwrap_or(false),
                download_static_resource_with_unknown_size: cli
                    .download_files_with_unknown_size
                    .unwrap_or(true),
                progress_update_interval: PROGRESS_UPDATE_INTERVAL,
                max_static_file_size: cli.max_file_size,
                black_list_urls: cli.blacklist_urls.clone(),
            },
            tx,
        )
        .await
        {
            Ok(_) => {
                log::info!(
                    "Webpage(s) downloaded successfully. {}",
                    cli.output_directory
                );
            }
            Err(e) => {
                log::error!("Download wasn't able to complete");
                log::error!("{}", e);
            }
        }
    });
    while let Some(update) = rx.recv().await {
        match update {
            Update::MessageUpdate(msg) => {
                if msg.is_error {
                    log::error!("{} | {}", msg.content, msg.resource_name);
                } else {
                    log::info!("{} | {}", msg.content, msg.resource_name);
                }
            }
            Update::ProgressUpdate(prog) => {
                if let Some(pb) = pb_files.get(&prog.resource_name) {
                    pb.inc(prog.bytes_written);
                } else {
                    let sty = ProgressStyle::with_template(
                        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                    )
                    .unwrap()
                    .progress_chars("##-");
                    let pb = ProgressBar::new(prog.file_size);
                    pb.set_style(sty);
                    pb.tick();
                    pb.set_message(prog.resource_name.clone());
                    pb_files.insert(prog.resource_name, pb);
                }
            }
        };
    }
}
