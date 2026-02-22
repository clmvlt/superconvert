use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[allow(dead_code)]
pub enum ConversionError {
    #[error("Unsupported input format: {0}")]
    UnsupportedInputFormat(String),

    #[error("Unsupported output format: {0}")]
    UnsupportedOutputFormat(String),

    #[error("Failed to read input file: {0}")]
    ReadError(String),

    #[error("Failed to write output file: {0}")]
    WriteError(String),

    #[error("Conversion failed: {0}")]
    ConversionFailed(String),

    #[error("Invalid options: {0}")]
    InvalidOptions(String),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

impl From<std::io::Error> for ConversionError {
    fn from(e: std::io::Error) -> Self {
        ConversionError::ReadError(e.to_string())
    }
}

impl From<image::ImageError> for ConversionError {
    fn from(e: image::ImageError) -> Self {
        ConversionError::ConversionFailed(e.to_string())
    }
}
