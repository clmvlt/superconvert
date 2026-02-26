pub mod archive;
pub mod audio;
pub mod data;
pub mod document;
pub mod error;
pub mod image;
pub mod presentation;
pub mod spreadsheet;
pub mod traits;
pub mod types;
#[cfg(feature = "ffmpeg")]
pub mod video;

pub use self::archive::ArchiveConverter;
pub use self::audio::AudioConverter;
pub use self::data::DataConverter;
pub use self::document::DocumentConverter;
pub use self::image::ImageConverter;
pub use self::presentation::PresentationConverter;
pub use self::spreadsheet::SpreadsheetConverter;
#[cfg(feature = "ffmpeg")]
pub use self::video::VideoConverter;
pub use error::ConversionError;
pub use traits::Converter;
pub use types::*;

use std::path::Path;
use std::sync::Arc;

/// Dispatch the correct converter based on input file extension category.
pub fn dispatch_converter(input_ext: &str) -> Arc<dyn Converter> {
    match file_category(input_ext) {
        Some("audio") => Arc::new(AudioConverter::new()),
        Some("document") => Arc::new(DocumentConverter::new()),
        Some("spreadsheet") => Arc::new(SpreadsheetConverter::new()),
        Some("presentation") => Arc::new(PresentationConverter::new()),
        Some("data") => Arc::new(DataConverter::new()),
        Some("archive") => Arc::new(ArchiveConverter::new()),
        #[cfg(feature = "ffmpeg")]
        Some("video") => Arc::new(VideoConverter::new()),
        _ => Arc::new(ImageConverter::new()),
    }
}

/// Convert a single file (no Tauri dependency).
/// Input and output paths are provided directly.
pub fn convert_single_file(
    input: &Path,
    output: &Path,
    options: &ConversionOptions,
) -> Result<(), ConversionError> {
    if !input.exists() {
        return Err(ConversionError::FileNotFound(
            input.display().to_string(),
        ));
    }

    if let Some(parent) = output.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let input_ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let converter = dispatch_converter(&input_ext);
    converter.convert(input, output, options, Box::new(|_| {}))
}

/// List output formats available for a given input extension.
pub fn output_formats_for_extension(input_ext: &str) -> Vec<OutputFormatInfo> {
    let ext = input_ext.to_lowercase();
    let category = match file_category(&ext) {
        Some(c) => c,
        None => return Vec::new(),
    };

    let mut formats = Vec::new();

    match category {
        "image" => {
            for f in ImageFormat::output_formats() {
                if f.extension() != ext {
                    formats.push(OutputFormatInfo {
                        format: f.extension().to_string(),
                        extension: f.extension().to_string(),
                        label: f.extension().to_uppercase(),
                        supports_quality: f.supports_quality(),
                        category: "image".to_string(),
                    });
                }
            }
            // Also add PDF for images
            if ext != "pdf" {
                formats.push(OutputFormatInfo {
                    format: "pdf".to_string(),
                    extension: "pdf".to_string(),
                    label: "PDF".to_string(),
                    supports_quality: false,
                    category: "image".to_string(),
                });
            }
        }
        "audio" => {
            for f in AudioFormat::output_formats() {
                if f.extension() != ext {
                    formats.push(OutputFormatInfo {
                        format: f.extension().to_string(),
                        extension: f.extension().to_string(),
                        label: f.extension().to_uppercase(),
                        supports_quality: f.supports_quality(),
                        category: "audio".to_string(),
                    });
                }
            }
        }
        "video" => {
            for f in VideoFormat::output_formats() {
                if f.extension() != ext {
                    formats.push(OutputFormatInfo {
                        format: f.extension().to_string(),
                        extension: f.extension().to_string(),
                        label: f.extension().to_uppercase(),
                        supports_quality: f.supports_quality(),
                        category: "video".to_string(),
                    });
                }
            }
        }
        "document" => {
            for f in DocumentFormat::document_output_formats() {
                if f.extension() != ext {
                    formats.push(OutputFormatInfo {
                        format: f.extension().to_string(),
                        extension: f.extension().to_string(),
                        label: f.extension().to_uppercase(),
                        supports_quality: f.supports_quality(),
                        category: "document".to_string(),
                    });
                }
            }
        }
        "spreadsheet" => {
            for f in DocumentFormat::spreadsheet_output_formats() {
                if f.extension() != ext {
                    formats.push(OutputFormatInfo {
                        format: f.extension().to_string(),
                        extension: f.extension().to_string(),
                        label: f.extension().to_uppercase(),
                        supports_quality: f.supports_quality(),
                        category: "spreadsheet".to_string(),
                    });
                }
            }
        }
        "presentation" => {
            for f in DocumentFormat::presentation_output_formats() {
                if f.extension() != ext {
                    formats.push(OutputFormatInfo {
                        format: f.extension().to_string(),
                        extension: f.extension().to_string(),
                        label: f.extension().to_uppercase(),
                        supports_quality: f.supports_quality(),
                        category: "presentation".to_string(),
                    });
                }
            }
        }
        "data" => {
            for f in DataFormat::output_formats() {
                if f.extension() != ext {
                    formats.push(OutputFormatInfo {
                        format: f.extension().to_string(),
                        extension: f.extension().to_string(),
                        label: f.extension().to_uppercase(),
                        supports_quality: f.supports_quality(),
                        category: "data".to_string(),
                    });
                }
            }
        }
        "archive" => {
            for f in ArchiveFormat::output_formats() {
                if f.extension() != ext {
                    formats.push(OutputFormatInfo {
                        format: f.format_token().to_string(),
                        extension: f.extension().to_string(),
                        label: f.extension().to_uppercase(),
                        supports_quality: f.supports_quality(),
                        category: "archive".to_string(),
                    });
                }
            }
        }
        _ => {}
    }

    formats
}

/// List all supported input extensions.
pub fn all_supported_extensions() -> Vec<&'static str> {
    vec![
        // Image
        "png", "jpg", "jpeg", "gif", "bmp", "ico", "tif", "tiff", "webp", "avif",
        "svg", "tga", "dds", "qoi", "hdr", "ppm", "pgm", "pbm", "exr", "psd",
        "heif", "heic", "cr2", "nef", "arw", "dng", "orf", "rw2", "jxl", "jp2", "j2k",
        // Audio
        "mp3", "wav", "flac", "ogg", "aac", "aiff", "aif", "m4a", "alac", "opus",
        "wma", "ac3", "dts",
        // Video
        "mp4", "avi", "mkv", "mov", "webm", "flv", "wmv", "mpeg", "mpg", "ts",
        "3gp", "m4v", "vob",
        // Document
        "pdf", "docx", "odt", "txt", "rtf", "epub",
        // Spreadsheet
        "xlsx", "xls", "ods", "csv",
        // Presentation
        "pptx", "odp",
        // Data
        "json", "yaml", "yml", "toml", "xml", "md", "html", "htm",
        // Archive
        "zip", "tar", "gz", "tgz", "bz2", "xz", "7z", "rar", "zst",
    ]
}
