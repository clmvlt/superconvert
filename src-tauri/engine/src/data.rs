use std::fs;
use std::path::Path;

use crate::error::ConversionError;
use crate::traits::Converter;
use crate::types::{ConversionOptions, DataFormat, DocumentFormat, OutputFormat};

pub struct DataConverter;

impl DataConverter {
    pub fn new() -> Self {
        Self
    }

    fn read_as_value(&self, input: &Path) -> Result<serde_json::Value, ConversionError> {
        let ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let content = fs::read_to_string(input)
            .map_err(|e| ConversionError::ReadError(e.to_string()))?;

        match ext.as_str() {
            "json" => {
                serde_json::from_str(&content).map_err(Into::into)
            }
            "yaml" | "yml" => {
                serde_yaml::from_str(&content).map_err(Into::into)
            }
            "toml" => {
                let val: toml::Value = toml::from_str(&content)?;
                let json_str = serde_json::to_string(&val)?;
                serde_json::from_str(&json_str).map_err(Into::into)
            }
            "xml" => {
                self.xml_to_json(&content)
            }
            _ => Err(ConversionError::UnsupportedInputFormat(ext)),
        }
    }

    fn xml_to_json(&self, xml_content: &str) -> Result<serde_json::Value, ConversionError> {
        let mut reader = quick_xml::Reader::from_str(xml_content);
        let value = self.parse_xml_element(&mut reader)?;
        Ok(value)
    }

    fn parse_xml_element(
        &self,
        reader: &mut quick_xml::Reader<&[u8]>,
    ) -> Result<serde_json::Value, ConversionError> {
        use quick_xml::events::Event;
        use serde_json::{Map, Value};

        let mut root = Map::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let child = self.parse_xml_children(reader, &name)?;
                    if root.contains_key(&name) {
                        let existing = root.remove(&name).unwrap();
                        if let Value::Array(mut arr) = existing {
                            arr.push(child);
                            root.insert(name, Value::Array(arr));
                        } else {
                            root.insert(name, Value::Array(vec![existing, child]));
                        }
                    } else {
                        root.insert(name, child);
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape().unwrap_or_default().trim().to_string();
                    if !text.is_empty() {
                        return Ok(Value::String(text));
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ConversionError::ReadError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(Value::Object(root))
    }

    fn parse_xml_children(
        &self,
        reader: &mut quick_xml::Reader<&[u8]>,
        _tag_name: &str,
    ) -> Result<serde_json::Value, ConversionError> {
        use quick_xml::events::Event;
        use serde_json::{Map, Value};

        let mut children = Map::new();
        let mut text_content = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let child = self.parse_xml_children(reader, &name)?;
                    if children.contains_key(&name) {
                        let existing = children.remove(&name).unwrap();
                        if let Value::Array(mut arr) = existing {
                            arr.push(child);
                            children.insert(name, Value::Array(arr));
                        } else {
                            children.insert(name, Value::Array(vec![existing, child]));
                        }
                    } else {
                        children.insert(name, child);
                    }
                }
                Ok(Event::Text(ref e)) => {
                    text_content.push_str(&e.unescape().unwrap_or_default());
                }
                Ok(Event::End(_)) => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(ConversionError::ReadError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        if children.is_empty() {
            let trimmed = text_content.trim().to_string();
            Ok(Value::String(trimmed))
        } else {
            Ok(Value::Object(children))
        }
    }

    fn json_to_xml(&self, value: &serde_json::Value) -> Result<String, ConversionError> {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        match value {
            serde_json::Value::Object(map) => {
                for (key, val) in map {
                    self.value_to_xml(key, val, &mut xml, 0);
                }
            }
            _ => {
                xml.push_str("<root>");
                self.write_xml_value(value, &mut xml);
                xml.push_str("</root>\n");
            }
        }
        Ok(xml)
    }

    fn value_to_xml(
        &self,
        tag: &str,
        value: &serde_json::Value,
        xml: &mut String,
        indent: usize,
    ) {
        let pad = "  ".repeat(indent);
        match value {
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.value_to_xml(tag, item, xml, indent);
                }
            }
            serde_json::Value::Object(map) => {
                xml.push_str(&format!("{}<{}>\n", pad, tag));
                for (key, val) in map {
                    self.value_to_xml(key, val, xml, indent + 1);
                }
                xml.push_str(&format!("{}</{}>\n", pad, tag));
            }
            _ => {
                xml.push_str(&format!("{}<{}>", pad, tag));
                self.write_xml_value(value, xml);
                xml.push_str(&format!("</{}>\n", tag));
            }
        }
    }

