use std::path::PathBuf;

use clap::Parser;
use sdf_server::{ServerError, TcpServer};

#[derive(Parser)]
struct Cli {
    #[arg(short, long, env = "FS_PORT", default_value = "9000")]
    port: u16,

    #[arg(long, env = "FS_DIR")]
    dir: PathBuf,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ServerError> {
    let cli = Cli::parse();

    let server = TcpServer::new(cli.port, cli.dir)?;

    server.start_server()?;
    Ok(())
}
