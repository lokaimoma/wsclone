use crate::cli::DaemonCli;
use clap::Parser;

mod cli;
mod command_handlers;
mod error;

#[tokio::main]
async fn main() {
    let daemon_cli = DaemonCli::parse();
    daemon_cli.run_server().await.unwrap();
}
