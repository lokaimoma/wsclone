use crate::cli::DaemonCli;
use clap::Parser;

mod cli;

#[tokio::main]
async fn main() {
    let daemon_cli = DaemonCli::parse();
}