    fn write_xml_value(&self, value: &serde_json::Value, xml: &mut String) {
        match value {
            serde_json::Value::String(s) => {
                xml.push_str(&s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;"));
            }
            serde_json::Value::Number(n) => xml.push_str(&n.to_string()),
            serde_json::Value::Bool(b) => xml.push_str(&b.to_string()),
            serde_json::Value::Null => xml.push_str("null"),
            _ => {}
        }
    }

    fn json_to_csv(&self, value: &serde_json::Value) -> Result<String, ConversionError> {
        match value {
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    return Ok(String::new());
                }

                let mut headers: Vec<String> = Vec::new();
                for item in arr {
                    if let serde_json::Value::Object(map) = item {
                        for key in map.keys() {
                            if !headers.contains(key) {
                                headers.push(key.clone());
                            }
                        }
                    }
                }

                if headers.is_empty() {
                    let mut wtr = csv::Writer::from_writer(Vec::new());
                    for item in arr {
                        wtr.write_record(&[item.to_string()])?;
                    }
                    let data = wtr.into_inner().map_err(|e| {
                        ConversionError::WriteError(e.to_string())
                    })?;
                    return Ok(String::from_utf8_lossy(&data).to_string());
                }

                let mut wtr = csv::Writer::from_writer(Vec::new());
                wtr.write_record(&headers)?;

                for item in arr {
                    if let serde_json::Value::Object(map) = item {
                        let row: Vec<String> = headers
                            .iter()
                            .map(|h| match map.get(h) {
                                Some(serde_json::Value::String(s)) => s.clone(),
                                Some(v) => v.to_string(),
                                None => String::new(),
                            })
                            .collect();
                        wtr.write_record(&row)?;
                    }
                }

                let data = wtr.into_inner().map_err(|e| {
                    ConversionError::WriteError(e.to_string())
                })?;
                Ok(String::from_utf8_lossy(&data).to_string())
            }
            _ => Err(ConversionError::ConversionFailed(
                "JSON to CSV requires an array of objects".to_string(),
            )),
        }
    }

    fn markdown_to_html(&self, md: &str) -> String {
        let parser = pulldown_cmark::Parser::new(md);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);
        format!(
            "<!DOCTYPE html>\n<html>\n<head><meta charset=\"UTF-8\"></head>\n<body>\n{}</body>\n</html>",
            html_output
        )
    }

    fn html_to_markdown(&self, html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        let mut tag_name = String::new();
        let mut collecting_tag = false;

        for ch in html.chars() {
            if ch == '<' {
                in_tag = true;
                collecting_tag = true;
                tag_name.clear();
                continue;
            }
            if ch == '>' {
                in_tag = false;
                collecting_tag = false;
                let lower = tag_name.to_lowercase();
                match lower.as_str() {
                    "br" | "br/" => result.push('\n'),
                    "/p" | "/div" | "/h1" | "/h2" | "/h3" | "/h4" | "/h5" | "/h6" | "/li" => {
                        result.push('\n');
                    }
                    "h1" => result.push_str("# "),
                    "h2" => result.push_str("## "),
                    "h3" => result.push_str("### "),
                    "h4" => result.push_str("#### "),
                    "li" => result.push_str("- "),
                    _ => {}
                }
                continue;
            }
            if in_tag {
                if collecting_tag && ch != ' ' && ch != '/' {
                    tag_name.push(ch);
                } else {
                    collecting_tag = false;
                }
                continue;
            }
            result.push(ch);
        }

        result
            .lines()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn text_to_pdf(&self, text: &str, output: &Path) -> Result<(), ConversionError> {
        let lines: Vec<&str> = text.lines().collect();
        let line_height = 4.5_f32;
        let margin = 20.0_f32;
        let page_height = 297.0_f32;
        let usable_height = page_height - 2.0 * margin;
        let max_lines_per_page = (usable_height / line_height) as usize;

        let font_size = 10.0;
        let max_chars_per_line = 85usize;

        let mut wrapped_lines: Vec<String> = Vec::new();
        for line in &lines {
            if line.len() <= max_chars_per_line {
                wrapped_lines.push(line.to_string());
            } else {
                let mut remaining = *line;
                while remaining.len() > max_chars_per_line {
                    let split_pos = remaining[..max_chars_per_line]
                        .rfind(' ')
                        .unwrap_or(max_chars_per_line);
                    wrapped_lines.push(remaining[..split_pos].to_string());
                    remaining = remaining[split_pos..].trim_start();
                }
                if !remaining.is_empty() {
                    wrapped_lines.push(remaining.to_string());
                }
            }
        }

        let mut pages = Vec::new();
        for chunk in wrapped_lines.chunks(max_lines_per_page) {
            let font = printpdf::PdfFontHandle::Builtin(printpdf::BuiltinFont::Helvetica);
            let mut ops = vec![
                printpdf::Op::StartTextSection,
                printpdf::Op::SetFont {
                    font,
                    size: printpdf::Pt(font_size),
                },
                printpdf::Op::SetLineHeight {
                    lh: printpdf::Pt(line_height * 2.83465),
                },
                printpdf::Op::SetTextCursor {
                    pos: printpdf::Point {
                        x: printpdf::Pt(margin * 2.83465),
                        y: printpdf::Pt((page_height - margin) * 2.83465),
                    },
                },
            ];

            for line in chunk {
                ops.push(printpdf::Op::ShowText {
                    items: vec![printpdf::TextItem::Text(line.clone())],
                });
                ops.push(printpdf::Op::AddLineBreak);
            }

            ops.push(printpdf::Op::EndTextSection);

            pages.push(printpdf::PdfPage::new(
                printpdf::Mm(210.0),
                printpdf::Mm(page_height),
                ops,
            ));
        }

        if pages.is_empty() {
            pages.push(printpdf::PdfPage::new(
                printpdf::Mm(210.0),
                printpdf::Mm(page_height),
                Vec::new(),
            ));
        }

        let mut doc = printpdf::PdfDocument::new("SuperConvert");
        doc.with_pages(pages);

        let mut warnings = Vec::new();
        let file =
            fs::File::create(output).map_err(|e| ConversionError::WriteError(e.to_string()))?;
        let mut writer = std::io::BufWriter::new(file);
        doc.save_writer(&mut writer, &printpdf::PdfSaveOptions::default(), &mut warnings);

        Ok(())
    }
}

