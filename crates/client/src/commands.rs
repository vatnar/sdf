use std::path::Path;

use anyhow::{Context, Result};
use crate::Client;

pub fn download(client: &mut Client, remote: &str, local: &Path) -> Result<()> {
    let data = client.download(remote)?;
    std::fs::write(local, &data)
        .with_context(|| format!("failed to write downloaded file: {}", local.display()))?;
    Ok(())
}

pub fn upload(client: &mut Client, local: &Path, remote: &str) -> Result<()> {
    println!("upload {:?} -> {:?}", local, remote);
    client.upload(remote, local)?;
    println!("upload complete");
    Ok(())
}

pub fn list(client: &mut Client, path: &str) -> Result<()> {
    println!("list {}", path);
    let entries = client.list(path)?;
    entries.iter().for_each(|e| println!("{:?}", e));
    Ok(())
}
