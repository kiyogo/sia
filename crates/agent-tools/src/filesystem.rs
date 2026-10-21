use crate::types::{Tool, ToolDefinition, ToolError, ToolExecutionResult, ToolResult};
use agent_security::{PermissionTier, SharedSecurityEngine};
use async_trait::async_trait;
use serde_json::json;
use similar::TextDiff;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

// --- Helper functions ---
fn resolve_path(workspace: &Path, input_path: &str) -> PathBuf {
    let p = Path::new(input_path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        workspace.join(p)
    }
}

// ==========================================
// File Read Tool
// ==========================================
pub struct FileReadTool {
    security: SharedSecurityEngine,
}

impl FileReadTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for FileReadTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_read".to_string(),
            description: "Read content of a file within the workspace with line offset and limit control.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Relative or absolute path to the file." },
                    "offset": { "type": "integer", "description": "1-based starting line number (optional)." },
                    "limit": { "type": "integer", "description": "Maximum number of lines to read (optional)." }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let path_str = params.get("path").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'path' parameter".to_string()))?;

        let workspace = self.security.get_workspace_root().await;
        let resolved = resolve_path(&workspace, path_str);

        self.security.check_permission(PermissionTier::ReadOnly, &format!("Read {}", resolved.display()), Some(&resolved), params.clone()).await?;

        // OpenCode behavior: if path is a directory, list its contents with pagination
        if resolved.is_dir() {
            let mut entries = Vec::new();
            if let Ok(rd) = fs::read_dir(&resolved) {
                for item in rd.filter_map(|e| e.ok()) {
                    let name = item.file_name().to_string_lossy().to_string();
                    let is_d = item.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    let size = item.metadata().map(|m| m.len()).unwrap_or(0);
                    entries.push(format!("{} {} ({} bytes)", if is_d { "[DIR] " } else { "[FILE]" }, name, size));
                }
            }
            entries.sort();
            let total_entries = entries.len();
            let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(1).max(1) as usize - 1;
            let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(500) as usize;
            let end = (offset + limit).min(total_entries);
            let selected = if offset < total_entries { &entries[offset..end] } else { &[] };

            let mut output = format!("[Directory entries {}-{} of {} from {}]\n", offset + 1, end, total_entries, resolved.display());
            output.push_str(&selected.join("\n"));

            return Ok(ToolExecutionResult {
                tool_call_id: call_id.to_string(),
                tool_name: "file_read".to_string(),
                stdout: output,
                stderr: String::new(),
                exit_code: Some(0),
                duration_ms: start.elapsed().as_millis() as u64,
                is_error: false,
                truncated: end < total_entries,
            });
        }

        let content = fs::read_to_string(&resolved).map_err(|e| ToolError::ExecutionFailed(format!("Failed to read {}: {e}", resolved.display())))?;
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(1).max(1) as usize - 1;
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(500) as usize;

        let end = (offset + limit).min(total_lines);
        let selected_lines = if offset < total_lines {
            &lines[offset..end]
        } else {
            &[]
        };

        let mut output = String::new();
        output.push_str(&format!("[Lines {}-{} of {} from {}]\n", offset + 1, end, total_lines, resolved.display()));
        for (i, line) in selected_lines.iter().enumerate() {
            output.push_str(&format!("{:4} | {}\n", offset + i + 1, line));
        }

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "file_read".to_string(),
            stdout: output,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: end < total_lines,
        })
    }
}

// ==========================================
// File Write Tool
// ==========================================
pub struct FileWriteTool {
    security: SharedSecurityEngine,
}

impl FileWriteTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for FileWriteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_write".to_string(),
            description: "Create or overwrite a file in the workspace. Automatically creates parent directories.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Relative or absolute path to write." },
                    "content": { "type": "string", "description": "Full text content to write." },
                    "append": { "type": "boolean", "description": "Append to file if true, overwrite if false." }
                },
                "required": ["path", "content"]
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let path_str = params.get("path").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'path' parameter".to_string()))?;
        let content = params.get("content").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'content' parameter".to_string()))?;
        let append = params.get("append").and_then(|v| v.as_bool()).unwrap_or(false);

        let workspace = self.security.get_workspace_root().await;
        let resolved = resolve_path(&workspace, path_str);

        self.security.check_permission(
            PermissionTier::WorkspaceWrite,
            &format!("Write file {}", resolved.display()),
            Some(&resolved),
            params.clone(),
        ).await?;

        if let Some(parent) = resolved.parent() {
            fs::create_dir_all(parent)?;
        }

        let existed = resolved.exists();
        let bytes_written = content.as_bytes().len();

        if append {
            use std::io::Write;
            let mut file = fs::OpenOptions::new().create(true).append(true).open(&resolved)?;
            file.write_all(content.as_bytes())?;
        } else {
            fs::write(&resolved, content)?;
        }

        let summary = format!(
            "Successfully {} {} ({} bytes)",
            if append { "appended to" } else if existed { "overwrote" } else { "created" },
            resolved.display(),
            bytes_written
        );

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "file_write".to_string(),
            stdout: summary,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

