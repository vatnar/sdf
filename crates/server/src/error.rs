use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("path escapes server root")]
    PathTraversal,
    #[error("requested path is not a directory")]
    NotADirectory,
    #[error("upload path must have a parent directory")]
    MissingParentDirectory,
    #[error("upload path must include a file name")]
    MissingFileName,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    InvalidInput(String),
    #[error("unknown command")]
    UnknownCommand,
    #[error("expected upload file chunk")]
    UnexpectedChunk,
}

impl From<ServerError> for sdf_protocol::Error {
    fn from(e: ServerError) -> Self {
        let code = match &e {
            ServerError::PathTraversal => 403,
            ServerError::NotFound => 404,
            ServerError::InvalidInput(_)
            | ServerError::NotADirectory
            | ServerError::MissingParentDirectory
            | ServerError::MissingFileName
            | ServerError::UnknownCommand
            | ServerError::UnexpectedChunk => 400,
            ServerError::Io(_) => 500,
        };
        sdf_protocol::Error {
            message: e.to_string(),
            code,
            special_fields: Default::default(),
        }
    }
}
