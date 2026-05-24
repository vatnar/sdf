use crate::error::ServerError;
use sdf_protocol::protocol::response;
use sdf_protocol::request::Command;
use sdf_protocol::{
    FileChunk, FileInfo, ListResponse, MAX_FILE_CHUNK_SIZE, Request, Response, read_message,
    write_message,
};
use std::io::{BufWriter, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::thread;
use std::{fs, io};

pub struct Server {
    port: u16,
    root: PathBuf,
}

impl Server {
    pub fn new(port: u16, root: impl Into<PathBuf>) -> Result<Self, ServerError> {
        let root = root.into().canonicalize()?;

        if !root.is_dir() {
            return Err(ServerError::InvalidInput(
                "server root must be a directory".to_string(),
            ));
        }

        Ok(Self { port, root })
    }

    pub fn start_server(&self) -> Result<(), ServerError> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let root = self.root.clone();

                    thread::spawn(move || {
                        if let Err(e) = handle_client(stream, root) {
                            eprintln!("client error: {e}");
                        }
                    });
                }
                Err(e) => eprintln!("accept error: {e}"),
            }
        }

        Ok(())
    }
}

fn handle_client(mut stream: TcpStream, root: PathBuf) -> Result<(), ServerError> {
    loop {
        // Read a length-delimited protobuf Request from the stream
        let req: Request = match read_message(&mut stream) {
            Ok(req) => req,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                println!("client disconnected");
                break;
            }
            Err(e) => return Err(e.into()),
        };

        match req.command {
            Some(Command::Download(req)) => {
                let mut resp = Response::new();
                match resolve_existing_path(&root, &req.path) {
                    Ok(path) => {
                        for chunk_resp in generate_download_responses(path)? {
                            write_message(&mut stream, &chunk_resp)?;
                        }
                        continue;
                    }
                    Err(e) => {
                        resp.payload = Some(response::Payload::Error(e.into()));
                    }
                }
                write_message(&mut stream, &resp)?;
            }
            Some(Command::Upload(req)) => {
                let mut resp = Response::new();
                match resolve_upload_path(&root, &req.path) {
                    Ok(path) => {
                        handle_upload(&mut stream, path)?;
                        resp.payload = Some(response::Payload::Ok(true));
                    }
                    Err(e) => {
                        resp.payload = Some(response::Payload::Error(e.into()));
                    }
                }
                write_message(&mut stream, &resp)?;
            }
            Some(Command::List(req)) => {
                let mut resp = Response::new();
                match list_files(&root, &req.path) {
                    Ok(entries) => {
                        resp.payload = Some(response::Payload::List(ListResponse {
                            entries,
                            special_fields: Default::default(),
                        }));
                    }
                    Err(e) => {
                        resp.payload = Some(response::Payload::Error(e.into()));
                    }
                }
                write_message(&mut stream, &resp)?;
            }
            _ => {
                let mut resp = Response::new();
                resp.payload = Some(response::Payload::Error(ServerError::UnknownCommand.into()));
                write_message(&mut stream, &resp)?;
            }
        }
    }

    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn handle_upload(stream: &mut TcpStream, path: PathBuf) -> Result<(), ServerError> {
    let mut data: Vec<u8> = Vec::new();
    let mut is_last = false;

    while !is_last {
        let req: Request = read_message(stream)?;

        match req.command {
            Some(Command::FileChunk(chunk)) => {
                data.extend_from_slice(&chunk.data);
                is_last = chunk.is_last;
            }
            _ => {
                let mut resp = Response::new();
                resp.payload = Some(response::Payload::Error(ServerError::UnexpectedChunk.into()));
                write_message(stream, &resp)?;
                continue;
            }
        }
    }

    BufWriter::new(fs::File::create(path)?).write_all(&data)?;
    Ok(())
}

fn generate_download_responses(path: PathBuf) -> Result<Vec<Response>, ServerError> {
    let mut responses = Vec::new();

    println!("download {}", path.display());

    let data: Vec<u8> = fs::read(&path)?;
    let file_size = data.len() as u64;
    let mut bytes_sent = 0_u64;

    for chunk in data.chunks(MAX_FILE_CHUNK_SIZE) {
        bytes_sent += chunk.len() as u64;

        let mut resp = Response::new();
        resp.payload = Some(response::Payload::FileChunk(FileChunk {
            data: chunk.to_vec(),
            is_last: bytes_sent == file_size,
            special_fields: Default::default(),
        }));

        responses.push(resp);
    }

    if file_size == 0 {
        let mut resp = Response::new();
        resp.payload = Some(response::Payload::FileChunk(FileChunk {
            data: Vec::new(),
            is_last: true,
            special_fields: Default::default(),
        }));
        responses.push(resp);
    }

    Ok(responses)
}

fn list_files(root: &Path, req_path: &str) -> Result<Vec<FileInfo>, ServerError> {
    println!("list files in {}", req_path);

    let path = resolve_existing_path(root, req_path)?;

    if !path.is_dir() {
        return Err(ServerError::NotADirectory);
    }

    let mut files = Vec::new();

    for entry in path.read_dir()? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let file_type = entry.file_type()?;

        files.push(FileInfo {
            name: entry.file_name().to_string_lossy().to_string(),
            size: metadata.len(),
            is_directory: file_type.is_dir(),
            special_fields: Default::default(),
        });
    }

    Ok(files)
}

fn resolve_existing_path(root: &Path, client_path: &str) -> Result<PathBuf, ServerError> {
    let joined = join_client_path(root, client_path)?;
    let canonical = joined.canonicalize()?;

    if !canonical.starts_with(root) {
        return Err(ServerError::PathTraversal);
    }

    Ok(canonical)
}

fn resolve_upload_path(root: &Path, client_path: &str) -> Result<PathBuf, ServerError> {
    let joined = join_client_path(root, client_path)?;

    let parent = joined
        .parent()
        .ok_or(ServerError::MissingParentDirectory)?;

    let canonical_parent = parent.canonicalize()?;

    if !canonical_parent.starts_with(root) {
        return Err(ServerError::PathTraversal);
    }

    let file_name = joined
        .file_name()
        .ok_or(ServerError::MissingFileName)?;

    Ok(canonical_parent.join(file_name))
}

fn join_client_path(root: &Path, client_path: &str) -> Result<PathBuf, ServerError> {
    let client_path = Path::new(client_path);
    let mut safe_path = PathBuf::new();

    for component in client_path.components() {
        match component {
            Component::RootDir | Component::CurDir => {}
            Component::Normal(part) => safe_path.push(part),
            Component::ParentDir | Component::Prefix(_) => {
                return Err(ServerError::PathTraversal);
            }
        }
    }

    Ok(root.join(safe_path))
}
