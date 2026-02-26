use std::fs;
use std::io::{Read as IoRead, Write as IoWrite};
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::error::ConversionError;
use crate::traits::Converter;
use crate::types::ConversionOptions;

struct SlideData {
    texts: Vec<String>,
}

pub struct PresentationConverter;

impl PresentationConverter {
    pub fn new() -> Self {
        Self
    }

    fn read_pptx_slides(&self, path: &Path) -> Result<Vec<SlideData>, ConversionError> {
        let file = fs::File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut slide_names: Vec<String> = Vec::new();
        for i in 0..archive.len() {
            let entry = archive.by_index(i)
                .map_err(|e| ConversionError::ReadError(e.to_string()))?;
            let name = entry.name().to_string();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_names.push(name);
            }
        }
        slide_names.sort();

        let mut slides = Vec::new();
        for name in &slide_names {
            let mut xml_content = String::new();
            {
                let mut entry = archive.by_name(name)
                    .map_err(|e| ConversionError::ReadError(e.to_string()))?;
                entry.read_to_string(&mut xml_content)?;
            }

            let texts = self.extract_pptx_texts(&xml_content)?;
            slides.push(SlideData { texts });
        }

        Ok(slides)
    }

    fn extract_pptx_texts(&self, xml: &str) -> Result<Vec<String>, ConversionError> {
        let mut reader = XmlReader::from_reader(xml.as_bytes());
        reader.config_mut().trim_text(true);

        let mut texts = Vec::new();
        let mut in_text_body = false;
        let mut current_paragraph = String::new();
        let mut in_paragraph = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if local == b"txBody" {
                        in_text_body = true;
                    } else if in_text_body && local == b"p" {
                        in_paragraph = true;
                        current_paragraph.clear();
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_paragraph {
                        if let Ok(t) = e.unescape() {
                            current_paragraph.push_str(&t);
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if local == b"p" && in_paragraph {
                        if !current_paragraph.trim().is_empty() {
                            texts.push(current_paragraph.clone());
                        }
                        in_paragraph = false;
                    } else if local == b"txBody" {
                        in_text_body = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ConversionError::ReadError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(texts)
    }

    fn read_odp_slides(&self, path: &Path) -> Result<Vec<SlideData>, ConversionError> {
        let file = fs::File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut content_xml = String::new();
        {
            let mut entry = archive.by_name("content.xml")
                .map_err(|e| ConversionError::ReadError(format!("No content.xml in ODP: {}", e)))?;
            entry.read_to_string(&mut content_xml)?;
        }

        let mut reader = XmlReader::from_reader(content_xml.as_bytes());
        reader.config_mut().trim_text(true);

        let mut slides = Vec::new();
        let mut in_page = false;
        let mut in_text_p = false;
        let mut current_texts: Vec<String> = Vec::new();
        let mut current_paragraph = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if local == b"page" {
                        in_page = true;
                        current_texts.clear();
                    } else if in_page && (local == b"p" || local == b"h") {
                        in_text_p = true;
                        current_paragraph.clear();
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_text_p {
                        if let Ok(t) = e.unescape() {
                            current_paragraph.push_str(&t);
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if (local == b"p" || local == b"h") && in_text_p {
                        if !current_paragraph.trim().is_empty() {
                            current_texts.push(current_paragraph.clone());
                        }
                        in_text_p = false;
                    } else if local == b"page" && in_page {
                        slides.push(SlideData {
                            texts: current_texts.clone(),
                        });
                        in_page = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ConversionError::ReadError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(slides)
    }

    fn write_pdf(&self, slides: &[SlideData], output: &Path) -> Result<(), ConversionError> {
        let font = printpdf::PdfFontHandle::Builtin(printpdf::BuiltinFont::Helvetica);
        let mut doc = printpdf::PdfDocument::new("Presentation");
        let mut pages = Vec::new();

        for (i, slide) in slides.iter().enumerate() {
            let mut ops = vec![
                printpdf::Op::StartTextSection,
                printpdf::Op::SetFont {
                    font: font.clone(),
                    size: printpdf::Pt(16.0),
                },
                printpdf::Op::SetTextCursor {
                    pos: printpdf::Point {
                        x: printpdf::Pt(56.7),
                        y: printpdf::Pt(553.0),
                    },
                },
                printpdf::Op::ShowText {
                    items: vec![printpdf::TextItem::Text(format!("Slide {}", i + 1))],
                },
                printpdf::Op::EndTextSection,
                printpdf::Op::StartTextSection,
                printpdf::Op::SetFont {
                    font: font.clone(),
                    size: printpdf::Pt(12.0),
                },
                printpdf::Op::SetLineHeight {
                    lh: printpdf::Pt(16.0),
                },
                printpdf::Op::SetTextCursor {
                    pos: printpdf::Point {
                        x: printpdf::Pt(56.7),
                        y: printpdf::Pt(520.0),
                    },
                },
            ];

            for text in &slide.texts {
                ops.push(printpdf::Op::ShowText {
                    items: vec![printpdf::TextItem::Text(text.clone())],
                });
                ops.push(printpdf::Op::AddLineBreak);
            }

            ops.push(printpdf::Op::EndTextSection);

            let page = printpdf::PdfPage::new(
                printpdf::Mm(297.0),
                printpdf::Mm(210.0),
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

    fn write_pptx(&self, slides: &[SlideData], output: &Path) -> Result<(), ConversionError> {
        let file = fs::File::create(output)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        let content_types = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  {}</Types>"#,
            (1..=slides.len())
                .map(|i| format!(
                    r#"<Override PartName="/ppt/slides/slide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#,
                    i
                ))
                .collect::<Vec<_>>()
                .join("\n  ")
        );

        zip.start_file("[Content_Types].xml", options)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        zip.write_all(content_types.as_bytes())
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        let root_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#;

        zip.start_file("_rels/.rels", options)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        zip.write_all(root_rels.as_bytes())
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        let slide_refs: String = (1..=slides.len())
            .map(|i| format!(r#"<p:sldId id="{}" r:id="rId{}"/>"#, 255 + i, i))
            .collect::<Vec<_>>()
            .join("\n    ");

        let presentation = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:sldIdLst>
    {}
  </p:sldIdLst>
  <p:sldSz cx="9144000" cy="6858000" type="screen4x3"/>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#,
            slide_refs
        );

        zip.start_file("ppt/presentation.xml", options)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        zip.write_all(presentation.as_bytes())
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        let ppt_rels = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  {}
</Relationships>"#,
            (1..=slides.len())
                .map(|i| format!(
                    r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>"#,
                    i, i
                ))
                .collect::<Vec<_>>()
                .join("\n  ")
        );

        zip.start_file("ppt/_rels/presentation.xml.rels", options)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        zip.write_all(ppt_rels.as_bytes())
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        for (i, slide) in slides.iter().enumerate() {
            let text_bodies: String = slide
                .texts
                .iter()
                .enumerate()
                .map(|(j, text)| {
                    let y_pos = 1600000 + (j as i64 * 500000);
                    format!(
                        r#"<p:sp>
  <p:nvSpPr><p:cNvPr id="{}" name="TextBox {}"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>
  <p:spPr><a:xfrm><a:off x="457200" y="{}"/><a:ext cx="8229600" cy="400000"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>
  <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>{}</a:t></a:r></a:p></p:txBody>
</p:sp>"#,
                        j + 2,
                        j + 1,
                        y_pos,
                        escape_xml(text)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let slide_xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>
{}
</p:spTree></p:cSld></p:sld>"#,
                text_bodies
            );

            zip.start_file(format!("ppt/slides/slide{}.xml", i + 1), options)
                .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            zip.write_all(slide_xml.as_bytes())
                .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        }

        zip.finish()
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        Ok(())
    }

    fn write_odp(&self, slides: &[SlideData], output: &Path) -> Result<(), ConversionError> {
        let file = fs::File::create(output)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        zip.start_file("mimetype", options)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        zip.write_all(b"application/vnd.oasis.opendocument.presentation")
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
  <manifest:file-entry manifest:media-type="application/vnd.oasis.opendocument.presentation" manifest:full-path="/"/>
  <manifest:file-entry manifest:media-type="text/xml" manifest:full-path="content.xml"/>
</manifest:manifest>"#;

        zip.start_file("META-INF/manifest.xml", options)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        zip.write_all(manifest.as_bytes())
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        let mut content = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:presentation="urn:oasis:names:tc:opendocument:xmlns:presentation:1.0" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0" office:version="1.2">
<office:body><office:presentation>"#,
        );

        for (i, slide) in slides.iter().enumerate() {
            content.push_str(&format!(
                r#"<draw:page draw:name="Slide {}" presentation:presentation-page-layout-name="AL0T0">"#,
                i + 1
            ));

            for text in &slide.texts {
                content.push_str(r#"<draw:frame draw:layer="layout" svg:x="2cm" svg:y="2cm" svg:width="24cm" svg:height="2cm"><draw:text-box><text:p>"#);
                content.push_str(&escape_xml(text));
                content.push_str("</text:p></draw:text-box></draw:frame>");
            }

            content.push_str("</draw:page>");
        }

        content.push_str("</office:presentation></office:body></office:document-content>");

        zip.start_file("content.xml", options)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        zip.write_all(content.as_bytes())
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        zip.finish()
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        Ok(())
    }

    fn read_slides(&self, path: &Path) -> Result<Vec<SlideData>, ConversionError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "pptx" => self.read_pptx_slides(path),
            "odp" => self.read_odp_slides(path),
            _ => Err(ConversionError::UnsupportedInputFormat(ext)),
        }
    }
}

impl Converter for PresentationConverter {
    fn supported_input_formats(&self) -> &[&str] {
        &["pptx", "odp"]
    }

    fn supported_output_formats(&self) -> &[&str] {
        &["pdf", "pptx", "odp"]
    }

    fn convert(
        &self,
        input: &Path,
        output: &Path,
        _options: &ConversionOptions,
        on_progress: Box<dyn Fn(f32) + Send>,
    ) -> Result<(), ConversionError> {
        on_progress(0.0);

        let slides = self.read_slides(input)?;
        on_progress(0.5);

        if slides.is_empty() {
            return Err(ConversionError::ConversionFailed(
                "No slides found in presentation".to_string(),
            ));
        }

        let out_ext = output
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match out_ext.as_str() {
            "pdf" => self.write_pdf(&slides, output)?,
            "pptx" => self.write_pptx(&slides, output)?,
            "odp" => self.write_odp(&slides, output)?,
            _ => return Err(ConversionError::UnsupportedOutputFormat(out_ext)),
        }

        on_progress(1.0);
        Ok(())
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
