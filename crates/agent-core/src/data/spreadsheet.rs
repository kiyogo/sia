use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ColumnDataType {
    Integer,
    Float,
    String,
    Boolean,
    Date,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: ColumnDataType,
    pub non_null_count: usize,
    pub null_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetTable {
    pub name: String,
    pub headers: Vec<String>,
    pub columns: Vec<ColumnMetadata>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetWorkbook {
    pub file_path: String,
    pub sheets: Vec<SpreadsheetTable>,
}

pub struct SpreadsheetReader;

impl SpreadsheetReader {
    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<SpreadsheetWorkbook, String> {
        let path_ref = path.as_ref();
        let ext = path_ref
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "csv" | "tsv" | "txt" => {
                let content = std::fs::read_to_string(path_ref)
                    .map_err(|e| format!("Failed to read CSV: {}", e))?;
                let delimiter = if ext == "tsv" { '\t' } else { ',' };
                let table = Self::parse_csv_content(&content, "Sheet1", delimiter)?;
                Ok(SpreadsheetWorkbook {
                    file_path: path_ref.to_string_lossy().to_string(),
                    sheets: vec![table],
                })
            }
            "xlsx" => {
                let bytes = std::fs::read(path_ref)
                    .map_err(|e| format!("Failed to read XLSX: {}", e))?;
                Self::parse_xlsx_bytes(&bytes, path_ref.to_string_lossy().as_ref())
            }
            _ => Err(format!("Unsupported spreadsheet format: .{}", ext)),
        }
    }

    pub fn parse_csv_content(content: &str, sheet_name: &str, delimiter: char) -> Result<SpreadsheetTable, String> {
        let mut lines = content.lines().filter(|l| !l.trim().is_empty());
        let header_line = lines.next().ok_or_else(|| "Empty CSV content".to_string())?;

        let headers: Vec<String> = header_line
            .split(delimiter)
            .map(|s| s.trim().trim_matches('"').to_string())
            .collect();

        let mut rows = Vec::new();
        for line in lines {
            let cells: Vec<String> = line
                .split(delimiter)
                .map(|s| s.trim().trim_matches('"').to_string())
                .collect();
            if cells.len() == headers.len() {
                rows.push(cells);
            } else if !cells.is_empty() {
                // Pad or truncate to match header length
                let mut aligned = cells;
                while aligned.len() < headers.len() {
                    aligned.push(String::new());
                }
                aligned.truncate(headers.len());
                rows.push(aligned);
            }
        }

        let columns = Self::infer_columns(&headers, &rows);
        let total_rows = rows.len();

        Ok(SpreadsheetTable {
            name: sheet_name.to_string(),
            headers,
            columns,
            rows,
            total_rows,
        })
    }

    pub fn parse_xlsx_bytes(bytes: &[u8], file_path: &str) -> Result<SpreadsheetWorkbook, String> {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| format!("Invalid XLSX archive: {}", e))?;

        // 1. Read shared strings if available
        let mut shared_strings = Vec::new();
        if let Ok(mut ss_file) = archive.by_name("xl/sharedStrings.xml") {
            let mut ss_content = String::new();
            if ss_file.read_to_string(&mut ss_content).is_ok() {
                shared_strings = extract_shared_strings(&ss_content);
            }
        }

        // 2. Discover sheets from xl/workbook.xml
        let mut sheet_names = Vec::new();
        if let Ok(mut wb_file) = archive.by_name("xl/workbook.xml") {
            let mut wb_content = String::new();
            if wb_file.read_to_string(&mut wb_content).is_ok() {
                sheet_names = extract_sheet_names(&wb_content);
            }
        }

        if sheet_names.is_empty() {
            sheet_names.push("Sheet1".to_string());
        }

        let mut sheets = Vec::new();
        // 3. Read sheet1.xml (and subsequent sheets if present)
        for i in 1..=sheet_names.len() {
            let sheet_path = format!("xl/worksheets/sheet{}.xml", i);
            if let Ok(mut sheet_file) = archive.by_name(&sheet_path) {
                let mut sheet_content = String::new();
                if sheet_file.read_to_string(&mut sheet_content).is_ok() {
                    let table = parse_worksheet_xml(
                        &sheet_content,
                        &sheet_names[i - 1],
                        &shared_strings,
                    )?;
                    sheets.push(table);
                }
            }
        }

        if sheets.is_empty() {
            return Err("No valid worksheet data found in XLSX archive".to_string());
        }

        Ok(SpreadsheetWorkbook {
            file_path: file_path.to_string(),
            sheets,
        })
    }

    fn infer_columns(headers: &[String], rows: &[Vec<String>]) -> Vec<ColumnMetadata> {
        let mut metadata = Vec::new();

        for (col_idx, header) in headers.iter().enumerate() {
            let mut non_null = 0;
            let mut null_count = 0;
            let mut is_int = true;
            let mut is_float = true;
            let mut is_bool = true;

            for row in rows {
                if let Some(val) = row.get(col_idx) {
                    let trimmed = val.trim();
                    if trimmed.is_empty() || trimmed == "null" || trimmed == "NA" || trimmed == "NaN" {
                        null_count += 1;
                    } else {
                        non_null += 1;
                        if trimmed.parse::<i64>().is_err() {
                            is_int = false;
                        }
                        if trimmed.parse::<f64>().is_err() {
                            is_float = false;
                        }
                        let lower = trimmed.to_lowercase();
                        if lower != "true" && lower != "false" && lower != "1" && lower != "0" {
                            is_bool = false;
                        }
                    }
                } else {
                    null_count += 1;
                }
            }

            let data_type = if non_null > 0 && is_int {
                ColumnDataType::Integer
            } else if non_null > 0 && is_float {
                ColumnDataType::Float
            } else if non_null > 0 && is_bool {
                ColumnDataType::Boolean
            } else {
                ColumnDataType::String
            };

            metadata.push(ColumnMetadata {
                name: header.clone(),
                data_type,
                non_null_count: non_null,
                null_count,
            });
        }

        metadata
    }
}

