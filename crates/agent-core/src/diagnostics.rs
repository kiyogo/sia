use agent_platform::{DiagnosticMessage, DiagnosticReport, DiagnosticSeverity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicatedDiagnostics {
    pub root_cause: Option<DiagnosticMessage>,
    pub primary_errors: Vec<DiagnosticMessage>,
    pub warning_count: usize,
    pub total_errors: usize,
    pub summary_prompt: String,
}

/// Deduplicates compiler diagnostics to preserve LLM token context
pub fn deduplicate_diagnostics(report: &DiagnosticReport, max_errors: usize) -> DeduplicatedDiagnostics {
    let mut primary_errors = Vec::new();
    let mut seen_locations = std::collections::HashSet::new();

    for diag in &report.diagnostics {
        if diag.severity == DiagnosticSeverity::Error {
            // Group errors by file + line to avoid duplicate reports of the same error site
            let loc_key = format!("{}:{}", diag.file, diag.line);
            if !seen_locations.contains(&loc_key) {
                seen_locations.insert(loc_key);
                primary_errors.push(diag.clone());
                if primary_errors.len() >= max_errors {
                    break;
                }
            }
        }
    }

    // Build a compact, token-efficient prompt string
    let mut lines = Vec::new();
    if primary_errors.is_empty() {
        lines.push("No compiler errors detected.".to_string());
    } else {
        lines.push(format!(
            "Build failed with {} error(s) (showing top {} root causes):",
            report.total_errors,
            primary_errors.len()
        ));
        for (idx, err) in primary_errors.iter().enumerate() {
            let code_str = err.code.as_deref().map(|c| format!(" [{c}]")).unwrap_or_default();
            lines.push(format!(
                "{}. {}:{}:{} - {}{}",
                idx + 1,
                err.file,
                err.line,
                err.column,
                err.message,
                code_str
            ));
            if let Some(snippet) = &err.context_snippet {
                lines.push(format!("   Snippet:\n   {}", snippet.replace('\n', "\n   ")));
            }
        }
    }

    if report.total_warnings > 0 {
        lines.push(format!("({} warning(s) omitted)", report.total_warnings));
    }

    let summary_prompt = lines.join("\n");

    DeduplicatedDiagnostics {
        root_cause: report.root_cause.clone(),
        primary_errors,
        warning_count: report.total_warnings,
        total_errors: report.total_errors,
        summary_prompt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_deduplication() {
        let report = DiagnosticReport {
            diagnostics: vec![
                DiagnosticMessage {
                    file: "src/main.rs".to_string(),
                    line: 12,
                    column: 5,
                    severity: DiagnosticSeverity::Error,
                    message: "cannot find value `x`".to_string(),
                    code: Some("E0425".to_string()),
                    context_snippet: Some("x += 1;".to_string()),
                },
                // Duplicate at same line
                DiagnosticMessage {
                    file: "src/main.rs".to_string(),
                    line: 12,
                    column: 5,
                    severity: DiagnosticSeverity::Error,
                    message: "cannot find value `x` in this scope".to_string(),
                    code: Some("E0425".to_string()),
                    context_snippet: None,
                },
                // Another error
                DiagnosticMessage {
                    file: "src/lib.rs".to_string(),
                    line: 45,
                    column: 1,
                    severity: DiagnosticSeverity::Error,
                    message: "mismatched types".to_string(),
                    code: Some("E0308".to_string()),
                    context_snippet: None,
                },
                // Warning
                DiagnosticMessage {
                    file: "src/lib.rs".to_string(),
                    line: 2,
                    column: 1,
                    severity: DiagnosticSeverity::Warning,
                    message: "unused import".to_string(),
                    code: None,
                    context_snippet: None,
                },
            ],
            root_cause: Some(DiagnosticMessage {
                file: "src/main.rs".to_string(),
                line: 12,
                column: 5,
                severity: DiagnosticSeverity::Error,
                message: "cannot find value `x`".to_string(),
                code: Some("E0425".to_string()),
                context_snippet: Some("x += 1;".to_string()),
            }),
            total_errors: 3,
            total_warnings: 1,
            raw_output: String::new(),
        };

        let dedup = deduplicate_diagnostics(&report, 5);
        assert_eq!(dedup.primary_errors.len(), 2);
        assert!(dedup.summary_prompt.contains("cannot find value `x`"));
        assert!(dedup.summary_prompt.contains("mismatched types"));
        assert!(dedup.summary_prompt.contains("1 warning(s) omitted"));
    }
}
