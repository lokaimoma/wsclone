use https://en.wikipedia.org/wiki/Osbert_Parsleycrate::cli::download;
use clap::Parser;
use std::path::MAIN_SEPARATOR;

mod cli;

#[tokio::main]
async fn main() {
    let f_appender =
        tracing_appender::rolling::hourly(format!(".{}", MAIN_SEPARATOR), "wsclone.log");
    let (non_blk, _guard) = tracing_appender::non_blocking(f_appender);
    tracing_subscriber::fmt()
        .event_format(tracing_subscriber::fmt::format().pretty())
        .with_writer(non_blk)
        .init();
    // pretty_env_logger::formatted_builder()
    //     .filter(Some("wsclone"), log::LevelFilter::Trace)
    //     .init();
    let cli = cli::Cli::parse();
    download(cli).await;
}
