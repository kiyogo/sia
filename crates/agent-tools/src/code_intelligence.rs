use crate::types::{Tool, ToolDefinition, ToolError, ToolExecutionResult, ToolResult};
use agent_platform::{LanguageToolingAdapter, MultiLanguageAdapter};
use agent_security::{PermissionTier, SharedSecurityEngine};
use async_trait::async_trait;
use regex::Regex;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

fn resolve_path(workspace: &Path, input_path: &str) -> PathBuf {
    let p = Path::new(input_path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        workspace.join(p)
    }
}

/// Project detection tool
pub struct ProjectDetectTool {
    security: SharedSecurityEngine,
}

impl ProjectDetectTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for ProjectDetectTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "project_detect".to_string(),
            description: "Discovers project languages, build systems, test runners, entry points, and Git information.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory to inspect. Defaults to workspace root."
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, arguments: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let path_str = arguments.get("path").and_then(|p| p.as_str()).unwrap_or("");
        let workspace = self.security.get_workspace_root().await;
        let target_dir = if path_str.is_empty() {
            workspace.clone()
        } else {
            resolve_path(&workspace, path_str)
        };

        self.security
            .check_permission(
                PermissionTier::ReadOnly,
                "Detect project structure",
                Some(&target_dir),
                arguments.clone(),
            )
            .await?;

        let adapter = MultiLanguageAdapter::new();
        let info = adapter.detect_project(&target_dir);

        let result_json = match info {
            Some(i) => json!({
                "found": true,
                "root_path": i.root_path.to_string_lossy(),
                "primary_language": i.primary_language,
                "languages": i.languages,
                "build_systems": i.build_systems,
                "test_runners": i.test_runners,
                "entry_points": i.entry_points.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
                "is_git": i.is_git,
                "git_branch": i.git_branch,
            }),
            None => json!({
                "found": false,
                "message": "No recognized project structure or manifest found.",
                "root_path": target_dir.to_string_lossy()
            }),
        };

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "project_detect".to_string(),
            stdout: serde_json::to_string_pretty(&result_json).unwrap_or_default(),
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

/// Compact Repo Map Generator
pub struct RepoMapTool {
    security: SharedSecurityEngine,
}

impl RepoMapTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }

    pub fn should_skip_dir(name: &str) -> bool {
        matches!(
            name,
            ".git"
                | "target"
                | "node_modules"
                | ".venv"
                | "venv"
                | "dist"
                | "build"
                | "__pycache__"
                | ".idea"
                | ".vscode"
                | "vendor"
        )
    }

    fn is_code_file(path: &Path) -> bool {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "c" | "cpp" | "h" | "hpp" | "go" | "toml" | "json") => true,
            _ => false,
        }
    }

    fn extract_symbols(content: &str, ext: &str) -> Vec<String> {
        let mut symbols = Vec::new();
        let pattern = match ext {
            "rs" => r"(?m)^(?:pub\s+)?(?:async\s+)?(?:fn|struct|enum|trait|type)\s+([A-Za-z0-9_]+)",
            "ts" | "tsx" | "js" | "jsx" => {
                r"(?m)^(?:export\s+)?(?:async\s+)?(?:function|class|interface|type|const|let)\s+([A-Za-z0-9_]+)"
            }
            "py" => r"(?m)^(?:async\s+)?(?:def|class)\s+([A-Za-z0-9_]+)",
            "c" | "cpp" | "h" | "hpp" => {
                r"(?m)^(?:class|struct|enum|void|int|bool|double|float|[A-Za-z0-9_]+)\s+([A-Za-z0-9_]+)\s*\("
            }
            _ => return symbols,
        };

        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(content) {
                if let Some(sym) = cap.get(1) {
                    let s = sym.as_str();
                    if !symbols.contains(&s.to_string()) && symbols.len() < 8 {
                        symbols.push(s.to_string());
                    }
                }
            }
        }
        symbols
    }
}

#[async_trait]
impl Tool for RepoMapTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "repo_map".to_string(),
            description: "Generates a compact, symbol-aware architectural map of the codebase within token budget.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "max_chars": {
                        "type": "integer",
                        "description": "Maximum character budget for the generated repo map (default: 4000)."
                    },
                    "subdir": {
                        "type": "string",
                        "description": "Optional subdirectory to map instead of entire workspace."
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, arguments: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let max_chars = arguments.get("max_chars").and_then(|m| m.as_u64()).unwrap_or(4000) as usize;
        let subdir = arguments.get("subdir").and_then(|s| s.as_str()).unwrap_or("");
        let workspace = self.security.get_workspace_root().await;

        let base_dir = if subdir.is_empty() {
            workspace.clone()
        } else {
            resolve_path(&workspace, subdir)
        };

        self.security
            .check_permission(
                PermissionTier::ReadOnly,
                "Generate repo map",
                Some(&base_dir),
                arguments.clone(),
            )
            .await?;

        let mut lines = Vec::new();
        lines.push(format!("# Repo Map: {}", base_dir.display()));

        let mut current_chars = lines[0].len();
        let mut truncated = false;
        let mut file_count = 0;

        for entry in WalkDir::new(&base_dir)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let name = e.file_name().to_string_lossy();
                    !Self::should_skip_dir(&name)
                } else {
                    true
                }
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && Self::is_code_file(path) {
                file_count += 1;
                let rel = path
                    .strip_prefix(&base_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/");
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                // Read symbols
                let symbols = if let Ok(content) = fs::read_to_string(path) {
                    Self::extract_symbols(&content, ext)
                } else {
                    Vec::new()
                };

                let line = if symbols.is_empty() {
                    format!("- {rel}")
                } else {
                    format!("- {rel} ({})", symbols.join(", "))
                };

                if current_chars + line.len() + 1 > max_chars {
                    truncated = true;
                    lines.push("... [remaining files omitted due to token budget]".to_string());
                    break;
                }

                current_chars += line.len() + 1;
                lines.push(line);
            }
        }

        let output = lines.join("\n");
        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "repo_map".to_string(),
            stdout: output,
            stderr: format!("Scanned {file_count} files (truncated: {truncated})"),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated,
        })
    }
}

