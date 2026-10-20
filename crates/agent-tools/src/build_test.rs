use crate::types::{Tool, ToolDefinition, ToolError, ToolExecutionResult, ToolResult};
use agent_platform::{LanguageToolingAdapter, MultiLanguageAdapter, ShellExecutor, ShellType};
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

/// Build Runner Tool: runs compiler/typechecker and extracts structured diagnostics
pub struct BuildRunnerTool {
    executor: Arc<dyn ShellExecutor>,
    security: SharedSecurityEngine,
}

impl BuildRunnerTool {
    pub fn new(executor: Arc<dyn ShellExecutor>, security: SharedSecurityEngine) -> Self {
        Self { executor, security }
    }
}

#[async_trait]
impl Tool for BuildRunnerTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "run_build".to_string(),
            description: "Executes project build or type check (cargo check, npm run build, python py_compile, etc.) and parses compiler errors.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Optional custom build command. If omitted, automatically detected from project type."
                    },
                    "path": {
                        "type": "string",
                        "description": "Optional subdirectory to run build from. Defaults to workspace root."
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
                PermissionTier::ShellExecution,
                "Execute project build",
                Some(&cwd),
                arguments.clone(),
            )
            .await?;

        let adapter = MultiLanguageAdapter::new();
        let cmd_string = if let Some(cmd) = arguments.get("command").and_then(|c| c.as_str()) {
            cmd.to_string()
        } else if let Some((cmd, args)) = adapter.build_command(&cwd) {
            format!("{cmd} {}", args.join(" "))
        } else {
            return Err(ToolError::InvalidParameters(
                "No build command specified and no supported build system detected in directory.".to_string(),
            ));
        };

        let out = self
            .executor
            .execute(ShellType::Cmd, &cmd_string, &cwd, &[], Some(60000))
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Build execution failed: {e}")))?;

        let report = adapter.parse_diagnostics(&out.stdout, &out.stderr);
        let success = out.exit_code == Some(0) && report.total_errors == 0;

        let result = json!({
            "command": cmd_string,
            "success": success,
            "exit_code": out.exit_code,
            "duration_ms": out.duration_ms,
            "total_errors": report.total_errors,
            "total_warnings": report.total_warnings,
            "root_cause": report.root_cause,
            "diagnostics": report.diagnostics,
            "stdout": out.stdout,
            "stderr": out.stderr,
        });

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "run_build".to_string(),
            stdout: serde_json::to_string_pretty(&result).unwrap_or_default(),
            stderr: out.stderr,
            exit_code: out.exit_code,
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: !success,
            truncated: false,
        })
    }
}

/// Test Runner Tool: executes tests and captures pass/fail output
pub struct TestRunnerTool {
    executor: Arc<dyn ShellExecutor>,
    security: SharedSecurityEngine,
}

impl TestRunnerTool {
    pub fn new(executor: Arc<dyn ShellExecutor>, security: SharedSecurityEngine) -> Self {
        Self { executor, security }
    }
}

#[async_trait]
impl Tool for TestRunnerTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "run_test".to_string(),
            description: "Executes unit or integration tests (cargo test, npm test, pytest, etc.) with optional filter.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "filter": {
                        "type": "string",
                        "description": "Optional name or filter pattern of specific test(s) to run."
                    },
                    "command": {
                        "type": "string",
                        "description": "Optional custom test command. If omitted, automatically detected from project type."
                    },
                    "path": {
                        "type": "string",
                        "description": "Optional subdirectory to run tests from."
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
                PermissionTier::ShellExecution,
                "Execute project tests",
                Some(&cwd),
                arguments.clone(),
            )
            .await?;

        let filter = arguments.get("filter").and_then(|f| f.as_str());
        let adapter = MultiLanguageAdapter::new();

        let cmd_string = if let Some(cmd) = arguments.get("command").and_then(|c| c.as_str()) {
            if let Some(f) = filter {
                format!("{cmd} {f}")
            } else {
                cmd.to_string()
            }
        } else if let Some((cmd, args)) = adapter.test_command(&cwd, filter) {
            format!("{cmd} {}", args.join(" "))
        } else {
            return Err(ToolError::InvalidParameters(
                "No test command specified and no supported test runner detected in directory.".to_string(),
            ));
        };

        let out = self
            .executor
            .execute(ShellType::Cmd, &cmd_string, &cwd, &[], Some(60000))
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Test execution failed: {e}")))?;

        let success = out.exit_code == Some(0);

        let result = json!({
            "command": cmd_string,
            "success": success,
            "exit_code": out.exit_code,
            "duration_ms": out.duration_ms,
            "stdout": out.stdout,
            "stderr": out.stderr,
        });

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "run_test".to_string(),
            stdout: serde_json::to_string_pretty(&result).unwrap_or_default(),
            stderr: out.stderr,
            exit_code: out.exit_code,
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: !success,
            truncated: false,
        })
    }
}
