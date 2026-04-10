#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IOError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("{0}")]
    TaskFailed(String),
    #[error("Task was cancelled")]
    TaskCancelled,
    #[error("Task not found")]
    TaskNotFound,
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("{0}")]
    ReqwestError(String),
    #[error("HTTP ERROR: {0}")]
    HttpError(String),
    #[error("Stream Error: {0}")]
    StreamError(String),
    #[error("JSON Error: {0}")]
    JsonError(String),
    #[error("Checksum verification failed")]
    ChecksumVerificationFailed,
    #[error("Cannot remove task in current status")]
    CannotRemoveTaskInCurrentStatus,
    #[error("Storage path error: {0}")]
    PathError(String),
    #[error("BitTorrent error: {0}")]
    BtError(String),
    #[error("Unknown error")]
    Unknown,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(value: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl From<tokio::sync::AcquireError> for Error {
    fn from(value: tokio::sync::AcquireError) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonError(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