fn extract_shared_strings(xml: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut search = xml;
    while let Some(start) = search.find("<t>") {
        let after = &search[start + 3..];
        if let Some(end) = after.find("</t>") {
            strings.push(after[..end].to_string());
            search = &after[end + 4..];
        } else {
            break;
        }
    }
    strings
}

fn extract_sheet_names(xml: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut search = xml;
    while let Some(start) = search.find("name=\"") {
        let after = &search[start + 6..];
        if let Some(end) = after.find('"') {
            names.push(after[..end].to_string());
            search = &after[end + 1..];
        } else {
            break;
        }
    }
    names
}

fn parse_worksheet_xml(
    xml: &str,
    sheet_name: &str,
    shared_strings: &[String],
) -> Result<SpreadsheetTable, String> {
    let mut raw_rows: Vec<Vec<String>> = Vec::new();
    let mut search = xml;

    while let Some(r_start) = search.find("<row ") {
        let after_row = &search[r_start..];
        if let Some(r_end) = after_row.find("</row>") {
            let row_xml = &after_row[..r_end];
            let mut cells = Vec::new();

            let mut cell_search = row_xml;
            while let Some(c_start) = cell_search.find("<c ") {
                let after_c = &cell_search[c_start..];
                if let Some(c_end) = after_c.find("</c>") {
                    let cell_xml = &after_c[..c_end];
                    let is_shared_string = cell_xml.contains("t=\"s\"");

                    let mut cell_value = String::new();
                    if let Some(v_start) = cell_xml.find("<v>") {
                        if let Some(v_end) = cell_xml[v_start + 3..].find("</v>") {
                            let raw_v = &cell_xml[v_start + 3..v_start + 3 + v_end];
                            if is_shared_string {
                                if let Ok(idx) = raw_v.parse::<usize>() {
                                    if let Some(s) = shared_strings.get(idx) {
                                        cell_value = s.clone();
                                    }
                                }
                            } else {
                                cell_value = raw_v.to_string();
                            }
                        }
                    }
                    cells.push(cell_value);
                    cell_search = &after_c[c_end + 4..];
                } else {
                    break;
                }
            }

            if !cells.is_empty() {
                raw_rows.push(cells);
            }
            search = &after_row[r_end + 6..];
        } else {
            break;
        }
    }

    if raw_rows.is_empty() {
        return Ok(SpreadsheetTable {
            name: sheet_name.to_string(),
            headers: Vec::new(),
            columns: Vec::new(),
            rows: Vec::new(),
            total_rows: 0,
        });
    }

    let headers = raw_rows.remove(0);
    let columns = SpreadsheetReader::infer_columns(&headers, &raw_rows);
    let total_rows = raw_rows.len();

    Ok(SpreadsheetTable {
        name: sheet_name.to_string(),
        headers,
        columns,
        rows: raw_rows,
        total_rows,
    })
}
