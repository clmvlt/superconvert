use std::fs;
use std::path::Path;

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, GrayImage, ImageEncoder, RgbImage};
use lopdf::{Document, Object, ObjectId};

use super::error::ConversionError;
use super::traits::Converter;
use super::types::ConversionOptions;

pub struct PdfConverter;

impl PdfConverter {
    pub fn new() -> Self {
        Self
    }

    fn resolve_dict<'a>(
        &self,
        doc: &'a Document,
        obj: &'a Object,
    ) -> Option<&'a lopdf::Dictionary> {
        match obj {
            Object::Dictionary(d) => Some(d),
            Object::Reference(id) => match doc.get_object(*id) {
                Ok(Object::Dictionary(d)) => Some(d),
                _ => None,
            },
            _ => None,
        }
    }

    fn extract_images_from_page(
        &self,
        doc: &Document,
        page_id: ObjectId,
    ) -> Vec<DynamicImage> {
        let mut images = Vec::new();

        let page_obj = match doc.get_object(page_id) {
            Ok(o) => o,
            Err(_) => return images,
        };

        let page_dict = match page_obj {
            Object::Dictionary(d) => d,
            _ => return images,
        };

        let resources = match page_dict.get(b"Resources") {
            Ok(r) => r,
            Err(_) => return images,
        };

        let resources_dict = match self.resolve_dict(doc, resources) {
            Some(d) => d,
            None => return images,
        };

        let xobjects = match resources_dict.get(b"XObject") {
            Ok(x) => x,
            Err(_) => return images,
        };

        let xobjects_dict = match self.resolve_dict(doc, xobjects) {
            Some(d) => d,
            None => return images,
        };

        for (_name, obj) in xobjects_dict.iter() {
            let resolved = match obj {
                Object::Reference(id) => match doc.get_object(*id) {
                    Ok(o) => o,
                    Err(_) => continue,
                },
                other => other,
            };

            if let Object::Stream(stream) = resolved {
                if let Some(img) = self.decode_image_stream(stream) {
                    images.push(img);
                }
            }
        }

        images
    }

    fn decode_image_stream(&self, stream: &lopdf::Stream) -> Option<DynamicImage> {
        let subtype = stream.dict.get(b"Subtype").ok()?;
        match subtype {
            Object::Name(name) if name == b"Image" => {}
            _ => return None,
        }

        let filter = self.get_filter_name(&stream.dict);

        match filter.as_deref() {
            Some(b"DCTDecode") => image::load_from_memory(&stream.content).ok(),
            Some(b"FlateDecode") => self.decode_flate_image(stream),
            _ => image::load_from_memory(&stream.content).ok(),
        }
    }

    fn get_filter_name(&self, dict: &lopdf::Dictionary) -> Option<Vec<u8>> {
        let filter = dict.get(b"Filter").ok()?;
        match filter {
            Object::Name(n) => Some(n.clone()),
            Object::Array(arr) => match arr.last() {
                Some(Object::Name(n)) => Some(n.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    fn get_integer(&self, dict: &lopdf::Dictionary, key: &[u8]) -> Option<i64> {
        match dict.get(key).ok()? {
            Object::Integer(i) => Some(*i),
            _ => None,
        }
    }

    fn decode_flate_image(&self, stream: &lopdf::Stream) -> Option<DynamicImage> {
        let width = self.get_integer(&stream.dict, b"Width")? as u32;
        let height = self.get_integer(&stream.dict, b"Height")? as u32;

        let decompressed = stream.decompressed_content().ok()?;

        let cs_name = match stream.dict.get(b"ColorSpace").ok() {
            Some(Object::Name(n)) => n.clone(),
            _ => b"DeviceRGB".to_vec(),
        };

        let channels: u32 = match cs_name.as_slice() {
            b"DeviceRGB" => 3,
            b"DeviceGray" => 1,
            _ => return None,
        };

        let expected_raw = (width * height * channels) as usize;
        let expected_with_predictor = ((width * channels + 1) * height) as usize;

        let raw_pixels = if decompressed.len() == expected_raw {
            decompressed
        } else if decompressed.len() == expected_with_predictor {
            self.remove_png_predictor(&decompressed, width, height, channels)?
        } else {
            return None;
        };

        match channels {
            3 => RgbImage::from_raw(width, height, raw_pixels).map(DynamicImage::ImageRgb8),
            1 => GrayImage::from_raw(width, height, raw_pixels).map(DynamicImage::ImageLuma8),
            _ => None,
        }
    }

    fn remove_png_predictor(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        channels: u32,
    ) -> Option<Vec<u8>> {
        let row_bytes = (width * channels) as usize;
        let stride = row_bytes + 1;
        let mut output = vec![0u8; (width * height * channels) as usize];
        let mut prev_row = vec![0u8; row_bytes];
        let bpp = channels as usize;

        for y in 0..height as usize {
            let row_start = y * stride;
            if row_start + stride > data.len() {
                return None;
            }
            let filter_type = data[row_start];
            let row_data = &data[row_start + 1..row_start + stride];
            let out_start = y * row_bytes;

            match filter_type {
                0 => {
                    output[out_start..out_start + row_bytes].copy_from_slice(row_data);
                }
                1 => {
                    for x in 0..row_bytes {
                        let left = if x >= bpp {
                            output[out_start + x - bpp]
                        } else {
                            0
                        };
                        output[out_start + x] = row_data[x].wrapping_add(left);
                    }
                }
                2 => {
                    for x in 0..row_bytes {
                        output[out_start + x] = row_data[x].wrapping_add(prev_row[x]);
                    }
                }
                3 => {
                    for x in 0..row_bytes {
                        let left = if x >= bpp {
                            output[out_start + x - bpp] as u16
                        } else {
                            0
                        };
                        let up = prev_row[x] as u16;
                        output[out_start + x] =
                            row_data[x].wrapping_add(((left + up) / 2) as u8);
                    }
                }
                4 => {
                    for x in 0..row_bytes {
                        let left = if x >= bpp {
                            output[out_start + x - bpp]
                        } else {
                            0
                        };
                        let up = prev_row[x];
                        let upper_left = if x >= bpp { prev_row[x - bpp] } else { 0 };
                        output[out_start + x] =
                            row_data[x].wrapping_add(paeth_predictor(left, up, upper_left));
                    }
                }
                _ => {
                    output[out_start..out_start + row_bytes].copy_from_slice(row_data);
                }
            }

            prev_row.copy_from_slice(&output[out_start..out_start + row_bytes]);
        }

        Some(output)
    }

    fn save_extracted_image(
        &self,
        img: &DynamicImage,
        output: &Path,
        options: &ConversionOptions,
    ) -> Result<(), ConversionError> {
        let ext = output
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "png" => {
                img.save(output)?;
            }
            "jpg" | "jpeg" => {
                let quality = options.quality.unwrap_or(85);
                let rgb = img.to_rgb8();
                let file = fs::File::create(output)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
                let mut writer = std::io::BufWriter::new(file);
                let encoder = JpegEncoder::new_with_quality(&mut writer, quality);
                encoder
                    .write_image(
                        rgb.as_raw(),
                        rgb.width(),
                        rgb.height(),
                        image::ExtendedColorType::Rgb8,
                    )
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            _ => {
                return Err(ConversionError::UnsupportedOutputFormat(ext));
            }
        }
        Ok(())
    }
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let a = a as i32;
    let b = b as i32;
    let c = c as i32;
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

impl Converter for PdfConverter {
    fn supported_input_formats(&self) -> &[&str] {
        &["pdf"]
    }

    fn supported_output_formats(&self) -> &[&str] {
        &["png", "jpg"]
    }

    fn convert(
        &self,
        input: &Path,
        output: &Path,
        options: &ConversionOptions,
        on_progress: Box<dyn Fn(f32) + Send>,
    ) -> Result<(), ConversionError> {
        on_progress(0.0);

        let doc = Document::load(input)
            .map_err(|e| ConversionError::ReadError(e.to_string()))?;

        let pages = doc.get_pages();
        let total_pages = pages.len();

        if total_pages == 0 {
            return Err(ConversionError::ConversionFailed(
                "PDF contains no pages".to_string(),
            ));
        }

        let ext = output
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let stem = output
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let parent = output.parent().unwrap_or(Path::new("."));

        let mut extracted_count = 0u32;

        for (i, (_page_num, page_id)) in pages.iter().enumerate() {
            let images = self.extract_images_from_page(&doc, *page_id);

            if let Some(largest) = images
                .into_iter()
                .max_by_key(|img| (img.width() as u64) * (img.height() as u64))
            {
                let output_path = if total_pages == 1 {
                    output.to_path_buf()
                } else {
                    parent.join(format!("{}_{:03}.{}", stem, i + 1, ext))
                };

                self.save_extracted_image(&largest, &output_path, options)?;
                extracted_count += 1;
            }

            on_progress((i + 1) as f32 / total_pages as f32);
        }

        if extracted_count == 0 {
            return Err(ConversionError::ConversionFailed(
                "No extractable images found in PDF".to_string(),
            ));
        }

        Ok(())
    }
}
