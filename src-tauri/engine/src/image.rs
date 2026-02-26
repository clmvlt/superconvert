use std::fs;
use std::path::Path;

use image::codecs::avif::AvifEncoder;
use image::codecs::ico::IcoEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageEncoder};

use crate::error::ConversionError;
use crate::traits::Converter;
use crate::types::{ConversionOptions, DocumentFormat, ImageFormat, OutputFormat};

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

        match ext.as_str() {
            "svg" => self.load_svg(input),
            "psd" => self.load_psd(input),
            #[cfg(feature = "heif")]
            "heif" | "heic" => self.load_heif(input),
            #[cfg(feature = "raw-photos")]
            "cr2" | "nef" | "arw" | "dng" | "orf" | "rw2" => self.load_raw(input),
            #[cfg(feature = "jxl")]
            "jxl" => self.load_jxl(input),
            _ => image::open(input).map_err(|e| ConversionError::ReadError(e.to_string())),
        }
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

    fn load_psd(&self, input: &Path) -> Result<DynamicImage, ConversionError> {
        let psd_bytes =
            fs::read(input).map_err(|e| ConversionError::ReadError(e.to_string()))?;

        let psd_file = psd::Psd::from_bytes(&psd_bytes)
            .map_err(|e| ConversionError::ReadError(format!("PSD parse error: {:?}", e)))?;

        let width = psd_file.width();
        let height = psd_file.height();
        let rgba = psd_file.rgba();

        let img_buf = image::RgbaImage::from_raw(width, height, rgba).ok_or_else(|| {
            ConversionError::ConversionFailed("Failed to create image from PSD".to_string())
        })?;

        Ok(DynamicImage::ImageRgba8(img_buf))
    }

    #[cfg(feature = "heif")]
    fn load_heif(&self, input: &Path) -> Result<DynamicImage, ConversionError> {
        let ctx = libheif_rs::HeifContext::read_from_file(input.to_str().unwrap_or(""))
            .map_err(|e| ConversionError::ReadError(format!("HEIF error: {}", e)))?;

        let handle = ctx
            .primary_image_handle()
            .map_err(|e| ConversionError::ReadError(format!("HEIF handle error: {}", e)))?;

        let heif_img = handle
            .decode(libheif_rs::ColorSpace::Rgb(libheif_rs::RgbChroma::Rgba), None)
            .map_err(|e| ConversionError::ReadError(format!("HEIF decode error: {}", e)))?;

        let width = heif_img.width(libheif_rs::Channel::Interleaved)
            .map_err(|e| ConversionError::ReadError(format!("HEIF width error: {}", e)))?;
        let height = heif_img.height(libheif_rs::Channel::Interleaved)
            .map_err(|e| ConversionError::ReadError(format!("HEIF height error: {}", e)))?;

        let plane = heif_img.plane(libheif_rs::Channel::Interleaved)
            .map_err(|e| ConversionError::ReadError(format!("HEIF plane error: {}", e)))?;

        let stride = plane.stride;
        let data = plane.data;
        let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height as usize {
            let row_start = y * stride;
            rgba_data.extend_from_slice(&data[row_start..row_start + (width as usize * 4)]);
        }

        let img_buf = image::RgbaImage::from_raw(width, height, rgba_data).ok_or_else(|| {
            ConversionError::ConversionFailed("Failed to create image from HEIF".to_string())
        })?;

        Ok(DynamicImage::ImageRgba8(img_buf))
    }

    #[cfg(feature = "raw-photos")]
    fn load_raw(&self, input: &Path) -> Result<DynamicImage, ConversionError> {
        let raw_image = rawloader::decode_file(input)
            .map_err(|e| ConversionError::ReadError(format!("Raw decode error: {}", e)))?;

        let pipeline = imagepipe::Pipeline::new_from_rawimage(&raw_image);
        let result = pipeline.output_8bit(None)
            .map_err(|e| ConversionError::ConversionFailed(format!("Raw pipeline error: {}", e)))?;

        let img_buf =
            image::RgbImage::from_raw(result.width as u32, result.height as u32, result.data)
                .ok_or_else(|| {
                    ConversionError::ConversionFailed(
                        "Failed to create image from raw photo".to_string(),
                    )
                })?;

        Ok(DynamicImage::ImageRgb8(img_buf))
    }

    #[cfg(feature = "jxl")]
    fn load_jxl(&self, input: &Path) -> Result<DynamicImage, ConversionError> {
        let data = fs::read(input).map_err(|e| ConversionError::ReadError(e.to_string()))?;
        let reader = jxl_oxide::JxlImage::builder()
            .read(&data)
            .map_err(|e| ConversionError::ReadError(format!("JPEG XL error: {}", e)))?;

        let render = reader
            .render_frame(0)
            .map_err(|e| ConversionError::ConversionFailed(format!("JXL render error: {}", e)))?;

        let fb = render.image();
        let width = fb.width() as u32;
        let height = fb.height() as u32;

        let buf = fb.buf();
        let channels = fb.channels();

        let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height as usize {
            for x in 0..width as usize {
                let idx = y * width as usize + x;
                let r = (buf[idx] * 255.0).clamp(0.0, 255.0) as u8;
                let g = if channels > 1 {
                    (buf[width as usize * height as usize + idx] * 255.0).clamp(0.0, 255.0) as u8
                } else {
                    r
                };
                let b = if channels > 2 {
                    (buf[2 * width as usize * height as usize + idx] * 255.0).clamp(0.0, 255.0)
                        as u8
                } else {
                    r
                };
                let a = if channels > 3 {
                    (buf[3 * width as usize * height as usize + idx] * 255.0).clamp(0.0, 255.0)
                        as u8
                } else {
                    255u8
                };
                rgba_data.push(r);
                rgba_data.push(g);
                rgba_data.push(b);
                rgba_data.push(a);
            }
        }

        let img_buf = image::RgbaImage::from_raw(width, height, rgba_data).ok_or_else(|| {
            ConversionError::ConversionFailed("Failed to create image from JXL".to_string())
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
            OutputFormat::Image(ImageFormat::Tga) => {
                img.save(output)?;
            }
            OutputFormat::Image(ImageFormat::Qoi) => {
                img.save(output)?;
            }
            OutputFormat::Image(ImageFormat::Hdr) => {
                let rgb32f = img.to_rgb32f();
                let file = fs::File::create(output)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
                let writer = std::io::BufWriter::new(file);
                let encoder = image::codecs::hdr::HdrEncoder::new(writer);
                let pixels: Vec<image::Rgb<f32>> = rgb32f
                    .pixels()
                    .map(|p| image::Rgb([p[0], p[1], p[2]]))
                    .collect();
                encoder
                    .encode(&pixels, rgb32f.width() as usize, rgb32f.height() as usize)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            OutputFormat::Image(ImageFormat::Ppm) => {
                let rgb = img.to_rgb8();
                let file = fs::File::create(output)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
                let writer = std::io::BufWriter::new(file);
                let encoder = image::codecs::pnm::PnmEncoder::new(writer)
                    .with_subtype(image::codecs::pnm::PnmSubtype::Pixmap(
                        image::codecs::pnm::SampleEncoding::Binary,
                    ));
                encoder
                    .write_image(
                        rgb.as_raw(),
                        rgb.width(),
                        rgb.height(),
                        image::ExtendedColorType::Rgb8,
                    )
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            OutputFormat::Image(ImageFormat::Exr) => {
                let rgb32f = img.to_rgb32f();
                image::save_buffer(
                    output,
                    &rgb32f.as_raw().iter().flat_map(|&f| f.to_ne_bytes()).collect::<Vec<u8>>(),
                    rgb32f.width(),
                    rgb32f.height(),
                    image::ExtendedColorType::Rgb32F,
                )
                .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            OutputFormat::Image(ImageFormat::Svg) => {
                return Err(ConversionError::UnsupportedOutputFormat(
                    "SVG output is not supported".to_string(),
                ));
            }
            OutputFormat::Image(ImageFormat::Dds) => {
                return Err(ConversionError::UnsupportedOutputFormat(
                    "DDS output is not supported".to_string(),
                ));
            }
            OutputFormat::Image(ImageFormat::Psd) => {
                return Err(ConversionError::UnsupportedOutputFormat(
                    "PSD output is not supported".to_string(),
                ));
            }
            OutputFormat::Image(ImageFormat::Heif) => {
                return Err(ConversionError::UnsupportedOutputFormat(
                    "HEIF output is not supported".to_string(),
                ));
            }
            OutputFormat::Image(ImageFormat::RawPhoto) => {
                return Err(ConversionError::UnsupportedOutputFormat(
                    "Raw photo output is not supported".to_string(),
                ));
            }
            OutputFormat::Image(ImageFormat::Jxl) => {
                return Err(ConversionError::UnsupportedOutputFormat(
                    "JPEG XL output is not yet supported".to_string(),
                ));
            }
            OutputFormat::Image(ImageFormat::Jp2) => {
                return Err(ConversionError::UnsupportedOutputFormat(
                    "JPEG 2000 output is not supported".to_string(),
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

                let mut doc = printpdf::PdfDocument::new("SuperConvert");
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
            "tga", "dds", "qoi", "hdr", "ppm", "pgm", "pbm", "exr", "psd",
        ]
    }

    fn supported_output_formats(&self) -> &[&str] {
        &[
            "png", "jpg", "gif", "bmp", "ico", "tiff", "webp", "avif", "tga", "qoi", "hdr",
            "ppm", "exr", "pdf",
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

        let img = self.load_image(input)?;
        on_progress(0.4);

        self.save_image(&img, output, options)?;
        on_progress(1.0);

        Ok(())
    }
}
