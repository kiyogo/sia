use crate::types::{Tool, ToolDefinition, ToolError, ToolExecutionResult, ToolResult};
use agent_platform::{ProcessOutput, ShellExecutor, ShellType};
use agent_security::{PermissionTier, SharedSecurityEngine};
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

pub struct TerminalTool {
    executor: Arc<dyn ShellExecutor>,
    security: SharedSecurityEngine,
}

impl TerminalTool {
    pub fn new(executor: Box<dyn ShellExecutor>, security: SharedSecurityEngine) -> Self {
        Self {
            executor: Arc::from(executor),
            security,
        }
    }

    pub fn new_arc(executor: Arc<dyn ShellExecutor>, security: SharedSecurityEngine) -> Self {
        Self { executor, security }
    }
}

#[async_trait]
impl Tool for TerminalTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "terminal_execute".to_string(),
            description: "Execute a command using Windows CMD or PowerShell with real-time output, exit code, and timeout control.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The command line string to execute."
                    },
                    "shell": {
                        "type": "string",
                        "enum": ["cmd", "powershell"],
                        "default": "powershell",
                        "description": "Shell to use: 'cmd' or 'powershell'."
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Working directory. Defaults to active workspace."
                    },
                    "timeout_ms": {
                        "type": "integer",
                        "default": 60000,
                        "description": "Timeout in milliseconds before cancelling."
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(
        &self,
        call_id: &str,
        params: serde_json::Value,
    ) -> ToolResult<ToolExecutionResult> {
        let command = params
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'command' parameter".to_string()))?;

        let shell_str = params
            .get("shell")
            .and_then(|v| v.as_str())
            .unwrap_or("powershell");
        let shell = match shell_str {
            "cmd" => ShellType::Cmd,
            _ => ShellType::PowerShell,
        };

        let timeout_ms = params.get("timeout_ms").and_then(|v| v.as_u64());

        let workspace_root = self.security.get_workspace_root().await;
        let cwd = match params.get("cwd").and_then(|v| v.as_str()) {
            Some(p) => PathBuf::from(p),
            None => workspace_root.clone(),
        };

        // Security check
        self.security
            .check_permission(
                PermissionTier::ShellExecution,
                command,
                Some(&cwd),
                params.clone(),
            )
            .await?;

        // Execute
        let output: ProcessOutput = self
            .executor
            .execute(shell, command, &cwd, &[], timeout_ms)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let is_error = output.exit_code != Some(0);

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "terminal_execute".to_string(),
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.exit_code,
            duration_ms: output.duration_ms,
            is_error,
            truncated: false,
        })
    }
}
