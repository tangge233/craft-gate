use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Serialization error: {0}")]
    Serialization(#[from] anyhow::Error),
    #[error("FileSystem error: {0}")]
    IO(#[from] tokio::io::Error),
    #[error("Invalid URL scheme")]
    InvalidUrl,
}
