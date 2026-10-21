use std::io::Write;
use std::path::Path;
use zip::write::SimpleFileOptions;

use super::parser::DocumentElementType;

#[derive(Debug, Clone)]
pub struct DocumentContent {
    pub title: String,
    pub elements: Vec<DocumentElementType>,
}

pub struct DocumentGenerator;

impl DocumentGenerator {
    /// Generates an ECMA-376 OpenXML Word Document (.docx)
    pub fn generate_docx<P: AsRef<Path>>(content: &DocumentContent, output_path: P) -> Result<(), String> {
        let file = std::fs::File::create(output_path).map_err(|e| format!("Failed to create DOCX file: {}", e))?;
        let mut zip = zip::ZipWriter::new(file);

        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        // 1. [Content_Types].xml
        zip.start_file("[Content_Types].xml", options)
            .map_err(|e| format!("Zip error: {}", e))?;
        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;
        zip.write_all(content_types.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        // 2. _rels/.rels
        zip.start_file("_rels/.rels", options)
            .map_err(|e| format!("Zip error: {}", e))?;
        let root_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
        zip.write_all(root_rels.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        // 3. word/_rels/document.xml.rels
        zip.start_file("word/_rels/document.xml.rels", options)
            .map_err(|e| format!("Zip error: {}", e))?;
        let doc_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#;
        zip.write_all(doc_rels.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        // 4. word/document.xml
        zip.start_file("word/document.xml", options)
            .map_err(|e| format!("Zip error: {}", e))?;

        let mut doc_xml = String::new();
        doc_xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
"#);

        // Title
        doc_xml.push_str(&format!(
            r#"    <w:p>
      <w:pPr><w:jc w:val="center"/></w:pPr>
      <w:r><w:rPr><w:b/><w:sz w:val="36"/></w:rPr><w:t>{}</w:t></w:r>
    </w:p>
"#,
            escape_xml(&content.title)
        ));

        // Elements
        for el in &content.elements {
            match el {
                DocumentElementType::Heading { level, text } => {
                    let sz = match level {
                        1 => 32,
                        2 => 28,
                        _ => 24,
                    };
                    doc_xml.push_str(&format!(
                        r#"    <w:p>
      <w:r><w:rPr><w:b/><w:sz w:val="{}"/></w:rPr><w:t>{}</w:t></w:r>
    </w:p>
"#,
                        sz,
                        escape_xml(text)
                    ));
                }
                DocumentElementType::Paragraph { text } => {
                    doc_xml.push_str(&format!(
                        r#"    <w:p>
      <w:r><w:t>{}</w:t></w:r>
    </w:p>
"#,
                        escape_xml(text)
                    ));
                }
                DocumentElementType::Table { headers, rows } => {
                    doc_xml.push_str("    <w:tbl>\n");
                    // Header row
                    doc_xml.push_str("      <w:tr>\n");
                    for h in headers {
                        doc_xml.push_str(&format!(
                            r#"        <w:tc>
          <w:p><w:r><w:rPr><w:b/></w:rPr><w:t>{}</w:t></w:r></w:p>
        </w:tc>
"#,
                            escape_xml(h)
                        ));
                    }
                    doc_xml.push_str("      </w:tr>\n");

                    // Data rows
                    for row in rows {
                        doc_xml.push_str("      <w:tr>\n");
                        for cell in row {
                            doc_xml.push_str(&format!(
                                r#"        <w:tc>
          <w:p><w:r><w:t>{}</w:t></w:r></w:p>
        </w:tc>
"#,
                                escape_xml(cell)
                            ));
                        }
                        doc_xml.push_str("      </w:tr>\n");
                    }
                    doc_xml.push_str("    </w:tbl>\n");
                }
                DocumentElementType::List { items } => {
                    for item in items {
                        doc_xml.push_str(&format!(
                            r#"    <w:p>
      <w:r><w:t>• {}</w:t></w:r>
    </w:p>
"#,
                            escape_xml(item)
                        ));
                    }
                }
                DocumentElementType::CodeBlock { code, .. } => {
                    doc_xml.push_str(&format!(
                        r#"    <w:p>
      <w:r><w:rPr><w:rFonts w:ascii="Courier New"/></w:rPr><w:t>{}</w:t></w:r>
    </w:p>
"#,
                        escape_xml(code)
                    ));
                }
            }
        }

        doc_xml.push_str("  </w:body>\n</w:document>");
        zip.write_all(doc_xml.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        zip.finish().map_err(|e| format!("Zip finish error: {}", e))?;
        Ok(())
    }

    /// Generates a standard PDF 1.4 document stream
    pub fn generate_pdf<P: AsRef<Path>>(content: &DocumentContent, output_path: P) -> Result<(), String> {
        let mut pdf_data = Vec::new();
        let mut offsets = Vec::new();

        // 1. Header
        pdf_data.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

        // Object 1: Catalog
        offsets.push(pdf_data.len());
        pdf_data.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        // Object 2: Pages tree
        offsets.push(pdf_data.len());
        pdf_data.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

        // Object 3: Page
        offsets.push(pdf_data.len());
        pdf_data.extend_from_slice(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n",
        );

        // Object 4: Content Stream
        let mut stream = String::new();
        stream.push_str("BT\n/F1 16 Tf\n50 740 Td\n");
        stream.push_str(&format!("({}) Tj\n", escape_pdf(&content.title)));

        let mut y_pos = 700;
        for el in &content.elements {
            match el {
                DocumentElementType::Heading { text, .. } => {
                    stream.push_str(&format!("/F1 14 Tf\n1 0 0 1 50 {} Tm\n({}) Tj\n", y_pos, escape_pdf(text)));
                    y_pos -= 25;
                }
                DocumentElementType::Paragraph { text } => {
                    stream.push_str(&format!("/F1 11 Tf\n1 0 0 1 50 {} Tm\n({}) Tj\n", y_pos, escape_pdf(text)));
                    y_pos -= 18;
                }
                DocumentElementType::Table { headers, rows } => {
                    let h_str = headers.join(" | ");
                    stream.push_str(&format!("/F1 10 Tf\n1 0 0 1 50 {} Tm\n({}) Tj\n", y_pos, escape_pdf(&h_str)));
                    y_pos -= 15;
                    for r in rows {
                        let r_str = r.join(" | ");
                        stream.push_str(&format!("/F1 10 Tf\n1 0 0 1 50 {} Tm\n({}) Tj\n", y_pos, escape_pdf(&r_str)));
                        y_pos -= 14;
                    }
                }
                DocumentElementType::List { items } => {
                    for item in items {
                        stream.push_str(&format!("/F1 11 Tf\n1 0 0 1 65 {} Tm\n(- {}) Tj\n", y_pos, escape_pdf(item)));
                        y_pos -= 16;
                    }
                }
                DocumentElementType::CodeBlock { code, .. } => {
                    stream.push_str(&format!("/F1 9 Tf\n1 0 0 1 50 {} Tm\n({}) Tj\n", y_pos, escape_pdf(code)));
                    y_pos -= 16;
                }
            }
            if y_pos < 60 {
                break; // 1-page fit
            }
        }
        stream.push_str("ET\n");

        offsets.push(pdf_data.len());
        let stream_bytes = stream.as_bytes();
        pdf_data.extend_from_slice(
            format!("4 0 obj\n<< /Length {} >>\nstream\n", stream_bytes.len()).as_bytes(),
        );
        pdf_data.extend_from_slice(stream_bytes);
        pdf_data.extend_from_slice(b"\nendstream\nendobj\n");

        // Object 5: Standard Helvetica Font
        offsets.push(pdf_data.len());
        pdf_data.extend_from_slice(b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n");

        // Xref table
        let xref_offset = pdf_data.len();
        pdf_data.extend_from_slice(format!("xref\n0 {}\n", offsets.len() + 1).as_bytes());
        pdf_data.extend_from_slice(b"0000000000 65535 f \n");
        for off in &offsets {
            pdf_data.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
        }

        // Trailer
        pdf_data.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
                offsets.len() + 1,
                xref_offset
            )
            .as_bytes(),
        );

        std::fs::write(output_path, pdf_data).map_err(|e| format!("Failed to write PDF: {}", e))?;
        Ok(())
    }

    /// Generates Markdown string
    pub fn generate_markdown(content: &DocumentContent) -> String {
        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", content.title));

        for el in &content.elements {
            match el {
                DocumentElementType::Heading { level, text } => {
                    let hashes = "#".repeat(*level);
                    md.push_str(&format!("{} {}\n\n", hashes, text));
                }
                DocumentElementType::Paragraph { text } => {
                    md.push_str(&format!("{}\n\n", text));
                }
                DocumentElementType::Table { headers, rows } => {
                    md.push_str(&format!("| {} |\n", headers.join(" | ")));
                    let seps: Vec<String> = headers.iter().map(|_| "---".to_string()).collect();
                    md.push_str(&format!("| {} |\n", seps.join(" | ")));
                    for r in rows {
                        md.push_str(&format!("| {} |\n", r.join(" | ")));
                    }
                    md.push_str("\n");
                }
                DocumentElementType::List { items } => {
                    for it in items {
                        md.push_str(&format!("- {}\n", it));
                    }
                    md.push_str("\n");
                }
                DocumentElementType::CodeBlock { language, code } => {
                    let lang = language.as_deref().unwrap_or("");
                    md.push_str(&format!("```{}\n{}\n```\n\n", lang, code));
                }
            }
        }
        md
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn escape_pdf(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}
