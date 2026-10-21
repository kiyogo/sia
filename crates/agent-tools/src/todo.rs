use crate::types::{Tool, ToolDefinition, ToolError, ToolExecutionResult, ToolResult};
use agent_security::{PermissionTier, SharedSecurityEngine};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub content: String,
    pub status: String, // "pending", "in_progress", "completed", "cancelled"
}

/// OpenCode-style `todowrite` Tool
/// Proactively creates and maintains a working plan / todo list for multi-step tasks.
pub struct TodoWriteTool {
    security: SharedSecurityEngine,
    current_todos: Arc<RwLock<Vec<TodoItem>>>,
}

impl TodoWriteTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self {
            security,
            current_todos: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_todos(&self) -> Vec<TodoItem> {
        self.current_todos.read().await.clone()
    }
}

#[async_trait]
impl Tool for TodoWriteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "todowrite".to_string(),
            description: "Create and update a todo list for the current session. The user sees it as your working plan. Use it proactively when a task has 3 or more distinct steps, or when the user gives several tasks. Send the full list on every call to replace the previous one. Mark items 'in_progress' when you start them and 'completed' as soon as they are done.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "todos": {
                        "type": "array",
                        "description": "The updated todo list with full item details.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "content": { "type": "string", "description": "Brief description of the task." },
                                "status": {
                                    "type": "string",
                                    "enum": ["pending", "in_progress", "completed", "cancelled"],
                                    "description": "Current status of the task."
                                }
                            },
                            "required": ["content", "status"]
                        }
                    }
                },
                "required": ["todos"]
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();

        self.security
            .check_permission(
                PermissionTier::ReadOnly,
                "Update session todos",
                None,
                params.clone(),
            )
            .await?;

        let todos_val = params.get("todos").ok_or_else(|| {
            ToolError::InvalidParameters("Missing 'todos' parameter in todowrite call".to_string())
        })?;

        let items: Vec<TodoItem> = serde_json::from_value(todos_val.clone()).map_err(|e| {
            ToolError::InvalidParameters(format!("Invalid todos schema: {e}"))
        })?;

        {
            let mut current = self.current_todos.write().await;
            *current = items.clone();
        }

        // Generate OpenCode style summary
        let total = items.len();
        let completed = items.iter().filter(|t| t.status == "completed").count();
        let in_progress: Vec<&str> = items
            .iter()
            .filter(|t| t.status == "in_progress")
            .map(|t| t.content.as_str())
            .collect();
        let pending = items.iter().filter(|t| t.status == "pending").count();
        let cancelled = items.iter().filter(|t| t.status == "cancelled").count();

        let mut summary_parts = Vec::new();
        summary_parts.push(format!("{completed}/{total} completed"));
        if !in_progress.is_empty() {
            summary_parts.push(format!("{} in progress", in_progress.len()));
        }
        if pending > 0 {
            summary_parts.push(format!("{pending} pending"));
        }
        if cancelled > 0 {
            summary_parts.push(format!("{cancelled} cancelled"));
        }

        let mut output = format!("Todos updated: {}", summary_parts.join(", "));
        if !in_progress.is_empty() {
            output.push_str(&format!(". Active: \"{}\"", in_progress.join("\", \"")));
        }

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "todowrite".to_string(),
            stdout: output,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}
