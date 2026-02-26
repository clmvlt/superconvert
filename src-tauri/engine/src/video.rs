#![cfg(feature = "ffmpeg")]

use std::path::Path;

use crate::error::ConversionError;
use crate::traits::Converter;
use crate::types::ConversionOptions;

pub struct VideoConverter;

impl VideoConverter {
    pub fn new() -> Self {
        ffmpeg_next::init()
            .unwrap_or_else(|e| tracing::warn!("FFmpeg init warning: {}", e));
        Self
    }
}

impl Converter for VideoConverter {
    fn supported_input_formats(&self) -> &[&str] {
        &[
            "mp4", "avi", "mkv", "mov", "webm", "flv", "wmv", "mpeg", "mpg", "ts", "3gp", "m4v",
            "vob",
        ]
    }

    fn supported_output_formats(&self) -> &[&str] {
        &[
            "mp4", "avi", "mkv", "mov", "webm", "flv", "wmv", "mpeg", "ts", "3gp", "m4v",
        ]
    }

    fn convert(
        &self,
        input: &Path,
        output: &Path,
        options: &ConversionOptions,
        on_progress: Box<dyn Fn(f32) + Send>,
    ) -> Result<(), ConversionError> {
        on_progress(0.0);

        let input_ctx = ffmpeg_next::format::input(&input)
            .map_err(|e| ConversionError::ReadError(format!("FFmpeg input error: {}", e)))?;

        let output_ext = options.output_format.extension();

        let mut output_ctx = ffmpeg_next::format::output(&output)
            .map_err(|e| ConversionError::WriteError(format!("FFmpeg output error: {}", e)))?;

        let video_stream_idx = input_ctx
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .map(|s| s.index());

        let audio_stream_idx = input_ctx
            .streams()
            .best(ffmpeg_next::media::Type::Audio)
            .map(|s| s.index());

        let total_duration = input_ctx.duration() as f64 / f64::from(ffmpeg_next::ffi::AV_TIME_BASE);

        for stream in input_ctx.streams() {
            let codec = stream.parameters();
            let mut out_stream = output_ctx
                .add_stream(ffmpeg_next::encoder::find(ffmpeg_next::codec::Id::None))
                .map_err(|e| {
                    ConversionError::ConversionFailed(format!(
                        "FFmpeg add stream error: {}",
                        e
                    ))
                })?;
            out_stream.set_parameters(codec);
        }

        output_ctx
            .write_header()
            .map_err(|e| {
                ConversionError::WriteError(format!("FFmpeg write header error: {}", e))
            })?;

        let mut packet_count: u64 = 0;
        for (stream, packet) in input_ctx.packets() {
            let out_stream = output_ctx.stream(stream.index()).unwrap();
            let mut pkt = packet.clone();
            pkt.rescale_ts(stream.time_base(), out_stream.time_base());
            pkt.set_stream(stream.index());
            pkt.write_interleaved(&mut output_ctx)
                .map_err(|e| {
                    ConversionError::WriteError(format!("FFmpeg write packet error: {}", e))
                })?;

            packet_count += 1;
            if packet_count % 100 == 0 && total_duration > 0.0 {
                let ts = packet.pts().unwrap_or(0) as f64
                    * f64::from(stream.time_base());
                let progress = (ts / total_duration).min(0.95);
                on_progress(progress as f32);
            }
        }

        output_ctx
            .write_trailer()
            .map_err(|e| {
                ConversionError::WriteError(format!("FFmpeg write trailer error: {}", e))
            })?;

        on_progress(1.0);
        Ok(())
    }
}
