use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerifyCondition {
    UrlMatches { expected_pattern: String },
    ElementVisible { selector_or_text: String },
    FileExists { path: PathBuf, min_size: Option<u64> },
    WindowActive { expected_title: String },
    ExitCodeZero,
    OutputContains { expected_substring: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationOutcome {
    Passed { details: String },
    Failed { reason: String, retryable: bool },
}

impl VerificationOutcome {
    pub fn is_passed(&self) -> bool {
        matches!(self, VerificationOutcome::Passed { .. })
    }
}

pub struct ActionVerifier;

impl ActionVerifier {
    pub fn new() -> Self {
        Self
    }

    pub fn verify(&self, condition: &VerifyCondition, context_info: &serde_json::Value) -> VerificationOutcome {
        match condition {
            VerifyCondition::UrlMatches { expected_pattern } => {
                let current_url = context_info
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if current_url.contains(expected_pattern) {
                    VerificationOutcome::Passed {
                        details: format!("URL verified: '{}' matches expected pattern '{}'", current_url, expected_pattern),
                    }
                } else {
                    VerificationOutcome::Failed {
                        reason: format!("URL mismatch: expected pattern '{}', got '{}'", expected_pattern, current_url),
                        retryable: true,
                    }
                }
            }
            VerifyCondition::ElementVisible { selector_or_text } => {
                let dom_or_text = context_info
                    .get("snapshot")
                    .or_else(|| context_info.get("stdout"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if dom_or_text.contains(selector_or_text) {
                    VerificationOutcome::Passed {
                        details: format!("Target element / text '{}' found in UI snapshot", selector_or_text),
                    }
                } else {
                    VerificationOutcome::Failed {
                        reason: format!("Target element / text '{}' not visible in UI state", selector_or_text),
                        retryable: true,
                    }
                }
            }
            VerifyCondition::FileExists { path, min_size } => {
                if !path.exists() {
                    return VerificationOutcome::Failed {
                        reason: format!("Expected file does not exist: {}", path.display()),
                        retryable: true,
                    };
                }

                if let Some(min_bytes) = min_size {
                    match std::fs::metadata(path) {
                        Ok(meta) if meta.len() >= *min_bytes => {
                            VerificationOutcome::Passed {
                                details: format!("File '{}' exists with valid size {} bytes", path.display(), meta.len()),
                            }
                        }
                        Ok(meta) => {
                            VerificationOutcome::Failed {
                                reason: format!("File '{}' exists but size ({} bytes) is less than required {} bytes", path.display(), meta.len(), min_bytes),
                                retryable: true,
                            }
                        }
                        Err(e) => {
                            VerificationOutcome::Failed {
                                reason: format!("Cannot read metadata for file '{}': {e}", path.display()),
                                retryable: false,
                            }
                        }
                    }
                } else {
                    VerificationOutcome::Passed {
                        details: format!("File '{}' exists on disk", path.display()),
                    }
                }
            }
            VerifyCondition::WindowActive { expected_title } => {
                let active_window = context_info
                    .get("active_window")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if active_window.to_lowercase().contains(&expected_title.to_lowercase()) {
                    VerificationOutcome::Passed {
                        details: format!("Active foreground window '{}' matches target '{}'", active_window, expected_title),
                    }
                } else {
                    VerificationOutcome::Failed {
                        reason: format!("Active window mismatch: expected '{}', got '{}'", expected_title, active_window),
                        retryable: true,
                    }
                }
            }
            VerifyCondition::ExitCodeZero => {
                let exit_code = context_info
                    .get("exit_code")
                    .and_then(|v| v.as_i64());

                if exit_code == Some(0) {
                    VerificationOutcome::Passed {
                        details: "Process exited successfully with code 0".to_string(),
                    }
                } else {
                    VerificationOutcome::Failed {
                        reason: format!("Process exited with non-zero code {:?}", exit_code),
                        retryable: false,
                    }
                }
            }
            VerifyCondition::OutputContains { expected_substring } => {
                let stdout = context_info
                    .get("stdout")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if stdout.contains(expected_substring) {
                    VerificationOutcome::Passed {
                        details: format!("Output contains expected substring '{}'", expected_substring),
                    }
                } else {
                    VerificationOutcome::Failed {
                        reason: format!("Output does not contain expected substring '{}'", expected_substring),
                        retryable: true,
                    }
                }
            }
        }
    }
}

impl Default for ActionVerifier {
    fn default() -> Self {
        Self::new()
    }
}
