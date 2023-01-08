use chrono::Utc;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use libwsclone::{init_download, DownloadRule, Update};
use std::collections::HashMap;
use url::Url;

const PROGRESS_UPDATE_INTERVAL: u64 = 1000;

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
        help = "Abort download if an error status code is received(400-599). Defaults to false.",
        long
    )]
    abort_on_error_status: Option<bool>,
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

impl Cli {
    pub async fn download(&self) {
        log::info!("Initializing download....");
        let mut pb_files: HashMap<String, ProgressBar> = HashMap::new();
        let on_update = |update: Update| async {
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
            }
        };
        match init_download(
            &format!("Session-{}", Utc::now().timestamp()),
            self.url.as_ref(),
            &self.output_directory,
            DownloadRule {
                max_level: self.max_level,
                abort_on_error_status: self.abort_on_error_status.unwrap_or(false),
                download_static_resource_with_unknown_size: self
                    .download_files_with_unknown_size
                    .unwrap_or(true),
                progress_update_interval: PROGRESS_UPDATE_INTERVAL,
                max_static_file_size: self.max_file_size,
                black_list_urls: self.blacklist_urls.clone(),
            },
            on_update,
        )
        .await
        {
            Ok(_) => {
                log::info!(
                    "Webpage(s) downloaded successfully. {}",
                    self.output_directory
                );
            }
            Err(e) => {
                log::error!("Download wasn't able to complete");
                log::error!("{}", e);
            }
        };
    }
}
