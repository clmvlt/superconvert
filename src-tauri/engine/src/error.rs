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

impl From<quick_xml::Error> for ConversionError {
    fn from(e: quick_xml::Error) -> Self {
        ConversionError::ReadError(format!("XML parse error: {}", e))
    }
}

impl From<zip::result::ZipError> for ConversionError {
    fn from(e: zip::result::ZipError) -> Self {
        ConversionError::ReadError(format!("ZIP error: {}", e))
    }
}

impl From<csv::Error> for ConversionError {
    fn from(e: csv::Error) -> Self {
        ConversionError::ConversionFailed(format!("CSV error: {}", e))
    }
}

impl From<serde_json::Error> for ConversionError {
    fn from(e: serde_json::Error) -> Self {
        ConversionError::ConversionFailed(format!("JSON error: {}", e))
    }
}

impl From<serde_yaml::Error> for ConversionError {
    fn from(e: serde_yaml::Error) -> Self {
        ConversionError::ConversionFailed(format!("YAML error: {}", e))
    }
}

impl From<toml::de::Error> for ConversionError {
    fn from(e: toml::de::Error) -> Self {
        ConversionError::ConversionFailed(format!("TOML parse error: {}", e))
    }
}

impl From<toml::ser::Error> for ConversionError {
    fn from(e: toml::ser::Error) -> Self {
        ConversionError::ConversionFailed(format!("TOML serialize error: {}", e))
    }
}
