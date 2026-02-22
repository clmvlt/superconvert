use std::fs;
use std::path::Path;

use image::codecs::avif::AvifEncoder;
use image::codecs::ico::IcoEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageEncoder};

use super::error::ConversionError;
use super::traits::Converter;
use super::types::{ConversionOptions, DocumentFormat, ImageFormat, OutputFormat};

pub struct ImageConverter;

impl ImageConverter {
    pub fn new() -> Self {
        Self
    }

    fn load_image(&self, input: &Path) -> Result<DynamicImage, ConversionError> {
        let ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext == "svg" {
            return self.load_svg(input);
        }

        image::open(input).map_err(|e| ConversionError::ReadError(e.to_string()))
    }

    fn load_svg(&self, input: &Path) -> Result<DynamicImage, ConversionError> {
        let svg_data =
            fs::read(input).map_err(|e| ConversionError::ReadError(e.to_string()))?;

        let options = usvg::Options::default();
        let tree = usvg::Tree::from_data(&svg_data, &options)
            .map_err(|e| ConversionError::ReadError(format!("SVG parse error: {}", e)))?;

        let size = tree.size();
        let width = size.width().ceil() as u32;
        let height = size.height().ceil() as u32;

        if width == 0 || height == 0 {
            return Err(ConversionError::ConversionFailed(
                "SVG has zero dimensions".to_string(),
            ));
        }

        let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).ok_or_else(|| {
            ConversionError::ConversionFailed("Failed to create pixmap".to_string())
        })?;

        resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

        let mut pixels = pixmap.take();
        for chunk in pixels.chunks_exact_mut(4) {
            let a = chunk[3] as f32 / 255.0;
            if a > 0.0 && a < 1.0 {
                chunk[0] = (chunk[0] as f32 / a).min(255.0) as u8;
                chunk[1] = (chunk[1] as f32 / a).min(255.0) as u8;
                chunk[2] = (chunk[2] as f32 / a).min(255.0) as u8;
            }
        }

        let img_buf = image::RgbaImage::from_raw(width, height, pixels).ok_or_else(|| {
            ConversionError::ConversionFailed("Failed to create image from SVG pixels".to_string())
        })?;

        Ok(DynamicImage::ImageRgba8(img_buf))
    }

    fn save_image(
        &self,
        img: &DynamicImage,
        output: &Path,
        options: &ConversionOptions,
    ) -> Result<(), ConversionError> {
        match &options.output_format {
            OutputFormat::Image(ImageFormat::Png) => {
                img.save(output)?;
            }
            OutputFormat::Image(ImageFormat::Jpg) => {
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
            OutputFormat::Image(ImageFormat::Gif) => {
                img.save(output)?;
            }
            OutputFormat::Image(ImageFormat::Bmp) => {
                img.save(output)?;
            }
            OutputFormat::Image(ImageFormat::Tiff) => {
                img.save(output)?;
            }
            OutputFormat::Image(ImageFormat::Webp) => {
                let rgba = img.to_rgba8();
                let file = fs::File::create(output)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
                let writer = std::io::BufWriter::new(file);
                let encoder = image::codecs::webp::WebPEncoder::new_lossless(writer);
                encoder
                    .write_image(
                        rgba.as_raw(),
                        rgba.width(),
                        rgba.height(),
                        image::ExtendedColorType::Rgba8,
                    )
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            OutputFormat::Image(ImageFormat::Avif) => {
                let quality = options.quality.unwrap_or(60);
                let rgba = img.to_rgba8();
                let file = fs::File::create(output)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
                let mut writer = std::io::BufWriter::new(file);
                let encoder = AvifEncoder::new_with_speed_quality(&mut writer, 6, quality);
                encoder
                    .write_image(
                        rgba.as_raw(),
                        rgba.width(),
                        rgba.height(),
                        image::ExtendedColorType::Rgba8,
                    )
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            OutputFormat::Image(ImageFormat::Ico) => {
                let resized = if img.width() > 256 || img.height() > 256 {
                    img.resize(256, 256, image::imageops::FilterType::Lanczos3)
                } else {
                    img.clone()
                };
                let rgba = resized.to_rgba8();
                let file = fs::File::create(output)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
                let mut writer = std::io::BufWriter::new(file);
                let encoder = IcoEncoder::new(&mut writer);
                encoder
                    .write_image(
                        rgba.as_raw(),
                        rgba.width(),
                        rgba.height(),
                        image::ExtendedColorType::Rgba8,
                    )
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            OutputFormat::Image(ImageFormat::Svg) => {
                return Err(ConversionError::UnsupportedOutputFormat(
                    "SVG output is not supported".to_string(),
                ));
            }
            OutputFormat::Image(ImageFormat::Tga) => {
                img.save(output)?;
            }
            OutputFormat::Image(ImageFormat::Qoi) => {
                img.save(output)?;
            }
            OutputFormat::Image(ImageFormat::Dds) => {
                return Err(ConversionError::UnsupportedOutputFormat(
                    "DDS output is not supported".to_string(),
                ));
            }
            OutputFormat::Document(DocumentFormat::Pdf) => {
                let rgb = img.to_rgb8();
                let (w, h) = (rgb.width() as usize, rgb.height() as usize);
                let px_to_mm = 25.4_f32 / 72.0;
                let page_w = w as f32 * px_to_mm;
                let page_h = h as f32 * px_to_mm;

                let raw_image = printpdf::RawImage {
                    pixels: printpdf::RawImageData::U8(rgb.into_raw()),
                    width: w,
                    height: h,
                    data_format: printpdf::RawImageFormat::RGB8,
                    tag: Vec::new(),
                };

                let mut doc = printpdf::PdfDocument::new("Convertor");
                let image_id = doc.add_image(&raw_image);

                let page = printpdf::PdfPage::new(
                    printpdf::Mm(page_w),
                    printpdf::Mm(page_h),
                    vec![printpdf::Op::UseXobject {
                        id: image_id,
                        transform: printpdf::XObjectTransform {
                            dpi: Some(72.0),
                            ..Default::default()
                        },
                    }],
                );

                doc.with_pages(vec![page]);

                let mut warnings = Vec::new();
                let file = fs::File::create(output)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
                let mut writer = std::io::BufWriter::new(file);
                doc.save_writer(&mut writer, &printpdf::PdfSaveOptions::default(), &mut warnings);
            }
            _ => {
                return Err(ConversionError::UnsupportedOutputFormat(
                    options.output_format.extension().to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl Converter for ImageConverter {
    fn supported_input_formats(&self) -> &[&str] {
        &[
            "png", "jpg", "jpeg", "gif", "bmp", "ico", "tif", "tiff", "webp", "avif", "svg",
            "tga", "dds", "qoi",
        ]
    }

    fn supported_output_formats(&self) -> &[&str] {
        &["png", "jpg", "gif", "bmp", "ico", "tiff", "webp", "avif", "tga", "qoi", "pdf"]
    }

    fn convert(
        &self,
        input: &Path,
        output: &Path,
        options: &ConversionOptions,
        on_progress: Box<dyn Fn(f32) + Send>,
    ) -> Result<(), ConversionError> {
        on_progress(0.0);

        let img = self.load_image(input)?;
        on_progress(0.4);

        self.save_image(&img, output, options)?;
        on_progress(1.0);

        Ok(())
    }
}
