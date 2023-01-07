use clap::Parser;

mod cli;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let cli = cli::Cli::parse();
    cli.download().await;
}
