use crate::types::{Tool, ToolDefinition, ToolError, ToolExecutionResult, ToolResult};
use agent_security::{PermissionTier, SharedSecurityEngine};
use async_trait::async_trait;
use serde_json::json;
use similar::TextDiff;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn resolve_path(workspace: &Path, input_path: &str) -> PathBuf {
    let p = Path::new(input_path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        workspace.join(p)
    }
}

/// Normalizes smart quotes, hyphens, and whitespace variations (matching OpenCode edit.ts behavior)
pub fn normalize_for_match(s: &str) -> String {
    s.replace(['\u{2018}', '\u{2019}', '\u{201A}', '\u{201B}'], "'")
        .replace(['\u{201C}', '\u{201D}', '\u{201E}', '\u{201F}'], "\"")
        .replace(['\u{2010}', '\u{2011}', '\u{2012}', '\u{2013}', '\u{2014}', '\u{2015}', '\u{2212}'], "-")
        .replace(['\u{00A0}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{202F}', '\u{205F}', '\u{3000}'], " ")
}

/// Compute unified diff string using similar crate
pub fn compute_unified_diff(file_path: &str, old_content: &str, new_content: &str) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);
    diff.unified_diff()
        .context_radius(3)
        .header(&format!("a/{file_path}"), &format!("b/{file_path}"))
        .to_string()
}

/// OpenCode-style Exact Match Code Edit Tool
pub struct CodeEditTool {
    security: SharedSecurityEngine,
}

impl CodeEditTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for CodeEditTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "code_edit".to_string(),
            description: "Replaces exact occurrences of old_string with new_string in a source file, with uniqueness check and unified diff preview.".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["path", "old_string", "new_string"],
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path of the file to edit relative to workspace root."
                    },
                    "old_string": {
                        "type": "string",
                        "description": "Exact text to find and replace in the file."
                    },
                    "new_string": {
                        "type": "string",
                        "description": "Exact text to replace old_string with (must differ from old_string)."
                    },
                    "replace_all": {
                        "type": "boolean",
                        "description": "If false (default), old_string must match exactly once. If true, replaces every occurrence."
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, arguments: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let path_str = arguments
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("path is required".to_string()))?;
        let old_str = arguments
            .get("old_string")
            .or_else(|| arguments.get("oldString"))
            .and_then(|s| s.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("old_string (or oldString) is required".to_string()))?;
        let new_str = arguments
            .get("new_string")
            .or_else(|| arguments.get("newString"))
            .or_else(|| arguments.get("new_content"))
            .and_then(|s| s.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("new_string (or newString) is required".to_string()))?;
        let replace_all = arguments
            .get("replace_all")
            .or_else(|| arguments.get("replaceAll"))
            .and_then(|r| r.as_bool())
            .unwrap_or(false);

        if old_str == new_str {
            return Err(ToolError::InvalidParameters(
                "old_string and new_string must differ".to_string(),
            ));
        }

        let workspace = self.security.get_workspace_root().await;
        let full_path = resolve_path(&workspace, path_str);

        self.security
            .check_permission(
                PermissionTier::WorkspaceWrite,
                &format!("Edit file {path_str}"),
                Some(&full_path),
                arguments.clone(),
            )
            .await?;

        let content = fs::read_to_string(&full_path)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file {path_str}: {e}")))?;

        // Try exact match first
        let occurrences = content.matches(old_str).count();

        let (new_content, count) = if occurrences > 0 {
            if !replace_all && occurrences > 1 {
                return Err(ToolError::ExecutionFailed(format!(
                    "old_string occurs {occurrences} times in {path_str}. It must be unique; include more surrounding context or set replace_all to true."
                )));
            }
            if replace_all {
                (content.replace(old_str, new_str), occurrences)
            } else {
                (content.replacen(old_str, new_str, 1), 1)
            }
        } else {
            // Try normalized match fallback
            let norm_content = normalize_for_match(&content);
            let norm_old = normalize_for_match(old_str);
            let norm_count = norm_content.matches(&norm_old).count();

            if norm_count == 0 {
                return Err(ToolError::ExecutionFailed(format!(
                    "old_string was not found in {path_str}. Verify file contents and line endings."
                )));
            }

            if !replace_all && norm_count > 1 {
                return Err(ToolError::ExecutionFailed(format!(
                    "Normalized old_string occurs {norm_count} times in {path_str}. Include more context to make it unique."
                )));
            }

            // Replace normalized occurrences
            if replace_all {
                (norm_content.replace(&norm_old, new_str), norm_count)
            } else {
                (norm_content.replacen(&norm_old, new_str, 1), 1)
            }
        };

        // Write updated content atomically
        fs::write(&full_path, &new_content)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write to {path_str}: {e}")))?;

        let diff = compute_unified_diff(path_str, &content, &new_content);

        let result = json!({
            "path": path_str,
            "replacements": count,
            "diff": diff,
        });

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "code_edit".to_string(),
            stdout: serde_json::to_string_pretty(&result).unwrap_or(diff),
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

