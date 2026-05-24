use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use sdf::Client;

#[derive(Parser)]
struct Cli {
    #[command[subcommand]]
    command: Commands,

    #[arg(short, long)]
    verbose: bool,

    #[arg(
        short,
        long,
        env = "FS_PEER",
        global = true,
        default_value = "127.0.0.1"
    )]
    address: std::net::IpAddr,

    #[arg(short, long, env = "FS_PORT", global = true, default_value = "9000")]
    port: u16,
}

#[derive(Subcommand)]
enum Commands {
    Download { path: PathBuf, local_path: PathBuf },
    Upload { path: PathBuf, local_path: PathBuf },
    List { path: Option<PathBuf> },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let peer = format!("{}:{}", cli.address, cli.port);
    let addr: SocketAddr = peer
        .parse()
        .with_context(|| format!("invalid peer address: {peer}"))?;

    let mut client = Client::connect(addr)?;

    match cli.command {
        Commands::Download { path, local_path } => {
            sdf::commands::download(&mut client, &path.to_string_lossy(), &local_path)?;
        }
        Commands::Upload { path, local_path } => {
            sdf::commands::upload(&mut client, &local_path, &path.to_string_lossy())?;
        }
        Commands::List { path } => {
            let path = path
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string());
            sdf::commands::list(&mut client, &path)?;
        }
    }

    client.disconnect()?;
    Ok(())
}
