use crate::types::{Tool, ToolDefinition, ToolError, ToolExecutionResult, ToolResult};
use agent_platform::{ShellExecutor, ShellType};
use agent_security::{PermissionTier, SharedSecurityEngine};
use async_trait::async_trait;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

fn resolve_path(workspace: &Path, input_path: &str) -> PathBuf {
    let p = Path::new(input_path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        workspace.join(p)
    }
}

/// Git status tool: returns porcelain status, modified files, staged files, untracked files
pub struct GitStatusTool {
    executor: Arc<dyn ShellExecutor>,
    security: SharedSecurityEngine,
}

impl GitStatusTool {
    pub fn new(executor: Arc<dyn ShellExecutor>, security: SharedSecurityEngine) -> Self {
        Self { executor, security }
    }
}

#[async_trait]
impl Tool for GitStatusTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "git_status".to_string(),
            description: "Gets current Git working tree status, branch, staged and unstaged files.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Optional subdirectory or repo path. Defaults to workspace root."
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, arguments: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let path_str = arguments.get("path").and_then(|p| p.as_str()).unwrap_or("");
        let workspace = self.security.get_workspace_root().await;
        let cwd = if path_str.is_empty() {
            workspace.clone()
        } else {
            resolve_path(&workspace, path_str)
        };

        self.security
            .check_permission(
                PermissionTier::ReadOnly,
                "Check Git status",
                Some(&cwd),
                arguments.clone(),
            )
            .await?;

        let out = self
            .executor
            .execute(
                ShellType::Cmd,
                "git status --porcelain=v1 -b",
                &cwd,
                &[],
                Some(15000),
            )
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to run git status: {e}")))?;

        let stdout = out.stdout.trim().to_string();
        let stderr = out.stderr.trim().to_string();

        let mut branch = String::new();
        let mut modified = Vec::new();
        let mut untracked = Vec::new();
        let mut staged = Vec::new();
        let mut deleted = Vec::new();

        for line in stdout.lines() {
            if let Some(b) = line.strip_prefix("## ") {
                branch = b.to_string();
            } else if line.len() >= 3 {
                let x = &line[0..1];
                let y = &line[1..2];
                let file = line[3..].trim().to_string();

                if x == "?" && y == "?" {
                    untracked.push(file);
                } else {
                    if x == "M" || x == "A" || x == "D" {
                        staged.push(file.clone());
                    }
                    if y == "M" {
                        modified.push(file.clone());
                    } else if y == "D" {
                        deleted.push(file);
                    }
                }
            }
        }

        let is_clean = modified.is_empty() && untracked.is_empty() && staged.is_empty() && deleted.is_empty();

        let details = json!({
            "branch": branch,
            "is_clean": is_clean,
            "modified": modified,
            "staged": staged,
            "untracked": untracked,
            "deleted": deleted,
            "raw": stdout,
            "stderr": stderr
        });

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "git_status".to_string(),
            stdout: serde_json::to_string_pretty(&details).unwrap_or(stdout),
            stderr,
            exit_code: out.exit_code,
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: out.exit_code.map(|c| c != 0).unwrap_or(false),
            truncated: false,
        })
    }
}

/// Git diff tool: displays changes between commits or working tree
pub struct GitDiffTool {
    executor: Arc<dyn ShellExecutor>,
    security: SharedSecurityEngine,
}

impl GitDiffTool {
    pub fn new(executor: Arc<dyn ShellExecutor>, security: SharedSecurityEngine) -> Self {
        Self { executor, security }
    }
}

#[async_trait]
impl Tool for GitDiffTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "git_diff".to_string(),
            description: "Gets git unified diff for modified files, specific file, or staged changes.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "Optional specific file to diff."
                    },
                    "staged": {
                        "type": "boolean",
                        "description": "If true, diff staged changes (--staged). Default is false."
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, arguments: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let cwd = self.security.get_workspace_root().await;
        let file = arguments.get("file").and_then(|f| f.as_str()).unwrap_or("");
        let staged = arguments.get("staged").and_then(|s| s.as_bool()).unwrap_or(false);

        self.security
            .check_permission(
                PermissionTier::ReadOnly,
                "Check Git diff",
                Some(&cwd),
                arguments.clone(),
            )
            .await?;

        let mut cmd = "git diff".to_string();
        if staged {
            cmd.push_str(" --staged");
        }
        if !file.is_empty() {
            cmd.push_str(&format!(" -- \"{file}\""));
        }

        let out = self
            .executor
            .execute(ShellType::Cmd, &cmd, &cwd, &[], Some(15000))
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to run git diff: {e}")))?;

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "git_diff".to_string(),
            stdout: out.stdout,
            stderr: out.stderr,
            exit_code: out.exit_code,
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: out.exit_code.map(|c| c != 0).unwrap_or(false),
            truncated: out.truncated,
        })
    }
}

