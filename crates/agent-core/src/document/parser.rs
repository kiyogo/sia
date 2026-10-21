use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentElementType {
    Heading { level: usize, text: String },
    Paragraph { text: String },
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    CodeBlock { language: Option<String>, code: String },
    List { items: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub format: String,
    pub title: Option<String>,
    pub elements: Vec<DocumentElementType>,
    pub raw_text: String,
    pub word_count: usize,
    pub page_count: Option<usize>,
}

pub struct DocumentParser;

impl DocumentParser {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<ParsedDocument, String> {
        let path_ref = path.as_ref();
        let ext = path_ref
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let bytes = std::fs::read(path_ref).map_err(|e| format!("Failed to read file: {}", e))?;

        match ext.as_str() {
            "docx" => Self::parse_docx(&bytes),
            "pdf" => Self::parse_pdf(&bytes),
            "md" | "markdown" => {
                let text = String::from_utf8_lossy(&bytes).to_string();
                Self::parse_markdown(&text)
            }
            "html" | "htm" => {
                let text = String::from_utf8_lossy(&bytes).to_string();
                Self::parse_html(&text)
            }
            "txt" | "log" | "csv" | "json" => {
                let text = String::from_utf8_lossy(&bytes).to_string();
                Self::parse_plain_text(&text)
            }
            _ => {
                // Fall back to plain text
                let text = String::from_utf8_lossy(&bytes).to_string();
                Self::parse_plain_text(&text)
            }
        }
    }

    pub fn parse_plain_text(text: &str) -> Result<ParsedDocument, String> {
        let mut elements = Vec::new();
        let paragraphs: Vec<&str> = text.split("\n\n").collect();

        for p in paragraphs {
            let trimmed = p.trim();
            if !trimmed.is_empty() {
                elements.push(DocumentElementType::Paragraph {
                    text: trimmed.to_string(),
                });
            }
        }

        let word_count = text.split_whitespace().count();
        Ok(ParsedDocument {
            format: "TXT".to_string(),
            title: text.lines().next().map(|l| l.trim().to_string()),
            elements,
            raw_text: text.to_string(),
            word_count,
            page_count: Some(1.max((word_count + 499) / 500)),
        })
    }

    pub fn parse_markdown(text: &str) -> Result<ParsedDocument, String> {
        let mut elements = Vec::new();
        let mut title = None;
        let mut current_table_headers = Vec::new();
        let mut current_table_rows = Vec::new();
        let mut in_table = false;

        for line in text.lines() {
            let trimmed = line.trim();

            // Check for Markdown table row
            if trimmed.starts_with('|') && trimmed.ends_with('|') {
                let cells: Vec<String> = trimmed
                    .split('|')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.trim().to_string())
                    .collect();

                // Check if separator row (e.g. |---|---|)
                if cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':')) {
                    continue;
                }

                if !in_table {
                    in_table = true;
                    current_table_headers = cells;
                } else {
                    current_table_rows.push(cells);
                }
                continue;
            } else if in_table {
                // Table ended
                elements.push(DocumentElementType::Table {
                    headers: current_table_headers.clone(),
                    rows: current_table_rows.clone(),
                });
                current_table_headers.clear();
                current_table_rows.clear();
                in_table = false;
            }

            // Headings
            if trimmed.starts_with("# ") {
                let heading_text = trimmed[2..].trim().to_string();
                if title.is_none() {
                    title = Some(heading_text.clone());
                }
                elements.push(DocumentElementType::Heading {
                    level: 1,
                    text: heading_text,
                });
            } else if trimmed.starts_with("## ") {
                elements.push(DocumentElementType::Heading {
                    level: 2,
                    text: trimmed[3..].trim().to_string(),
                });
            } else if trimmed.starts_with("### ") {
                elements.push(DocumentElementType::Heading {
                    level: 3,
                    text: trimmed[4..].trim().to_string(),
                });
            } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                elements.push(DocumentElementType::List {
                    items: vec![trimmed[2..].trim().to_string()],
                });
            } else if !trimmed.is_empty() {
                elements.push(DocumentElementType::Paragraph {
                    text: trimmed.to_string(),
                });
            }
        }

        if in_table {
            elements.push(DocumentElementType::Table {
                headers: current_table_headers,
                rows: current_table_rows,
            });
        }

        let word_count = text.split_whitespace().count();
        Ok(ParsedDocument {
            format: "MD".to_string(),
            title,
            elements,
            raw_text: text.to_string(),
            word_count,
            page_count: Some(1.max((word_count + 499) / 500)),
        })
    }

    pub fn parse_html(html: &str) -> Result<ParsedDocument, String> {
        // Lightweight regex-free HTML content extraction
        let mut plain = String::new();
        let mut in_tag = false;

        for c in html.chars() {
            if c == '<' {
                in_tag = true;
            } else if c == '>' {
                in_tag = false;
                plain.push(' ');
            } else if !in_tag {
                plain.push(c);
            }
        }

        let clean_text = plain
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        Self::parse_plain_text(&clean_text)
    }

    pub fn parse_docx(bytes: &[u8]) -> Result<ParsedDocument, String> {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| format!("Invalid DOCX zip archive: {}", e))?;

        let mut xml_content = String::new();
        {
            let mut file = archive
                .by_name("word/document.xml")
                .map_err(|e| format!("Missing word/document.xml in DOCX: {}", e))?;
            file.read_to_string(&mut xml_content)
                .map_err(|e| format!("Failed to read word/document.xml: {}", e))?;
        }

        let mut elements = Vec::new();
        let mut raw_text = String::new();
        let mut title = None;

        // Extract paragraphs and tables from OpenXML
        // OpenXML paragraphs are enclosed in <w:p>...</w:p>
        // OpenXML tables are enclosed in <w:tbl>...</w:tbl>
        let p_end = "</w:p>";

        // Simple XML tag stripper for text inside <w:t>...</w:t>
        let extract_wt = |segment: &str| -> String {
            let mut text = String::new();
            let mut search_idx = 0;
            while let Some(start) = segment[search_idx..].find("<w:t") {
                let actual_start = search_idx + start;
                if let Some(close_tag) = segment[actual_start..].find('>') {
                    let text_start = actual_start + close_tag + 1;
                    if let Some(end) = segment[text_start..].find("</w:t>") {
                        let text_end = text_start + end;
                        text.push_str(&segment[text_start..text_end]);
                        search_idx = text_end + 6;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            text
        };

        // Extract tables first
        let mut remaining = xml_content.as_str();
        while let Some(tbl_start) = remaining.find("<w:tbl>") {
            let before_tbl = &remaining[..tbl_start];
            // Extract paragraphs before table
            for p_chunk in before_tbl.split(p_end) {
                let text = extract_wt(p_chunk);
                if !text.is_empty() {
                    if title.is_none() {
                        title = Some(text.clone());
                    }
                    elements.push(DocumentElementType::Paragraph { text: text.clone() });
                    raw_text.push_str(&text);
                    raw_text.push('\n');
                }
            }

            if let Some(tbl_end) = remaining[tbl_start..].find("</w:tbl>") {
                let tbl_xml = &remaining[tbl_start..tbl_start + tbl_end];
                let mut rows = Vec::new();
                for tr_chunk in tbl_xml.split("</w:tr>") {
                    let mut cells = Vec::new();
                    for tc_chunk in tr_chunk.split("</w:tc>") {
                        let cell_text = extract_wt(tc_chunk);
                        if !cell_text.is_empty() {
                            cells.push(cell_text);
                        }
                    }
                    if !cells.is_empty() {
                        rows.push(cells);
                    }
                }

                if !rows.is_empty() {
                    let headers = rows.remove(0);
                    elements.push(DocumentElementType::Table { headers, rows });
                }

                remaining = &remaining[tbl_start + tbl_end + 8..];
            } else {
                break;
            }
        }

        // Extract any remaining paragraphs
        for p_chunk in remaining.split(p_end) {
            let text = extract_wt(p_chunk);
            if !text.is_empty() {
                if title.is_none() {
                    title = Some(text.clone());
                }
                elements.push(DocumentElementType::Paragraph { text: text.clone() });
                raw_text.push_str(&text);
                raw_text.push('\n');
            }
        }

        let word_count = raw_text.split_whitespace().count();
        Ok(ParsedDocument {
            format: "DOCX".to_string(),
            title,
            elements,
            raw_text,
            word_count,
            page_count: Some(1.max((word_count + 499) / 500)),
        })
    }

    pub fn parse_pdf(bytes: &[u8]) -> Result<ParsedDocument, String> {
        // PDF parser extracting text streams enclosed between BT (Begin Text) and ET (End Text)
        // and string literals ( ... ) Tj or [ ... ] TJ
        let mut raw_text = String::new();
        let mut elements = Vec::new();

        let s = String::from_utf8_lossy(bytes);
        let mut search_idx = 0;

        while let Some(bt_idx) = s[search_idx..].find("BT") {
            let actual_bt = search_idx + bt_idx;
            if let Some(et_idx) = s[actual_bt..].find("ET") {
                let actual_et = actual_bt + et_idx;
                let text_block = &s[actual_bt..actual_et];

                // Extract parenthesized strings: (Hello World) Tj
                let mut in_paren = false;
                let mut current_literal = String::new();
                for ch in text_block.chars() {
                    if ch == '(' && !in_paren {
                        in_paren = true;
                    } else if ch == ')' && in_paren {
                        in_paren = false;
                        if !current_literal.trim().is_empty() {
                            raw_text.push_str(&current_literal);
                            raw_text.push(' ');
                            elements.push(DocumentElementType::Paragraph {
                                text: current_literal.trim().to_string(),
                            });
                        }
                        current_literal.clear();
                    } else if in_paren {
                        current_literal.push(ch);
                    }
                }

                search_idx = actual_et + 2;
            } else {
                break;
            }
        }

        // If no BT/ET blocks found (e.g. uncompressed plain streams or fallback), scan for ASCII strings
        if raw_text.is_empty() {
            for line in s.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('(') && trimmed.ends_with(") Tj") {
                    let content = &trimmed[1..trimmed.len() - 4];
                    raw_text.push_str(content);
                    raw_text.push('\n');
                    elements.push(DocumentElementType::Paragraph {
                        text: content.to_string(),
                    });
                }
            }
        }

        let word_count = raw_text.split_whitespace().count();
        let page_count = s.matches("/Type /Page").count();

        Ok(ParsedDocument {
            format: "PDF".to_string(),
            title: elements.first().and_then(|e| match e {
                DocumentElementType::Heading { text, .. } => Some(text.clone()),
                DocumentElementType::Paragraph { text } => Some(text.clone()),
                _ => None,
            }),
            elements,
            raw_text,
            word_count,
            page_count: Some(1.max(page_count)),
        })
    }
}
