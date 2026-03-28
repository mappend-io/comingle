use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("URI error: {0}")]
    BadUri(String),
    #[error("URI validation error: {0}")]
    InvalidUri(#[from] iri_string::validate::Error),
    #[error("Archive error: {0}")]
    BadArchive(String),
    #[error("Decompression error: {0}")]
    Decompression(String),
}