/// Git log tool
pub struct GitLogTool {
    executor: Arc<dyn ShellExecutor>,
    security: SharedSecurityEngine,
}

impl GitLogTool {
    pub fn new(executor: Arc<dyn ShellExecutor>, security: SharedSecurityEngine) -> Self {
        Self { executor, security }
    }
}

#[async_trait]
impl Tool for GitLogTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "git_log".to_string(),
            description: "Shows recent git commits in reverse chronological order.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Number of commits to return. Defaults to 10."
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, arguments: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let cwd = self.security.get_workspace_root().await;
        let limit = arguments.get("limit").and_then(|l| l.as_u64()).unwrap_or(10);

        self.security
            .check_permission(
                PermissionTier::ReadOnly,
                "Check Git log",
                Some(&cwd),
                arguments.clone(),
            )
            .await?;

        let cmd = format!("git log -n {limit} --oneline --decorate");
        let out = self
            .executor
            .execute(ShellType::Cmd, &cmd, &cwd, &[], Some(15000))
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to run git log: {e}")))?;

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "git_log".to_string(),
            stdout: out.stdout,
            stderr: out.stderr,
            exit_code: out.exit_code,
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: out.exit_code.map(|c| c != 0).unwrap_or(false),
            truncated: false,
        })
    }
}

/// Git commit tool: adds files and creates commit safely
pub struct GitCommitTool {
    executor: Arc<dyn ShellExecutor>,
    security: SharedSecurityEngine,
}

impl GitCommitTool {
    pub fn new(executor: Arc<dyn ShellExecutor>, security: SharedSecurityEngine) -> Self {
        Self { executor, security }
    }
}

#[async_trait]
impl Tool for GitCommitTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "git_commit".to_string(),
            description: "Stages modified files and creates a git commit with a commit message.".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["message"],
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Commit message describing the changes."
                    },
                    "files": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Specific files to stage. If empty or omitted, stages modified tracked files."
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, arguments: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let cwd = self.security.get_workspace_root().await;
        let message = arguments
            .get("message")
            .and_then(|m| m.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("message is required".to_string()))?;

        self.security
            .check_permission(
                PermissionTier::WorkspaceWrite,
                &format!("Create Git commit: {message}"),
                Some(&cwd),
                arguments.clone(),
            )
            .await?;

        // Stage files
        let files = arguments.get("files").and_then(|f| f.as_array());
        let stage_cmd = if let Some(arr) = files {
            if arr.is_empty() {
                "git add -u".to_string()
            } else {
                let targets = arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| format!("\"{s}\""))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("git add {targets}")
            }
        } else {
            "git add -u".to_string()
        };

        let add_out = self
            .executor
            .execute(ShellType::Cmd, &stage_cmd, &cwd, &[], Some(15000))
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("git add failed: {e}")))?;

        if add_out.exit_code != Some(0) {
            return Ok(ToolExecutionResult {
                tool_call_id: call_id.to_string(),
                tool_name: "git_commit".to_string(),
                stdout: add_out.stdout,
                stderr: format!("git add failed: {}", add_out.stderr),
                exit_code: add_out.exit_code,
                duration_ms: start.elapsed().as_millis() as u64,
                is_error: true,
                truncated: false,
            });
        }

        // Clean message quotes for cmd execution
        let safe_msg = message.replace('"', "\\\"");
        let commit_cmd = format!("git commit -m \"{safe_msg}\"");
        let commit_out = self
            .executor
            .execute(ShellType::Cmd, &commit_cmd, &cwd, &[], Some(15000))
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("git commit failed: {e}")))?;

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "git_commit".to_string(),
            stdout: commit_out.stdout,
            stderr: commit_out.stderr,
            exit_code: commit_out.exit_code,
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: commit_out.exit_code.map(|c| c != 0).unwrap_or(false),
            truncated: false,
        })
    }
}
