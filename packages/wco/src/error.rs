use thiserror::Error;

/// Error types for WCO operations
#[derive(Error, Debug)]
pub enum WcoError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, WcoError>;