/// Fast code content search tool (ripgrep-like)
pub struct CodeSearchTool {
    security: SharedSecurityEngine,
}

impl CodeSearchTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for CodeSearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "code_search".to_string(),
            description: "Searches for regex or literal text patterns across code files in the workspace.".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["pattern"],
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regular expression or substring to search for."
                    },
                    "path": {
                        "type": "string",
                        "description": "Optional subdirectory to limit search to."
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of matching lines to return. Defaults to 50."
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, arguments: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let pattern_str = arguments
            .get("pattern")
            .and_then(|p| p.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("pattern is required".to_string()))?;
        let path_str = arguments.get("path").and_then(|p| p.as_str()).unwrap_or("");
        let max_results = arguments.get("max_results").and_then(|m| m.as_u64()).unwrap_or(50) as usize;
        let workspace = self.security.get_workspace_root().await;

        let base_dir = if path_str.is_empty() {
            workspace.clone()
        } else {
            resolve_path(&workspace, path_str)
        };

        self.security
            .check_permission(
                PermissionTier::ReadOnly,
                &format!("Search code for pattern: {pattern_str}"),
                Some(&base_dir),
                arguments.clone(),
            )
            .await?;

        let regex = Regex::new(pattern_str)
            .map_err(|e| ToolError::InvalidParameters(format!("Invalid regex '{pattern_str}': {e}")))?;

        let mut matches = Vec::new();
        let mut truncated = false;

        for entry in WalkDir::new(&base_dir)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let name = e.file_name().to_string_lossy();
                    !RepoMapTool::should_skip_dir(&name)
                } else {
                    true
                }
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = fs::read_to_string(path) {
                    let rel = path
                        .strip_prefix(&base_dir)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .replace('\\', "/");

                    for (idx, line) in content.lines().enumerate() {
                        if regex.is_match(line) {
                            matches.push(json!({
                                "file": rel,
                                "line": idx + 1,
                                "content": line.trim(),
                            }));
                            if matches.len() >= max_results {
                                truncated = true;
                                break;
                            }
                        }
                    }
                }
            }
            if truncated {
                break;
            }
        }

        let result = json!({
            "pattern": pattern_str,
            "total_matches": matches.len(),
            "truncated": truncated,
            "matches": matches,
        });

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "code_search".to_string(),
            stdout: serde_json::to_string_pretty(&result).unwrap_or_default(),
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated,
        })
    }
}

/// Symbol Outline Tool
pub struct FileOutlineTool {
    security: SharedSecurityEngine,
}

impl FileOutlineTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for FileOutlineTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_outline".to_string(),
            description: "Extracts an outline of symbols (functions, structs, classes, methods) in a specific source file.".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["file"],
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "Path of the source file to outline."
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, arguments: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let file_str = arguments
            .get("file")
            .and_then(|f| f.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("file is required".to_string()))?;

        let workspace = self.security.get_workspace_root().await;
        let path = resolve_path(&workspace, file_str);

        self.security
            .check_permission(
                PermissionTier::ReadOnly,
                &format!("Outline symbols in {file_str}"),
                Some(&path),
                arguments.clone(),
            )
            .await?;

        let content = fs::read_to_string(&path)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {e}")))?;

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let pattern = match ext {
            "rs" => r"(?m)^(\s*(?:pub\s+)?(?:async\s+)?(?:fn|struct|enum|trait|type|impl)\s+([A-Za-z0-9_]+))",
            "ts" | "tsx" | "js" | "jsx" => {
                r"(?m)^(\s*(?:export\s+)?(?:async\s+)?(?:function|class|interface|type)\s+([A-Za-z0-9_]+))"
            }
            "py" => r"(?m)^(\s*(?:async\s+)?(?:def|class)\s+([A-Za-z0-9_]+))",
            _ => r"(?m)^(\s*(?:class|struct|fn|def|function)\s+([A-Za-z0-9_]+))",
        };

        let mut symbols = Vec::new();
        if let Ok(re) = Regex::new(pattern) {
            for (idx, line) in content.lines().enumerate() {
                if let Some(cap) = re.captures(line) {
                    symbols.push(json!({
                        "line": idx + 1,
                        "declaration": line.trim(),
                        "name": cap.get(2).map(|m| m.as_str()).unwrap_or(""),
                    }));
                }
            }
        }

        let result = json!({
            "file": file_str,
            "symbols": symbols,
            "total": symbols.len(),
        });

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "file_outline".to_string(),
            stdout: serde_json::to_string_pretty(&result).unwrap_or_default(),
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}
