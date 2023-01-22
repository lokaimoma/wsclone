use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "WsClone daemon", version)]
pub struct DaemonCli {
    port: u8,
    #[arg(default_value_t = String::from("localhost"))]
    host: String,
}
