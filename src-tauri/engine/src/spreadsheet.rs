use std::path::Path;

use calamine::{open_workbook, Reader, Xlsx, Ods};
use rust_xlsxwriter::Workbook;

use crate::error::ConversionError;
use crate::traits::Converter;
use crate::types::ConversionOptions;

struct SheetData {
    name: String,
    rows: Vec<Vec<String>>,
}

pub struct SpreadsheetConverter;

impl SpreadsheetConverter {
    pub fn new() -> Self {
        Self
    }

    fn read_xlsx(&self, path: &Path) -> Result<Vec<SheetData>, ConversionError> {
        let mut workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| ConversionError::ReadError(format!("Failed to open XLSX: {}", e)))?;
        let sheet_names = workbook.sheet_names().to_vec();
        let mut sheets = Vec::new();

        for name in sheet_names {
            let range = workbook
                .worksheet_range(&name)
                .map_err(|e| ConversionError::ReadError(format!("Failed to read sheet '{}': {}", name, e)))?;
            sheets.push(SheetData {
                name,
                rows: self.range_to_rows(&range),
            });
        }

        Ok(sheets)
    }

    fn read_ods(&self, path: &Path) -> Result<Vec<SheetData>, ConversionError> {
        let mut workbook: Ods<_> = open_workbook(path)
            .map_err(|e| ConversionError::ReadError(format!("Failed to open ODS: {}", e)))?;
        let sheet_names = workbook.sheet_names().to_vec();
        let mut sheets = Vec::new();

        for name in sheet_names {
            let range = workbook
                .worksheet_range(&name)
                .map_err(|e| ConversionError::ReadError(format!("Failed to read sheet '{}': {}", name, e)))?;
            sheets.push(SheetData {
                name,
                rows: self.range_to_rows(&range),
            });
        }

        Ok(sheets)
    }

    fn range_to_rows(&self, range: &calamine::Range<calamine::Data>) -> Vec<Vec<String>> {
        range
            .rows()
            .map(|row| {
                row.iter()
                    .map(|cell| match cell {
                        calamine::Data::Empty => String::new(),
                        calamine::Data::String(s) => s.clone(),
                        calamine::Data::Float(f) => {
                            if *f == (*f as i64) as f64 {
                                format!("{}", *f as i64)
                            } else {
                                format!("{}", f)
                            }
                        }
                        calamine::Data::Int(i) => format!("{}", i),
                        calamine::Data::Bool(b) => format!("{}", b),
                        calamine::Data::Error(e) => format!("{:?}", e),
                        calamine::Data::DateTime(dt) => format!("{}", dt),
                        calamine::Data::DateTimeIso(s) => s.clone(),
                        calamine::Data::DurationIso(s) => s.clone(),
                    })
                    .collect()
            })
            .collect()
    }

    fn read_csv(&self, path: &Path) -> Result<Vec<SheetData>, ConversionError> {
        let mut reader = csv::Reader::from_path(path)
            .map_err(|e| ConversionError::ReadError(format!("Failed to open CSV: {}", e)))?;

        let mut rows = Vec::new();

        if reader.has_headers() {
            let headers: Vec<String> = reader
                .headers()
                .map_err(|e| ConversionError::ReadError(e.to_string()))?
                .iter()
                .map(|s| s.to_string())
                .collect();
            rows.push(headers);
        }

        for result in reader.records() {
            let record = result
                .map_err(|e| ConversionError::ReadError(e.to_string()))?;
            let cells: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            rows.push(cells);
        }

        Ok(vec![SheetData {
            name: "Sheet1".to_string(),
            rows,
        }])
    }

    fn write_csv(&self, sheets: &[SheetData], output: &Path) -> Result<(), ConversionError> {
        let sheet = sheets.first().ok_or_else(|| {
            ConversionError::ConversionFailed("No sheet data to write".to_string())
        })?;

        let mut writer = csv::Writer::from_path(output)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        for row in &sheet.rows {
            writer
                .write_record(row)
                .map_err(|e| ConversionError::WriteError(e.to_string()))?;
        }

        writer
            .flush()
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        Ok(())
    }

    fn write_xlsx(&self, sheets: &[SheetData], output: &Path) -> Result<(), ConversionError> {
        let mut workbook = Workbook::new();

        for sheet_data in sheets {
            let worksheet = workbook.add_worksheet();
            worksheet
                .set_name(&sheet_data.name)
                .map_err(|e| ConversionError::WriteError(e.to_string()))?;

            for (row_idx, row) in sheet_data.rows.iter().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    if let Ok(num) = cell.parse::<f64>() {
                        worksheet
                            .write_number(row_idx as u32, col_idx as u16, num)
                            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
                    } else {
                        worksheet
                            .write_string(row_idx as u32, col_idx as u16, cell)
                            .map_err(|e| ConversionError::WriteError(e.to_string()))?;
                    }
                }
            }
        }

        workbook
            .save(output)
            .map_err(|e| ConversionError::WriteError(e.to_string()))?;

        Ok(())
    }

    fn write_ods(&self, sheets: &[SheetData], output: &Path) -> Result<(), ConversionError> {
        let mut workbook = spreadsheet_ods::WorkBook::new_empty();

        for sheet_data in sheets {
            let mut sheet = spreadsheet_ods::Sheet::new(&sheet_data.name);

            for (row_idx, row) in sheet_data.rows.iter().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    if let Ok(num) = cell.parse::<f64>() {
                        sheet.set_value(row_idx as u32, col_idx as u32, num);
                    } else {
                        sheet.set_value(row_idx as u32, col_idx as u32, cell.as_str());
                    }
                }
            }

            workbook.push_sheet(sheet);
        }

        spreadsheet_ods::write_ods(&mut workbook, output)
            .map_err(|e| ConversionError::WriteError(format!("Failed to write ODS: {}", e)))?;

        Ok(())
    }

    fn read_sheets(&self, path: &Path) -> Result<Vec<SheetData>, ConversionError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "xlsx" | "xls" => self.read_xlsx(path),
            "ods" => self.read_ods(path),
            "csv" => self.read_csv(path),
            _ => Err(ConversionError::UnsupportedInputFormat(ext)),
        }
    }
}

impl Converter for SpreadsheetConverter {
    fn supported_input_formats(&self) -> &[&str] {
        &["xlsx", "xls", "ods", "csv"]
    }

    fn supported_output_formats(&self) -> &[&str] {
        &["csv", "xlsx", "ods"]
    }

    fn convert(
        &self,
        input: &Path,
        output: &Path,
        _options: &ConversionOptions,
        on_progress: Box<dyn Fn(f32) + Send>,
    ) -> Result<(), ConversionError> {
        on_progress(0.0);

        let sheets = self.read_sheets(input)?;
        on_progress(0.5);

        let out_ext = output
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match out_ext.as_str() {
            "csv" => self.write_csv(&sheets, output)?,
            "xlsx" => self.write_xlsx(&sheets, output)?,
            "ods" => self.write_ods(&sheets, output)?,
            _ => return Err(ConversionError::UnsupportedOutputFormat(out_ext)),
        }

        on_progress(1.0);
        Ok(())
    }
}
