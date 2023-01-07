use clap::Parser;

mod cli;

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter(Some("wsclone"), log::LevelFilter::Trace)
        .init();
    let cli = cli::Cli::parse();
    cli.download().await;
}