impl Converter for DataConverter {
    fn supported_input_formats(&self) -> &[&str] {
        &["json", "yaml", "yml", "toml", "xml", "md", "html", "htm"]
    }

    fn supported_output_formats(&self) -> &[&str] {
        &["json", "yaml", "toml", "xml", "md", "html", "csv", "txt", "pdf"]
    }

    fn convert(
        &self,
        input: &Path,
        output: &Path,
        options: &ConversionOptions,
        on_progress: Box<dyn Fn(f32) + Send>,
    ) -> Result<(), ConversionError> {
        on_progress(0.0);

        let ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let is_markup = matches!(ext.as_str(), "md" | "html" | "htm");

        on_progress(0.2);

        if is_markup {
            let content = fs::read_to_string(input)
                .map_err(|e| ConversionError::ReadError(e.to_string()))?;

            on_progress(0.5);

            let output_text = match (&ext.as_str(), &options.output_format) {
                (&"md", OutputFormat::Data(DataFormat::Html)) => {
                    self.markdown_to_html(&content)
                }
                (&"md", OutputFormat::Data(DataFormat::Txt)) | (&"md", OutputFormat::Document(DocumentFormat::Txt)) => {
                    content.clone()
                }
                (&"md", OutputFormat::Document(DocumentFormat::Pdf)) | (&"md", OutputFormat::Data(DataFormat::Csv)) => {
                    let html = self.markdown_to_html(&content);
                    let text = self.html_to_markdown(&html);
                    if matches!(options.output_format, OutputFormat::Document(DocumentFormat::Pdf)) {
                        self.text_to_pdf(&text, output)?;
                        on_progress(1.0);
                        return Ok(());
                    }
                    text
                }
                (&"html" | &"htm", OutputFormat::Data(DataFormat::Markdown)) => {
                    self.html_to_markdown(&content)
                }
                (&"html" | &"htm", OutputFormat::Data(DataFormat::Txt)) | (&"html" | &"htm", OutputFormat::Document(DocumentFormat::Txt)) => {
                    self.html_to_markdown(&content)
                }
                (&"html" | &"htm", OutputFormat::Document(DocumentFormat::Pdf)) => {
                    let text = self.html_to_markdown(&content);
                    self.text_to_pdf(&text, output)?;
                    on_progress(1.0);
                    return Ok(());
                }
                _ => {
                    return Err(ConversionError::UnsupportedOutputFormat(
                        options.output_format.extension().to_string(),
                    ));
                }
            };

            fs::write(output, output_text)
                .map_err(|e| ConversionError::WriteError(e.to_string()))?;

            on_progress(1.0);
            return Ok(());
        }

        let value = self.read_as_value(input)?;
        on_progress(0.5);

        match &options.output_format {
            OutputFormat::Data(DataFormat::Json) => {
                let json_str = serde_json::to_string_pretty(&value)?;
                fs::write(output, json_str)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            OutputFormat::Data(DataFormat::Yaml) => {
                let yaml_str = serde_yaml::to_string(&value)?;
                fs::write(output, yaml_str)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            OutputFormat::Data(DataFormat::Toml) => {
                let toml_str = toml::to_string_pretty(&value)?;
                fs::write(output, toml_str)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            OutputFormat::Data(DataFormat::Xml) => {
                let xml_str = self.json_to_xml(&value)?;
                fs::write(output, xml_str)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            OutputFormat::Data(DataFormat::Csv) | OutputFormat::Document(DocumentFormat::Csv) => {
                let csv_str = self.json_to_csv(&value)?;
                fs::write(output, csv_str)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            OutputFormat::Data(DataFormat::Txt) | OutputFormat::Document(DocumentFormat::Txt) => {
                let text = serde_json::to_string_pretty(&value)?;
                fs::write(output, text)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
            }
            OutputFormat::Document(DocumentFormat::Pdf) => {
                let text = serde_json::to_string_pretty(&value)?;
                self.text_to_pdf(&text, output)?;
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
