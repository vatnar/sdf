use sdf::UnconnectedTcpClient;
use std::io::{BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::thread::sleep;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[command[subcommand]]
    command: Commands,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long, env = "FS_PEER")]
    address: std::net::IpAddr,

    #[arg(short, long, env = "FS_PORT")]
    port: u16,
}

#[derive(Subcommand)]
enum Commands {
    Download{
        path: PathBuf,
        local_path: PathBuf,
    },
    Upload{
        path: PathBuf,
        local_path: PathBuf,
    },
    List {
        path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let peer = format!("{}:{}", cli.address, cli.port);
    let addr: SocketAddr = peer
        .parse()
        .with_context(|| format!("invalid peer address: {peer}"))?;


    match cli.command {
        Commands::Download { path, local_path } => {
            println!("download {:?} -> {:?}", path, local_path)
        }
        Commands::Upload { path, local_path} => {
            println!("upload {:?} -> {:?}", local_path, path)
        }
        Commands::List { path} => {
            println!("list {:?}", path)
        }
    }


    let client = UnconnectedTcpClient::new(addr);
    let mut client = client.connect()?;

    // TODO: Send command to server

    // PLAN: only support absolute paths, (which actually are relative to the sdf-server root), (to support multiple sdf-servers on one host etc).
    // TODO: Wait for response and handle it
    // TODO: Close

    client
        .stream
        .write_all(b"Hello, server!\n How are you?\n")?;

    let mut reader = BufReader::new(client.stream.try_clone()?);
    let mut line = String::new();

    sleep(std::time::Duration::from_millis(500));
    reader.read_line(&mut line)?;
    println!("got: {:?}", line);

    if client.disconnect().is_ok() {
        println!("client disconnected");
    }
    Ok(())
}
