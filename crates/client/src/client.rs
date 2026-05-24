use std::net::{SocketAddr, TcpStream};
use std::path::Path;

use anyhow::{Context, Result};
use sdf_protocol::protocol::{request, DownloadRequest, ListRequest, UploadRequest};
use sdf_protocol::{
    FileChunk, Request, Response, read_message, response, write_message, MAX_FILE_CHUNK_SIZE,
};

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .with_context(|| format!("failed to connect to {addr}"))?;
        Ok(Self { stream })
    }

    pub fn download(&mut self, remote_path: &str) -> Result<Vec<u8>> {
        let mut req = Request::new();
        req.command = Some(request::Command::Download(DownloadRequest {
            path: remote_path.to_string(),
            ..Default::default()
        }));
        write_message(&mut self.stream, &req)?;

        let resp: Response = read_message(&mut self.stream)?;
        match resp.payload {
            Some(response::Payload::FileChunk(first_chunk)) => {
                let mut data = first_chunk.data;
                let mut is_last = first_chunk.is_last;

                while !is_last {
                    let resp: Response = read_message(&mut self.stream)?;
                    match resp.payload {
                        Some(response::Payload::FileChunk(chunk)) => {
                            data.extend_from_slice(&chunk.data);
                            is_last = chunk.is_last;
                        }
                        _ => {
                            return Err(anyhow::anyhow!(
                                "unexpected payload from server during file download"
                            ));
                        }
                    }
                }
                Ok(data)
            }
            Some(response::Payload::Error(err)) => Err(anyhow::anyhow!(
                "server error: {} (code: {})",
                err.message,
                err.code
            )),
            _ => Err(anyhow::anyhow!("unexpected payload from server")),
        }
    }

    pub fn upload(&mut self, remote_path: &str, local_path: &Path) -> Result<()> {
        let file_size = local_path
            .metadata()
            .with_context(|| format!(
                "failed to read metadata for upload file: {}",
                local_path.display()
            ))?
            .len();

        let mut req = Request::new();
        req.command = Some(request::Command::Upload(UploadRequest {
            path: remote_path.to_string(),
            file_size,
            ..Default::default()
        }));
        write_message(&mut self.stream, &req)?;

        let data: Vec<u8> = std::fs::read(local_path)
            .with_context(|| format!(
                "failed to read upload file: {}",
                local_path.display()
            ))?;
        let mut bytes_sent = 0_u64;

        for chunk in data.chunks(MAX_FILE_CHUNK_SIZE) {
            bytes_sent += chunk.len() as u64;

            let mut req = Request::new();
            req.command = Some(request::Command::FileChunk(FileChunk {
                data: chunk.to_vec(),
                is_last: bytes_sent == file_size,
                special_fields: Default::default(),
            }));
            write_message(&mut self.stream, &req)?;
        }

        if file_size == 0 {
            let mut req = Request::new();
            req.command = Some(request::Command::FileChunk(FileChunk {
                data: Vec::new(),
                is_last: true,
                special_fields: Default::default(),
            }));
            write_message(&mut self.stream, &req)?;
        }

        let resp: Response = read_message(&mut self.stream)?;
        match resp.payload {
            Some(response::Payload::Ok(_)) => Ok(()),
            Some(response::Payload::Error(err)) => Err(anyhow::anyhow!(
                "server error: {} (code: {})",
                err.message,
                err.code
            )),
            _ => Err(anyhow::anyhow!("unexpected payload from server after upload")),
        }
    }

    pub fn list(&mut self, remote_path: &str) -> Result<Vec<sdf_protocol::FileInfo>> {
        let mut req = Request::new();
        req.command = Some(request::Command::List(ListRequest {
            path: remote_path.to_string(),
            ..Default::default()
        }));
        write_message(&mut self.stream, &req)?;

        let resp: Response = read_message(&mut self.stream)?;
        match resp.payload {
            Some(response::Payload::List(list)) => Ok(list.entries),
            Some(response::Payload::Error(err)) => Err(anyhow::anyhow!(
                "server error: {} (code: {})",
                err.message,
                err.code
            )),
            _ => Err(anyhow::anyhow!("unexpected payload from server")),
        }
    }

    pub fn disconnect(self) -> Result<()> {
        if let Err(e) = self.stream.shutdown(std::net::Shutdown::Both) {
            eprintln!("warning: failed to disconnect cleanly: {e}");
        } else {
            println!("client disconnected");
        }
        Ok(())
    }
}