// ==========================================
// File List Tool
// ==========================================
pub struct FileListTool {
    security: SharedSecurityEngine,
}

impl FileListTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for FileListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_list".to_string(),
            description: "List files and directories in the workspace with depth and metadata.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Subdirectory to list (optional, defaults to workspace root)." },
                    "max_depth": { "type": "integer", "default": 2, "description": "Maximum directory traversal depth." }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let workspace = self.security.get_workspace_root().await;
        let target_dir = match params.get("path").and_then(|v| v.as_str()) {
            Some(p) => resolve_path(&workspace, p),
            None => workspace.clone(),
        };

        self.security.check_permission(PermissionTier::ReadOnly, &format!("List directory {}", target_dir.display()), Some(&target_dir), params.clone()).await?;

        let max_depth = params.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(2) as usize;

        let mut entries = Vec::new();
        for entry in WalkDir::new(&target_dir).max_depth(max_depth).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path == target_dir {
                continue;
            }
            let rel = path.strip_prefix(&target_dir).unwrap_or(path);
            let is_dir = entry.file_type().is_dir();
            let size = if is_dir { 0 } else { entry.metadata().map(|m| m.len()).unwrap_or(0) };
            entries.push(format!("{} {} ({} bytes)", if is_dir { "[DIR] " } else { "[FILE]" }, rel.display(), size));
        }

        let output = if entries.is_empty() {
            format!("Directory {} is empty", target_dir.display())
        } else {
            format!("[Directory Listing of {}]\n{}", target_dir.display(), entries.join("\n"))
        };

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "file_list".to_string(),
            stdout: output,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

// ==========================================
// File Search Tool
// ==========================================
pub struct FileSearchTool {
    security: SharedSecurityEngine,
}

impl FileSearchTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for FileSearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_search".to_string(),
            description: "Search for files by name pattern or search text within file contents across the workspace.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search string or regex query." },
                    "content_search": { "type": "boolean", "default": false, "description": "If true, searches file contents instead of file names." },
                    "max_results": { "type": "integer", "default": 20, "description": "Maximum results to return." }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let query = params.get("query").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'query' parameter".to_string()))?;
        let content_search = params.get("content_search").and_then(|v| v.as_bool()).unwrap_or(false);
        let max_results = params.get("max_results").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

        let workspace = self.security.get_workspace_root().await;
        self.security.check_permission(PermissionTier::ReadOnly, &format!("Search in {}", workspace.display()), Some(&workspace), params.clone()).await?;

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for entry in WalkDir::new(&workspace).into_iter().filter_map(|e| e.ok()) {
            if results.len() >= max_results {
                break;
            }
            let path = entry.path();
            if path.is_file() {
                let rel = path.strip_prefix(&workspace).unwrap_or(path);
                if content_search {
                    if let Ok(content) = fs::read_to_string(path) {
                        for (i, line) in content.lines().enumerate() {
                            if line.to_lowercase().contains(&query_lower) {
                                results.push(format!("{}:{}: {}", rel.display(), i + 1, line.trim()));
                                if results.len() >= max_results {
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if file_name.to_lowercase().contains(&query_lower) {
                        results.push(rel.display().to_string());
                    }
                }
            }
        }

        let output = if results.is_empty() {
            format!("No matches found for '{query}' in {}", workspace.display())
        } else {
            results.join("\n")
        };

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "file_search".to_string(),
            stdout: output,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: results.len() >= max_results,
        })
    }
}

// ==========================================
// File Patch Tool
// ==========================================
pub struct FilePatchTool {
    security: SharedSecurityEngine,
}

impl FilePatchTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for FilePatchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_patch".to_string(),
            description: "Apply a targeted search-and-replace patch to a file with unified diff generation.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Relative or absolute path of file to patch." },
                    "target": { "type": "string", "description": "Exact text block to replace." },
                    "replacement": { "type": "string", "description": "Replacement text block." }
                },
                "required": ["path", "target", "replacement"]
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let path_str = params.get("path").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'path' parameter".to_string()))?;
        let target = params.get("target").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'target' parameter".to_string()))?;
        let replacement = params.get("replacement").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'replacement' parameter".to_string()))?;

        let workspace = self.security.get_workspace_root().await;
        let resolved = resolve_path(&workspace, path_str);

        self.security.check_permission(
            PermissionTier::WorkspaceWrite,
            &format!("Patch file {}", resolved.display()),
            Some(&resolved),
            params.clone(),
        ).await?;

        let original = fs::read_to_string(&resolved)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read {}: {e}", resolved.display())))?;

        if !original.contains(target) {
            return Err(ToolError::ExecutionFailed(format!(
                "Target string not found in {}",
                resolved.display()
            )));
        }

        let updated = original.replacen(target, replacement, 1);
        fs::write(&resolved, &updated)?;

        let diff = TextDiff::from_lines(&original, &updated);
        let diff_str = diff.unified_diff().context_radius(3).header("original", "modified").to_string();

        let stdout = format!("Patch applied to {}:\n\n{}", resolved.display(), diff_str);

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "file_patch".to_string(),
            stdout,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}
