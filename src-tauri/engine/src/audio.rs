use std::fs::File;
use std::io::Write;
use std::path::Path;

use hound::{SampleFormat, WavSpec, WavWriter};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::ConversionError;
use crate::traits::Converter;
use crate::types::{AudioFormat, ConversionOptions, OutputFormat};

struct DecodedAudio {
    samples: Vec<i16>,
    channels: u16,
    sample_rate: u32,
}

pub struct AudioConverter;

impl AudioConverter {
    pub fn new() -> Self {
        Self
    }

    fn decode_audio(
        &self,
        input: &Path,
        on_progress: &dyn Fn(f32),
    ) -> Result<DecodedAudio, ConversionError> {
        let input_ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

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

        Ok(DecodedAudio {
            samples: all_samples,
            channels,
            sample_rate,
        })
    }

    fn write_wav(&self, audio: &DecodedAudio, output: &Path) -> Result<(), ConversionError> {
        let wav_spec = WavSpec {
            channels: audio.channels,
            sample_rate: audio.sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(output, wav_spec)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        for sample in &audio.samples {
            writer
                .write_sample(*sample)
                .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        }

        writer
            .finalize()
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        Ok(())
    }

    fn write_aiff(&self, audio: &DecodedAudio, output: &Path) -> Result<(), ConversionError> {
        let mut file =
            File::create(output).map_err(|e| ConversionError::WriteError(e.to_string()))?;

        let num_frames = audio.samples.len() as u32 / audio.channels as u32;
        let bits_per_sample: u16 = 16;
        let sample_size = (bits_per_sample / 8) as u32;
        let sound_data_size = audio.samples.len() as u32 * sample_size;
        let comm_chunk_size: u32 = 18;
        let ssnd_chunk_size: u32 = 8 + sound_data_size;
        let form_size: u32 = 4 + 8 + comm_chunk_size + 8 + ssnd_chunk_size;

        file.write_all(b"FORM")?;
        file.write_all(&form_size.to_be_bytes())?;
        file.write_all(b"AIFF")?;

        file.write_all(b"COMM")?;
        file.write_all(&comm_chunk_size.to_be_bytes())?;
        file.write_all(&audio.channels.to_be_bytes())?;
        file.write_all(&num_frames.to_be_bytes())?;
        file.write_all(&bits_per_sample.to_be_bytes())?;
        file.write_all(&sample_rate_to_ieee_extended(audio.sample_rate))?;

        file.write_all(b"SSND")?;
        file.write_all(&ssnd_chunk_size.to_be_bytes())?;
        file.write_all(&0u32.to_be_bytes())?;
        file.write_all(&0u32.to_be_bytes())?;
        for sample in &audio.samples {
            file.write_all(&sample.to_be_bytes())?;
        }

        Ok(())
    }

    fn write_flac(&self, audio: &DecodedAudio, output: &Path) -> Result<(), ConversionError> {
        let mut file =
            File::create(output).map_err(|e| ConversionError::WriteError(e.to_string()))?;

        let channels = audio.channels as u8;
        let bits_per_sample: u8 = 16;
        let total_samples = audio.samples.len() as u64 / channels as u64;
        let sample_rate = audio.sample_rate;

        file.write_all(b"fLaC")?;

        let mut streaminfo = Vec::new();
        let block_size: u16 = 4096;
        streaminfo.extend_from_slice(&block_size.to_be_bytes());
        streaminfo.extend_from_slice(&block_size.to_be_bytes());
        streaminfo.extend_from_slice(&[0u8; 3]);
        streaminfo.extend_from_slice(&[0u8; 3]);

        let sr_chan_bps: u32 = ((sample_rate & 0xFFFFF) << 12)
            | (((channels as u32 - 1) & 0x7) << 9)
            | (((bits_per_sample as u32 - 1) & 0x1F) << 4)
            | ((total_samples >> 32) as u32 & 0xF);
        streaminfo.extend_from_slice(&sr_chan_bps.to_be_bytes());
        streaminfo.extend_from_slice(&(total_samples as u32).to_be_bytes());

        streaminfo.extend_from_slice(&[0u8; 16]);

        let header_byte: u8 = 0x80;
        file.write_all(&[header_byte])?;
        let si_len = streaminfo.len() as u32;
        file.write_all(&[(si_len >> 16) as u8, (si_len >> 8) as u8, si_len as u8])?;
        file.write_all(&streaminfo)?;

        let frames_per_block = block_size as usize;
        let total_frames = total_samples as usize;
        let mut frame_number: u32 = 0;
        let mut offset = 0usize;

        while offset < total_frames {
            let this_block = (total_frames - offset).min(frames_per_block);
            let mut frame_data = Vec::new();

            frame_data.extend_from_slice(&[0xFF, 0xF8]);

            let bs_code: u8 = if this_block == frames_per_block { 0x0C } else { 0x06 };
            let sr_code: u8 = match sample_rate {
                8000 => 0x04,
                16000 => 0x05,
                22050 => 0x06,
                24000 => 0x07,
                32000 => 0x08,
                44100 => 0x09,
                48000 => 0x0A,
                96000 => 0x0B,
                _ => 0x0C,
            };
            frame_data.push((bs_code << 4) | sr_code);

            let ch_assign: u8 = (channels as u8 - 1) << 4;
            let bps_code: u8 = 0x04;
            frame_data.push(ch_assign | (bps_code << 1));

            if frame_number < 0x80 {
                frame_data.push(frame_number as u8);
            } else {
                let mut utf8 = Vec::new();
                let val = frame_number;
                if val < 0x800 {
                    utf8.push(0xC0 | (val >> 6) as u8);
                    utf8.push(0x80 | (val & 0x3F) as u8);
                } else if val < 0x10000 {
                    utf8.push(0xE0 | (val >> 12) as u8);
                    utf8.push(0x80 | ((val >> 6) & 0x3F) as u8);
                    utf8.push(0x80 | (val & 0x3F) as u8);
                } else {
                    utf8.push(0xF0 | (val >> 18) as u8);
                    utf8.push(0x80 | ((val >> 12) & 0x3F) as u8);
                    utf8.push(0x80 | ((val >> 6) & 0x3F) as u8);
                    utf8.push(0x80 | (val & 0x3F) as u8);
                }
                frame_data.extend_from_slice(&utf8);
            }

            if bs_code == 0x06 {
                frame_data.push((this_block - 1) as u8);
            }

            if sr_code == 0x0C {
                frame_data.extend_from_slice(&(sample_rate as u16).to_be_bytes());
            }

            let crc8 = compute_crc8(&frame_data);
            frame_data.push(crc8);

            for ch in 0..channels as usize {
                frame_data.push(0x00);

                for i in 0..this_block {
                    let sample_idx = (offset + i) * channels as usize + ch;
                    let sample = if sample_idx < audio.samples.len() {
                        audio.samples[sample_idx]
                    } else {
                        0
                    };
                    frame_data.extend_from_slice(&sample.to_be_bytes());
                }
            }

            let padding_bits = (8 - (frame_data.len() * 8) % 8) % 8;
            if padding_bits > 0 {
                frame_data.push(0);
            }

            let crc16 = compute_crc16(&frame_data);
            frame_data.extend_from_slice(&crc16.to_be_bytes());

            file.write_all(&frame_data)?;

            offset += this_block;
            frame_number += 1;
        }

        Ok(())
    }

    #[cfg(feature = "mp3-encode")]
    fn write_mp3(
        &self,
        audio: &DecodedAudio,
        output: &Path,
        quality: Option<u8>,
    ) -> Result<(), ConversionError> {
        use mp3lame_encoder::{Builder, InterleavedPcm};

        let bitrate = match quality.unwrap_or(75) {
            0..=25 => mp3lame_encoder::Bitrate::Kbps96,
            26..=50 => mp3lame_encoder::Bitrate::Kbps128,
            51..=75 => mp3lame_encoder::Bitrate::Kbps192,
            76..=90 => mp3lame_encoder::Bitrate::Kbps256,
            _ => mp3lame_encoder::Bitrate::Kbps320,
        };

        let mut encoder = Builder::new().expect("Failed to create LAME encoder");
        encoder.set_num_channels(audio.channels as u8).unwrap();
        encoder.set_sample_rate(audio.sample_rate).unwrap();
        encoder.set_brate(bitrate).unwrap();
        encoder.set_quality(mp3lame_encoder::Quality::Best).unwrap();

        let mut encoder = encoder.build().unwrap();

        let input = InterleavedPcm(&audio.samples);
        let mut mp3_data = Vec::new();
        let encoded = encoder
            .encode(input)
            .map_err(|e| ConversionError::ConversionFailed(format!("MP3 encode error: {:?}", e)))?;
        mp3_data.extend_from_slice(&encoded);

        let flushed = encoder
            .flush::<Vec<u8>>()
            .map_err(|e| ConversionError::ConversionFailed(format!("MP3 flush error: {:?}", e)))?;
        mp3_data.extend_from_slice(&flushed);

        std::fs::write(output, &mp3_data)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        Ok(())
    }

    #[cfg(feature = "vorbis-encode")]
    fn write_ogg(
        &self,
        audio: &DecodedAudio,
        output: &Path,
        _quality: Option<u8>,
    ) -> Result<(), ConversionError> {
        let _ = (audio, output, _quality);
        Err(ConversionError::ConversionFailed(
            "OGG Vorbis encoding is not yet fully implemented".to_string(),
        ))
    }

    #[cfg(feature = "opus-encode")]
    fn write_opus(
        &self,
        audio: &DecodedAudio,
        output: &Path,
        _quality: Option<u8>,
    ) -> Result<(), ConversionError> {
        let _ = (audio, output, _quality);
        Err(ConversionError::ConversionFailed(
            "Opus encoding is not yet fully implemented".to_string(),
        ))
    }
}

fn sample_rate_to_ieee_extended(sample_rate: u32) -> [u8; 10] {
    let mut result = [0u8; 10];
    let sr = sample_rate as f64;

    if sr == 0.0 {
        return result;
    }

    let mut exponent: i32 = 16383 + 31;
    let mut mantissa = sr as u64;

    while mantissa < (1u64 << 63) {
        mantissa <<= 1;
        exponent -= 1;
    }

    let exp = exponent as u16;
    result[0] = (exp >> 8) as u8;
    result[1] = exp as u8;
    result[2] = (mantissa >> 56) as u8;
    result[3] = (mantissa >> 48) as u8;
    result[4] = (mantissa >> 40) as u8;
    result[5] = (mantissa >> 32) as u8;
    result[6] = (mantissa >> 24) as u8;
    result[7] = (mantissa >> 16) as u8;
    result[8] = (mantissa >> 8) as u8;
    result[9] = mantissa as u8;

    result
}

fn compute_crc8(data: &[u8]) -> u8 {
    let mut crc: u8 = 0;
    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if crc & 0x80 != 0 {
                crc = (crc << 1) ^ 0x07;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

fn compute_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x8005;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

impl Converter for AudioConverter {
    fn supported_input_formats(&self) -> &[&str] {
        &[
            "mp3", "wav", "flac", "ogg", "aac", "aiff", "aif", "m4a", "alac", "opus", "wma",
            "ac3", "dts",
        ]
    }

    fn supported_output_formats(&self) -> &[&str] {
        if cfg!(feature = "mp3-encode") {
            &["wav", "flac", "aiff", "mp3"]
        } else {
            &["wav", "flac", "aiff"]
        }
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

        if !input.exists() {
            return Err(ConversionError::FileNotFound(input.display().to_string()));
        }

        let audio = self.decode_audio(input, &*on_progress)?;
        on_progress(0.7);

        match &options.output_format {
            OutputFormat::Audio(AudioFormat::Wav) => {
                self.write_wav(&audio, output)?;
            }
            OutputFormat::Audio(AudioFormat::Flac) => {
                self.write_flac(&audio, output)?;
            }
            OutputFormat::Audio(AudioFormat::Aiff) => {
                self.write_aiff(&audio, output)?;
            }
            #[cfg(feature = "mp3-encode")]
            OutputFormat::Audio(AudioFormat::Mp3) => {
                self.write_mp3(&audio, output, options.quality)?;
            }
            #[cfg(feature = "vorbis-encode")]
            OutputFormat::Audio(AudioFormat::Ogg) => {
                self.write_ogg(&audio, output, options.quality)?;
            }
            #[cfg(feature = "opus-encode")]
            OutputFormat::Audio(AudioFormat::Opus) => {
                self.write_opus(&audio, output, options.quality)?;
            }
            _ => {
                return Err(ConversionError::UnsupportedOutputFormat(
                    options.output_format.extension().to_string(),
                ));
            }
        }

        on_progress(1.0);
        Ok(())
    }
}
