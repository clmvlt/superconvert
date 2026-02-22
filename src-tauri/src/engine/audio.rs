use std::fs::File;
use std::path::Path;

use hound::{SampleFormat, WavSpec, WavWriter};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use super::error::ConversionError;
use super::traits::Converter;
use super::types::ConversionOptions;

pub struct AudioConverter;

impl AudioConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Converter for AudioConverter {
    fn supported_input_formats(&self) -> &[&str] {
        &["mp3", "wav", "flac", "ogg", "aac", "aiff", "aif", "m4a"]
    }

    fn supported_output_formats(&self) -> &[&str] {
        &["wav"]
    }

    fn convert(
        &self,
        input: &Path,
        output: &Path,
        options: &ConversionOptions,
        on_progress: Box<dyn Fn(f32) + Send>,
    ) -> Result<(), ConversionError> {
        on_progress(0.0);

        let input_ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !self.supported_input_formats().contains(&input_ext.as_str()) {
            return Err(ConversionError::UnsupportedInputFormat(input_ext));
        }

        let output_ext = options.output_format.extension();
        if !self.supported_output_formats().contains(&output_ext) {
            return Err(ConversionError::UnsupportedOutputFormat(
                output_ext.to_string(),
            ));
        }

        if !input.exists() {
            return Err(ConversionError::FileNotFound(input.display().to_string()));
        }

        let file = File::open(input).map_err(|e| ConversionError::ReadError(e.to_string()))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension(&input_ext);

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| {
                ConversionError::ReadError(format!("Failed to probe audio format: {}", e))
            })?;

        let mut format_reader = probed.format;

        let track = format_reader
            .default_track()
            .ok_or_else(|| ConversionError::ReadError("No audio track found".to_string()))?;

        let track_id = track.id;
        let channels = track
            .codec_params
            .channels
            .map(|c| c.count())
            .unwrap_or(2) as u16;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let n_frames = track.codec_params.n_frames;

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| {
                ConversionError::ReadError(format!("Failed to create audio decoder: {}", e))
            })?;

        let mut all_samples: Vec<i16> = Vec::new();
        let mut decoded_frames: u64 = 0;

        loop {
            let packet = match format_reader.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(e) => return Err(ConversionError::ReadError(e.to_string())),
            };

            if packet.track_id() != track_id {
                continue;
            }

            let decoded = match decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(e) => return Err(ConversionError::ConversionFailed(e.to_string())),
            };

            let spec = *decoded.spec();
            let frames = decoded.frames() as u64;
            let capacity = decoded.capacity() as u64;

            let mut buf = SampleBuffer::<i16>::new(capacity, spec);
            buf.copy_interleaved_ref(decoded);
            all_samples.extend_from_slice(buf.samples());

            decoded_frames += frames;

            if let Some(total) = n_frames {
                if total > 0 {
                    on_progress((decoded_frames as f32 / total as f32).min(0.9));
                }
            }
        }

        if all_samples.is_empty() {
            return Err(ConversionError::ConversionFailed(
                "No audio samples decoded".to_string(),
            ));
        }

        let wav_spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(output, wav_spec)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        for sample in &all_samples {
            writer
                .write_sample(*sample)
                .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        }

        writer
            .finalize()
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        on_progress(1.0);

        Ok(())
    }
}
