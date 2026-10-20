use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Security violation: {0}")]
    Security(#[from] agent_security::SecurityError),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("MCP client error: {0}")]
    Mcp(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type ToolResult<T> = Result<T, ToolError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub tool_call_id: String,
    pub tool_name: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub is_error: bool,
    pub truncated: bool,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn execute(
        &self,
        call_id: &str,
        params: serde_json::Value,
    ) -> ToolResult<ToolExecutionResult>;
}
