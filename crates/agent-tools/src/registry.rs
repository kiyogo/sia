use crate::filesystem::{FileListTool, FilePatchTool, FileReadTool, FileSearchTool, FileWriteTool};
use crate::terminal::TerminalTool;
use crate::types::{Tool, ToolDefinition, ToolError, ToolExecutionResult, ToolResult};
use agent_platform::ShellExecutor;
use agent_security::SharedSecurityEngine;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn Tool>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, tool: Arc<dyn Tool>) {
        let name = tool.definition().name;
        let mut map = self.tools.write().await;
        map.insert(name, tool);
    }

    pub async fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let map = self.tools.read().await;
        map.get(name).cloned()
    }

    pub async fn list_definitions(&self) -> Vec<ToolDefinition> {
        let map = self.tools.read().await;
        map.values().map(|t| t.definition()).collect()
    }

    /// Smart tool selection: return only definitions for allowed tool names or categories
    pub async fn list_definitions_selective(&self, allowed_names: &[String]) -> Vec<ToolDefinition> {
        let map = self.tools.read().await;
        map.values()
            .map(|t| t.definition())
            .filter(|def| allowed_names.contains(&def.name))
            .collect()
    }

    pub async fn execute(
        &self,
        name: &str,
        call_id: &str,
        params: serde_json::Value,
    ) -> ToolResult<ToolExecutionResult> {
        let tool = self
            .get(name)
            .await
            .ok_or_else(|| ToolError::NotFound(format!("Tool '{}' is not registered", name)))?;

        tool.execute(call_id, params).await
    }
}

pub fn create_default_registry(
    shell_executor: Box<dyn ShellExecutor>,
    security: SharedSecurityEngine,
) -> ToolRegistry {
    let terminal_tool = Arc::new(TerminalTool::new(shell_executor, security.clone()));
    let file_read_tool = Arc::new(FileReadTool::new(security.clone()));
    let file_write_tool = Arc::new(FileWriteTool::new(security.clone()));
    let file_list_tool = Arc::new(FileListTool::new(security.clone()));
    let file_search_tool = Arc::new(FileSearchTool::new(security.clone()));
    let file_patch_tool = Arc::new(FilePatchTool::new(security.clone()));

    let mut map = HashMap::new();
    map.insert(terminal_tool.definition().name, terminal_tool as Arc<dyn Tool>);
    map.insert(file_read_tool.definition().name, file_read_tool as Arc<dyn Tool>);
    map.insert(file_write_tool.definition().name, file_write_tool as Arc<dyn Tool>);
    map.insert(file_list_tool.definition().name, file_list_tool as Arc<dyn Tool>);
    map.insert(file_search_tool.definition().name, file_search_tool as Arc<dyn Tool>);
    map.insert(file_patch_tool.definition().name, file_patch_tool as Arc<dyn Tool>);

    ToolRegistry {
        tools: RwLock::new(map),
    }
}

pub type SharedToolRegistry = Arc<ToolRegistry>;
