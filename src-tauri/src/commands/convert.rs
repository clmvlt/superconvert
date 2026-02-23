use std::path::Path;

use tauri::AppHandle;

use crate::engine::{
    ArchiveFormat, AudioFormat, BatchConversionRequest, BatchConversionResult, ConversionError,
    DataFormat, DocumentFormat, FileInfo, ImageFormat, OutputFormatInfo, VideoFormat, file_category,
    resolve_format,
};
use crate::orchestrator::run_batch_conversion;

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    let target = Path::new(&path);
    if !target.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn scan_directory(path: String) -> Result<Vec<FileInfo>, String> {
    let dir_path = Path::new(&path);
    if !dir_path.is_dir() {
        return Err(format!("'{}' is not a directory", path));
    }

    let mut files = Vec::new();
    scan_dir_recursive(dir_path, &mut files).map_err(|e| e.to_string())?;
    Ok(files)
}

fn scan_dir_recursive(dir: &Path, files: &mut Vec<FileInfo>) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            scan_dir_recursive(&path, files)?;
            continue;
        }

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let format = resolve_format(&extension);
        if format.is_none() {
            continue;
        }

        let metadata = std::fs::metadata(&path)?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let path_str = path.to_string_lossy().to_string();

        files.push(FileInfo {
            path: path_str,
            name,
            extension,
            size: metadata.len(),
            format,
        });
    }
    Ok(())
}

#[tauri::command]
pub async fn get_files_info(paths: Vec<String>) -> Result<Vec<FileInfo>, String> {
    let mut files = Vec::new();

    for path_str in paths {
        let path = Path::new(&path_str);

        if !path.exists() {
            continue;
        }

        let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;

        if metadata.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let format = resolve_format(&extension);

        if file_category(&extension).is_none() {
            continue;
        }

        files.push(FileInfo {
            path: path_str,
            name,
            extension,
            size: metadata.len(),
            format,
        });
    }

    Ok(files)
}

#[tauri::command]
pub fn get_output_formats() -> Vec<OutputFormatInfo> {
    let mut formats = Vec::new();

    for f in ImageFormat::output_formats() {
        formats.push(OutputFormatInfo {
            format: f.extension().to_string(),
            extension: f.extension().to_string(),
            label: f.extension().to_uppercase(),
            supports_quality: f.supports_quality(),
            category: "image".to_string(),
        });
    }

    formats.push(OutputFormatInfo {
        format: "pdf".to_string(),
        extension: "pdf".to_string(),
        label: "PDF".to_string(),
        supports_quality: false,
        category: "image".to_string(),
    });

    for f in AudioFormat::output_formats() {
        formats.push(OutputFormatInfo {
            format: f.extension().to_string(),
            extension: f.extension().to_string(),
            label: f.extension().to_uppercase(),
            supports_quality: f.supports_quality(),
            category: "audio".to_string(),
        });
    }

    for f in DocumentFormat::pdf_output_formats() {
        formats.push(OutputFormatInfo {
            format: f.extension().to_string(),
            extension: f.extension().to_string(),
            label: f.extension().to_uppercase(),
            supports_quality: f.supports_quality(),
            category: "document".to_string(),
        });
    }

    for f in DocumentFormat::textdoc_output_formats() {
        formats.push(OutputFormatInfo {
            format: f.extension().to_string(),
            extension: f.extension().to_string(),
            label: f.extension().to_uppercase(),
            supports_quality: f.supports_quality(),
            category: "textdoc".to_string(),
        });
    }

    for f in DocumentFormat::spreadsheet_output_formats() {
        formats.push(OutputFormatInfo {
            format: f.extension().to_string(),
            extension: f.extension().to_string(),
            label: f.extension().to_uppercase(),
            supports_quality: f.supports_quality(),
            category: "spreadsheet".to_string(),
        });
    }

    for f in DocumentFormat::presentation_output_formats() {
        formats.push(OutputFormatInfo {
            format: f.extension().to_string(),
            extension: f.extension().to_string(),
            label: f.extension().to_uppercase(),
            supports_quality: f.supports_quality(),
            category: "presentation".to_string(),
        });
    }

    for f in VideoFormat::output_formats() {
        formats.push(OutputFormatInfo {
            format: f.extension().to_string(),
            extension: f.extension().to_string(),
            label: f.extension().to_uppercase(),
            supports_quality: f.supports_quality(),
            category: "video".to_string(),
        });
    }

    for f in DataFormat::output_formats() {
        formats.push(OutputFormatInfo {
            format: f.extension().to_string(),
            extension: f.extension().to_string(),
            label: f.extension().to_uppercase(),
            supports_quality: f.supports_quality(),
            category: "data".to_string(),
        });
    }

    for f in ArchiveFormat::output_formats() {
        formats.push(OutputFormatInfo {
            format: f.format_token().to_string(),
            extension: f.extension().to_string(),
            label: f.extension().to_uppercase(),
            supports_quality: f.supports_quality(),
            category: "archive".to_string(),
        });
    }

    formats
}

#[tauri::command]
pub fn get_available_features() -> Vec<String> {
    #[allow(unused_mut)]
    let mut features = vec![
        "image".to_string(),
        "audio-wav".to_string(),
        "audio-flac".to_string(),
        "audio-aiff".to_string(),
        "documents".to_string(),
        "data".to_string(),
        "archives".to_string(),
    ];

    #[cfg(feature = "mp3-encode")]
    features.push("mp3-encode".to_string());

    #[cfg(feature = "vorbis-encode")]
    features.push("vorbis-encode".to_string());

    #[cfg(feature = "opus-encode")]
    features.push("opus-encode".to_string());

    #[cfg(feature = "ffmpeg")]
    features.push("ffmpeg".to_string());

    #[cfg(feature = "heif")]
    features.push("heif".to_string());

    #[cfg(feature = "jxl")]
    features.push("jxl".to_string());

    #[cfg(feature = "raw-photos")]
    features.push("raw-photos".to_string());

    #[cfg(feature = "jpeg2000")]
    features.push("jpeg2000".to_string());

    #[cfg(feature = "rar")]
    features.push("rar".to_string());

    features
}

#[tauri::command]
pub async fn convert_files(
    app: AppHandle,
    request: BatchConversionRequest,
) -> Result<BatchConversionResult, String> {
    for job in &request.jobs {
        let input = Path::new(&job.input_path);
        if !input.exists() {
            return Err(ConversionError::FileNotFound(job.input_path.clone()).to_string());
        }
    }

    let result = tokio::task::spawn_blocking(move || run_batch_conversion(app, request))
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

    Ok(result)
}
