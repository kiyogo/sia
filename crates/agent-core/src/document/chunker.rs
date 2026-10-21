use serde::{Deserialize, Serialize};
use super::parser::{DocumentElementType, ParsedDocument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub chunk_index: usize,
    pub heading_context: Option<String>,
    pub text: String,
    pub token_estimate: usize,
    pub element_types: Vec<String>,
}

pub struct DocumentChunker;

impl DocumentChunker {
    pub fn chunk_document(doc: &ParsedDocument, max_tokens_per_chunk: usize) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let mut current_heading: Option<String> = None;
        let mut current_text = String::new();
        let mut current_element_types = Vec::new();

        for el in &doc.elements {
            match el {
                DocumentElementType::Heading { level: _, text } => {
                    // Headings often indicate natural chunk boundaries if current text is substantial
                    if !current_text.is_empty() && (estimate_tokens(&current_text) > max_tokens_per_chunk / 2) {
                        chunks.push(DocumentChunk {
                            chunk_index: chunks.len(),
                            heading_context: current_heading.clone(),
                            token_estimate: estimate_tokens(&current_text),
                            text: current_text.trim().to_string(),
                            element_types: current_element_types.clone(),
                        });
                        current_text.clear();
                        current_element_types.clear();
                    }
                    current_heading = Some(text.clone());
                    current_text.push_str(&format!("\n# {}\n\n", text));
                    current_element_types.push("Heading".to_string());
                }
                DocumentElementType::Paragraph { text } => {
                    let tokens = estimate_tokens(text);
                    if estimate_tokens(&current_text) + tokens > max_tokens_per_chunk && !current_text.is_empty() {
                        chunks.push(DocumentChunk {
                            chunk_index: chunks.len(),
                            heading_context: current_heading.clone(),
                            token_estimate: estimate_tokens(&current_text),
                            text: current_text.trim().to_string(),
                            element_types: current_element_types.clone(),
                        });
                        current_text.clear();
                        current_element_types.clear();
                    }
                    current_text.push_str(text);
                    current_text.push_str("\n\n");
                    current_element_types.push("Paragraph".to_string());
                }
                DocumentElementType::Table { headers, rows } => {
                    let mut tbl_str = format!("| {} |\n", headers.join(" | "));
                    for r in rows {
                        tbl_str.push_str(&format!("| {} |\n", r.join(" | ")));
                    }
                    current_text.push_str(&tbl_str);
                    current_text.push('\n');
                    current_element_types.push("Table".to_string());
                }
                DocumentElementType::List { items } => {
                    for it in items {
                        current_text.push_str(&format!("- {}\n", it));
                    }
                    current_element_types.push("List".to_string());
                }
                DocumentElementType::CodeBlock { language, code } => {
                    current_text.push_str(&format!("```{:?}\n{}\n```\n", language, code));
                    current_element_types.push("CodeBlock".to_string());
                }
            }
        }

        if !current_text.trim().is_empty() {
            chunks.push(DocumentChunk {
                chunk_index: chunks.len(),
                heading_context: current_heading,
                token_estimate: estimate_tokens(&current_text),
                text: current_text.trim().to_string(),
                element_types: current_element_types,
            });
        }

        chunks
    }
}

fn estimate_tokens(text: &str) -> usize {
    // 1 token ~= 4 characters or 0.75 words
    (text.len() + 3) / 4
}
