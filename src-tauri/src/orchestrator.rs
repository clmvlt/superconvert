use std::sync::Arc;

use rayon::prelude::*;
use tauri::{AppHandle, Emitter};

use crate::engine::{
    ArchiveConverter, AudioConverter, BatchConversionRequest, BatchConversionResult, Converter,
    DataConverter, ImageConverter, JobResult, JobStatus, PdfConverter, PresentationConverter,
    ProgressEvent, SpreadsheetConverter, TextDocConverter, file_category,
};

#[cfg(feature = "ffmpeg")]
use crate::engine::VideoConverter;

fn dispatch_converter(input_ext: &str) -> Arc<dyn Converter> {
    match file_category(input_ext) {
        Some("audio") => Arc::new(AudioConverter::new()),
        Some("document") => Arc::new(PdfConverter::new()),
        Some("textdoc") => Arc::new(TextDocConverter::new()),
        Some("spreadsheet") => Arc::new(SpreadsheetConverter::new()),
        Some("presentation") => Arc::new(PresentationConverter::new()),
        Some("data") => Arc::new(DataConverter::new()),
        Some("archive") => Arc::new(ArchiveConverter::new()),
        #[cfg(feature = "ffmpeg")]
        Some("video") => Arc::new(VideoConverter::new()),
        _ => Arc::new(ImageConverter::new()),
    }
}

pub fn run_batch_conversion(
    app: AppHandle,
    request: BatchConversionRequest,
) -> BatchConversionResult {
    let total = request.jobs.len();

    let results: Vec<JobResult> = request
        .jobs
        .par_iter()
        .map(|job| {
            let app = app.clone();
            let job_id = job.id.clone();

            let _ = app.emit(
                "conversion:progress",
                ProgressEvent {
                    job_id: job_id.clone(),
                    progress: 0.0,
                    status: JobStatus::Converting,
                    error: None,
                },
            );

            let on_progress = {
                let app = app.clone();
                let job_id = job_id.clone();
                Box::new(move |progress: f32| {
                    let _ = app.emit(
                        "conversion:progress",
                        ProgressEvent {
                            job_id: job_id.clone(),
                            progress,
                            status: JobStatus::Converting,
                            error: None,
                        },
                    );
                })
            };

            let input = std::path::Path::new(&job.input_path);
            let output = std::path::Path::new(&job.output_path);

            if let Some(parent) = output.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let input_ext = input
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let converter = dispatch_converter(&input_ext);

            match converter.convert(input, output, &job.options, on_progress) {
                Ok(()) => {
                    let _ = app.emit(
                        "conversion:progress",
                        ProgressEvent {
                            job_id: job_id.clone(),
                            progress: 1.0,
                            status: JobStatus::Done,
                            error: None,
                        },
                    );
                    JobResult {
                        job_id,
                        success: true,
                        output_path: Some(job.output_path.clone()),
                        error: None,
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    let _ = app.emit(
                        "conversion:progress",
                        ProgressEvent {
                            job_id: job_id.clone(),
                            progress: 0.0,
                            status: JobStatus::Error,
                            error: Some(error_msg.clone()),
                        },
                    );
                    JobResult {
                        job_id,
                        success: false,
                        output_path: None,
                        error: Some(error_msg),
                    }
                }
            }
        })
        .collect();

    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();

    BatchConversionResult {
        total,
        succeeded,
        failed,
        results,
    }
}
