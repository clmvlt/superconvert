use std::fs;
use std::io::{Read as IoRead, Write as IoWrite};
use std::path::Path;

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, GrayImage, ImageEncoder, RgbImage};
use lopdf::{Document, Object, ObjectId};
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::error::ConversionError;
use crate::traits::Converter;
use crate::types::ConversionOptions;

pub struct DocumentConverter;

impl DocumentConverter {
    pub fn new() -> Self {
        Self
    }

    // ── PDF image extraction (existing logic) ──────────────────────────

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

    fn convert_pdf_to_image(
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

    // ── PDF text extraction ────────────────────────────────────────────

    fn read_pdf_text(&self, path: &Path) -> Result<Vec<String>, ConversionError> {
        let doc = Document::load(path)
            .map_err(|e| ConversionError::ReadError(format!("Failed to load PDF: {}", e)))?;

        let pages = doc.get_pages();
        let page_numbers: Vec<u32> = pages.keys().copied().collect();

        if page_numbers.is_empty() {
            return Ok(vec![]);
        }

        let text = doc.extract_text(&page_numbers)
            .unwrap_or_default();

        let paragraphs: Vec<String> = text
            .lines()
            .map(|l| l.to_string())
            .collect();

        Ok(paragraphs)
    }

    // ── Text document readers (from TextDocConverter) ──────────────────

    fn read_docx_text(&self, path: &Path) -> Result<Vec<String>, ConversionError> {
        let data = fs::read(path)?;
        let docx = docx_rs::read_docx(&data)
            .map_err(|e| ConversionError::ReadError(format!("Failed to parse DOCX: {}", e)))?;

        let mut paragraphs = Vec::new();

        for child in docx.document.children {
            if let docx_rs::DocumentChild::Paragraph(para) = child {
                let mut text = String::new();
                for pc in &para.children {
                    if let docx_rs::ParagraphChild::Run(run) = pc {
                        for rc in &run.children {
                            if let docx_rs::RunChild::Text(t) = rc {
                                text.push_str(&t.text);
                            }
                        }
                    }
                }
                paragraphs.push(text);
            }
        }

        Ok(paragraphs)
    }

    fn read_odt_text(&self, path: &Path) -> Result<Vec<String>, ConversionError> {
        let file = fs::File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut content_xml = String::new();
        {
            let mut content_file = archive
                .by_name("content.xml")
                .map_err(|e| ConversionError::ReadError(format!("No content.xml in ODT: {}", e)))?;
            content_file.read_to_string(&mut content_xml)?;
        }

        let mut paragraphs = Vec::new();
        let mut reader = XmlReader::from_reader(content_xml.as_bytes());
        reader.config_mut().trim_text(true);

        let mut in_text_p = false;
        let mut current_text = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if local == b"p" || local == b"h" {
                        in_text_p = true;
                        current_text.clear();
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_text_p {
                        if let Ok(t) = e.unescape() {
                            current_text.push_str(&t);
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if (local == b"p" || local == b"h") && in_text_p {
                        paragraphs.push(current_text.clone());
                        in_text_p = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ConversionError::ReadError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(paragraphs)
    }

    fn read_txt(&self, path: &Path) -> Result<Vec<String>, ConversionError> {
        let content = fs::read_to_string(path)?;
        Ok(content.lines().map(|l| l.to_string()).collect())
    }

    fn read_rtf_text(&self, path: &Path) -> Result<Vec<String>, ConversionError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConversionError::ReadError(e.to_string()))?;

        let mut paragraphs = Vec::new();
        let mut current = String::new();
        let mut brace_depth: i32 = 0;
        let mut skip_group = false;
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '{' => {
                    brace_depth += 1;
                    if skip_group {
                        continue;
                    }
                }
                '}' => {
                    brace_depth -= 1;
                    if skip_group && brace_depth <= 1 {
                        skip_group = false;
                    }
                    continue;
                }
                '\\' => {
                    if skip_group {
                        while chars.peek().map_or(false, |c| c.is_ascii_alphabetic()) {
                            chars.next();
                        }
                        if chars.peek() == Some(&' ') {
                            chars.next();
                        }
                        continue;
                    }

                    let mut control_word = String::new();
                    while chars.peek().map_or(false, |c| c.is_ascii_alphabetic()) {
                        control_word.push(chars.next().unwrap());
                    }

                    match control_word.as_str() {
                        "par" | "line" => {
                            paragraphs.push(current.clone());
                            current.clear();
                        }
                        "tab" => current.push('\t'),
                        "fonttbl" | "colortbl" | "stylesheet" | "info" | "pict" => {
                            skip_group = true;
                        }
                        _ => {}
                    }

                    if chars.peek() == Some(&' ') {
                        chars.next();
                    }
                    continue;
                }
                '\n' | '\r' => continue,
                _ => {
                    if !skip_group {
                        current.push(ch);
                    }
                }
            }
        }

        if !current.is_empty() {
            paragraphs.push(current);
        }

        Ok(paragraphs)
    }

    fn read_epub_text(&self, path: &Path) -> Result<Vec<String>, ConversionError> {
        let file = fs::File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut all_paragraphs = Vec::new();

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)
                .map_err(|e| ConversionError::ReadError(format!("EPUB entry error: {}", e)))?;

            let name = entry.name().to_string();
            if !name.ends_with(".xhtml") && !name.ends_with(".html") && !name.ends_with(".htm") {
                continue;
            }

            let mut content = String::new();
            entry.read_to_string(&mut content)?;

            let mut reader = XmlReader::from_reader(content.as_bytes());
            reader.config_mut().trim_text(true);

            let mut in_text = false;
            let mut current_text = String::new();
            let mut buf = Vec::new();

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                        let name_bytes = e.name().as_ref().to_vec();
                        let local = local_name(&name_bytes);
                        if local == b"p" || local == b"h1" || local == b"h2" || local == b"h3"
                            || local == b"h4" || local == b"h5" || local == b"h6"
                            || local == b"li" || local == b"div"
                        {
                            in_text = true;
                            current_text.clear();
                        }
                    }
                    Ok(Event::Text(ref e)) => {
                        if in_text {
                            if let Ok(t) = e.unescape() {
                                current_text.push_str(&t);
                            }
                        }
                    }
                    Ok(Event::End(ref e)) => {
                        let name_bytes = e.name().as_ref().to_vec();
                        let local = local_name(&name_bytes);
                        if (local == b"p" || local == b"h1" || local == b"h2" || local == b"h3"
                            || local == b"h4" || local == b"h5" || local == b"h6"
                            || local == b"li" || local == b"div")
                            && in_text
                        {
                            let trimmed = current_text.trim().to_string();
                            if !trimmed.is_empty() {
                                all_paragraphs.push(trimmed);
                            }
                            in_text = false;
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(_) => break,
                    _ => {}
                }
                buf.clear();
            }
        }

        Ok(all_paragraphs)
    }

    // ── Unified paragraph reader ───────────────────────────────────────

    fn read_paragraphs(&self, path: &Path) -> Result<Vec<String>, ConversionError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "pdf" => self.read_pdf_text(path),
            "docx" => self.read_docx_text(path),
            "odt" => self.read_odt_text(path),
            "txt" => self.read_txt(path),
            "rtf" => self.read_rtf_text(path),
            "epub" => self.read_epub_text(path),
            _ => Err(ConversionError::UnsupportedInputFormat(ext)),
        }
    }

    // ── Text document writers (from TextDocConverter) ──────────────────

    fn write_txt(&self, paragraphs: &[String], output: &Path) -> Result<(), ConversionError> {
        let content = paragraphs.join("\n");
        fs::write(output, content).map_err(|e| ConversionError::WriteError(e.to_string()))
    }

    fn write_docx(&self, paragraphs: &[String], output: &Path) -> Result<(), ConversionError> {
        let mut docx = docx_rs::Docx::new();

        for text in paragraphs {
            let run = docx_rs::Run::new().add_text(text);
            let para = docx_rs::Paragraph::new().add_run(run);
            docx = docx.add_paragraph(para);
        }

        let file = fs::File::create(output)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        docx.build()
            .pack(file)
            .map_err(|e| ConversionError::WriteError(format!("Failed to write DOCX: {}", e)))?;

        Ok(())
    }

    fn write_odt(&self, paragraphs: &[String], output: &Path) -> Result<(), ConversionError> {
        let file = fs::File::create(output)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        zip.start_file("mimetype", options)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        zip.write_all(b"application/vnd.oasis.opendocument.text")
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
  <manifest:file-entry manifest:media-type="application/vnd.oasis.opendocument.text" manifest:full-path="/"/>
  <manifest:file-entry manifest:media-type="text/xml" manifest:full-path="content.xml"/>
</manifest:manifest>"#;

        zip.start_file("META-INF/manifest.xml", options)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        zip.write_all(manifest.as_bytes())
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        let mut content = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" office:version="1.2">
<office:body><office:text>"#,
        );

        for text in paragraphs {
            content.push_str("<text:p>");
            content.push_str(&escape_xml(text));
            content.push_str("</text:p>");
        }

        content.push_str("</office:text></office:body></office:document-content>");

        zip.start_file("content.xml", options)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        zip.write_all(content.as_bytes())
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        zip.finish()
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        Ok(())
    }

    fn write_pdf(&self, paragraphs: &[String], output: &Path) -> Result<(), ConversionError> {
        let font_size_pt = 11.0_f32;
        let line_height_pt = 14.0_f32;
        let margin_left_pt = 56.7;
        let margin_top_pt = 793.7;
        let margin_bottom_pt = 56.7;
        let chars_per_line = 85;
        let usable_height = margin_top_pt - margin_bottom_pt;
        let lines_per_page = (usable_height / line_height_pt) as usize;

        let mut all_lines: Vec<String> = Vec::new();
        for para in paragraphs {
            if para.is_empty() {
                all_lines.push(String::new());
                continue;
            }
            let mut remaining = para.as_str();
            while !remaining.is_empty() {
                let take = remaining.len().min(chars_per_line);
                let split_at = if take < remaining.len() {
                    remaining[..take]
                        .rfind(' ')
                        .map(|i| i + 1)
                        .unwrap_or(take)
                } else {
                    take
                };
                all_lines.push(remaining[..split_at].to_string());
                remaining = &remaining[split_at..];
            }
        }

        let font = printpdf::PdfFontHandle::Builtin(printpdf::BuiltinFont::Helvetica);
        let mut doc = printpdf::PdfDocument::new("Document");
        let mut pages: Vec<printpdf::PdfPage> = Vec::new();

        let chunks: Vec<&[String]> = all_lines.chunks(lines_per_page).collect();
        if chunks.is_empty() {
            let page = printpdf::PdfPage::new(
                printpdf::Mm(210.0),
                printpdf::Mm(297.0),
                Vec::new(),
            );
            pages.push(page);
        }

        for chunk in &chunks {
            let mut ops = vec![
                printpdf::Op::StartTextSection,
                printpdf::Op::SetFont {
                    font: font.clone(),
                    size: printpdf::Pt(font_size_pt),
                },
                printpdf::Op::SetLineHeight {
                    lh: printpdf::Pt(line_height_pt),
                },
                printpdf::Op::SetTextCursor {
                    pos: printpdf::Point {
                        x: printpdf::Pt(margin_left_pt),
                        y: printpdf::Pt(margin_top_pt),
                    },
                },
            ];

            for line in *chunk {
                ops.push(printpdf::Op::ShowText {
                    items: vec![printpdf::TextItem::Text(line.clone())],
                });
                ops.push(printpdf::Op::AddLineBreak);
            }

            ops.push(printpdf::Op::EndTextSection);

            let page = printpdf::PdfPage::new(
                printpdf::Mm(210.0),
                printpdf::Mm(297.0),
                ops,
            );
            pages.push(page);
        }

        doc.with_pages(pages);

        let mut warnings = Vec::new();
        let file = fs::File::create(output)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        let mut writer = std::io::BufWriter::new(file);
        doc.save_writer(&mut writer, &printpdf::PdfSaveOptions::default(), &mut warnings);

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

fn local_name(full: &[u8]) -> &[u8] {
    match full.iter().position(|&b| b == b':') {
        Some(pos) => &full[pos + 1..],
        None => full,
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

impl Converter for DocumentConverter {
    fn supported_input_formats(&self) -> &[&str] {
        &["pdf", "docx", "odt", "txt", "rtf", "epub"]
    }

    fn supported_output_formats(&self) -> &[&str] {
        &["pdf", "docx", "odt", "txt", "png", "jpg"]
    }

    fn convert(
        &self,
        input: &Path,
        output: &Path,
        options: &ConversionOptions,
        on_progress: Box<dyn Fn(f32) + Send>,
    ) -> Result<(), ConversionError> {
        let out_ext = output
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // For image outputs (png/jpg), only PDF input supports image extraction
        if out_ext == "png" || out_ext == "jpg" {
            let in_ext = input
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if in_ext == "pdf" {
                return self.convert_pdf_to_image(input, output, options, on_progress);
            } else {
                return Err(ConversionError::UnsupportedOutputFormat(
                    format!("{} to {} is not supported", in_ext, out_ext),
                ));
            }
        }

        // For text-based outputs (pdf, docx, odt, txt): read paragraphs then write
        on_progress(0.0);

        let paragraphs = self.read_paragraphs(input)?;
        on_progress(0.5);

        match out_ext.as_str() {
            "txt" => self.write_txt(&paragraphs, output)?,
            "pdf" => self.write_pdf(&paragraphs, output)?,
            "docx" => self.write_docx(&paragraphs, output)?,
            "odt" => self.write_odt(&paragraphs, output)?,
            _ => return Err(ConversionError::UnsupportedOutputFormat(out_ext)),
        }

        on_progress(1.0);
        Ok(())
    }
}