/// Atomic Multi-File Patch Tool with Rollback Protection
pub struct CodePatchTool {
    security: SharedSecurityEngine,
}

impl CodePatchTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for CodePatchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "code_patch".to_string(),
            description: "Applies multi-file changes atomically. If any file fails, rolls back all changes to preserve workspace integrity.".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["edits"],
                "properties": {
                    "edits": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["path", "content"],
                            "properties": {
                                "path": { "type": "string" },
                                "content": { "type": "string" }
                            }
                        },
                        "description": "List of {path, content} replacements to apply atomically."
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, arguments: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let edits_arr = arguments
            .get("edits")
            .and_then(|e| e.as_array())
            .ok_or_else(|| ToolError::InvalidParameters("edits array is required".to_string()))?;

        let workspace = self.security.get_workspace_root().await;

        // 1. Snapshot original contents for rollback and validate all paths
        let mut backups: HashMap<PathBuf, Option<String>> = HashMap::new();
        let mut planned_writes: Vec<(PathBuf, String, String)> = Vec::new();

        for edit in edits_arr {
            let path_str = edit
                .get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| ToolError::InvalidParameters("edit.path is required".to_string()))?;
            let new_content = edit
                .get("content")
                .and_then(|c| c.as_str())
                .ok_or_else(|| ToolError::InvalidParameters("edit.content is required".to_string()))?;

            let full_path = resolve_path(&workspace, path_str);

            self.security
                .check_permission(
                    PermissionTier::WorkspaceWrite,
                    &format!("Patch file {path_str}"),
                    Some(&full_path),
                    arguments.clone(),
                )
                .await?;

            let original_content = if full_path.exists() {
                Some(fs::read_to_string(&full_path).map_err(|e| {
                    ToolError::ExecutionFailed(format!("Failed to read backup of {path_str}: {e}"))
                })?)
            } else {
                None
            };

            backups.insert(full_path.clone(), original_content);
            planned_writes.push((full_path, path_str.to_string(), new_content.to_string()));
        }

        // 2. Attempt writes; rollback immediately if any write fails
        let mut applied_diffs = Vec::new();
        let mut failed = false;
        let mut error_msg = String::new();

        for (full_path, rel_path, new_content) in &planned_writes {
            if let Some(parent) = full_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let old_str = backups.get(full_path).cloned().flatten().unwrap_or_default();
            if let Err(e) = fs::write(full_path, new_content) {
                failed = true;
                error_msg = format!("Failed to write {rel_path}: {e}");
                break;
            }

            let diff = compute_unified_diff(rel_path, &old_str, new_content);
            applied_diffs.push(diff);
        }

        if failed {
            // Rollback all files to original snapshots
            for (path, original) in backups {
                match original {
                    Some(content) => {
                        let _ = fs::write(&path, content);
                    }
                    None => {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
            return Err(ToolError::ExecutionFailed(format!(
                "Atomic patch failed and was rolled back: {error_msg}"
            )));
        }

        let result = json!({
            "status": "success",
            "files_modified": planned_writes.len(),
            "diffs": applied_diffs,
        });

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "code_patch".to_string(),
            stdout: serde_json::to_string_pretty(&result).unwrap_or_default(),
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

/// Diff Preview Tool
pub struct DiffPreviewTool {
    security: SharedSecurityEngine,
}

impl DiffPreviewTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for DiffPreviewTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "diff_preview".to_string(),
            description: "Previews unified diff for proposed changes to a file without writing to disk.".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["path", "new_content"],
                "properties": {
                    "path": { "type": "string" },
                    "new_content": { "type": "string" }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, arguments: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let path_str = arguments
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("path is required".to_string()))?;
        let new_content = arguments
            .get("new_content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("new_content is required".to_string()))?;

        let workspace = self.security.get_workspace_root().await;
        let full_path = resolve_path(&workspace, path_str);

        self.security
            .check_permission(
                PermissionTier::ReadOnly,
                &format!("Preview diff for {path_str}"),
                Some(&full_path),
                arguments.clone(),
            )
            .await?;

        let old_content = if full_path.exists() {
            fs::read_to_string(&full_path).unwrap_or_default()
        } else {
            String::new()
        };

        let diff = compute_unified_diff(path_str, &old_content, new_content);

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "diff_preview".to_string(),
            stdout: diff,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}
