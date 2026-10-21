use crate::mcp_client::McpClient;
use crate::types::{Tool, ToolDefinition, ToolError, ToolExecutionResult, ToolResult};
use agent_security::{PermissionTier, SharedSecurityEngine};
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::info;

pub struct BrowserMcpManager {
    client: Arc<RwLock<Option<McpClient>>>,
    security: SharedSecurityEngine,
}

impl BrowserMcpManager {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            security,
        }
    }

    pub async fn check_prerequisites(&self) -> (bool, String) {
        let mut node_cmd = Command::new("node");
        #[cfg(windows)]
        node_cmd.creation_flags(0x08000000);
        let node_check = node_cmd.arg("--version").output().await;

        let mut npx_cmd = Command::new("npx.cmd");
        #[cfg(windows)]
        npx_cmd.creation_flags(0x08000000);
        let npx_check = npx_cmd.arg("--version").output().await;

        match (node_check, npx_check) {
            (Ok(n), _) if n.status.success() => {
                let ver = String::from_utf8_lossy(&n.stdout).trim().to_string();
                (true, format!("Node.js ({ver}) and npx available for Playwright MCP."))
            }
            (_, Ok(x)) if x.status.success() => {
                let ver = String::from_utf8_lossy(&x.stdout).trim().to_string();
                (true, format!("npx ({ver}) available for Playwright MCP."))
            }
            _ => (
                false,
                "Node.js/npx not found in PATH. Install Node.js 18+ to run Playwright MCP.".to_string(),
            ),
        }
    }

    pub async fn is_connected(&self) -> bool {
        self.client.read().await.is_some()
    }

    pub async fn connect(
        &self,
        custom_command: Option<(&str, Vec<String>)>,
    ) -> ToolResult<Vec<ToolDefinition>> {
        let (ok, msg) = self.check_prerequisites().await;
        if !ok {
            return Err(ToolError::Mcp(msg));
        }

        let (cmd, args_vec) = if let Some((c, a)) = custom_command {
            (c.to_string(), a)
        } else {
            (
                "npx.cmd".to_string(),
                vec![
                    "-y".to_string(),
                    "@playwright/mcp".to_string(),
                    "--browser".to_string(),
                    "chromium".to_string(),
                    "--isolated".to_string(),
                ],
            )
        };

        let args_refs: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

        let client = McpClient::spawn(&cmd, &args_refs, &[]).await?;
        let discovered_tools = client.list_tools().await?;

        let mut definitions = Vec::new();
        for t in &discovered_tools {
            definitions.push(ToolDefinition {
                name: format!("playwright_{}", t.name),
                description: format!(
                    "[Playwright Browser Automation] {}",
                    t.description.as_deref().unwrap_or("Browser Tool")
                ),
                parameters: t.input_schema.clone(),
            });
        }

        let mut lock = self.client.write().await;
        *lock = Some(client);

        info!("Playwright MCP connected with {} tools registered", definitions.len());
        Ok(definitions)
    }

    pub async fn disconnect(&self) {
        let mut lock = self.client.write().await;
        if let Some(client) = lock.take() {
            client.shutdown().await;
            info!("Playwright MCP connection closed");
        }
    }

    pub async fn call_raw(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> ToolResult<serde_json::Value> {
        let lock = self.client.read().await;
        let client = lock.as_ref().ok_or_else(|| {
            ToolError::Mcp("Playwright MCP server is not connected. Call connect() first.".to_string())
        })?;

        client.call_tool(tool_name, arguments).await
    }

    pub async fn navigate(&self, url: &str) -> ToolResult<serde_json::Value> {
        self.call_raw("browser_navigate", json!({ "url": url })).await
    }

    pub async fn snapshot(&self) -> ToolResult<serde_json::Value> {
        self.call_raw("browser_snapshot", json!({})).await
    }

    pub async fn click(&self, selector: &str) -> ToolResult<serde_json::Value> {
        self.call_raw("browser_click", json!({ "selector": selector })).await
    }

    pub async fn fill(&self, selector: &str, value: &str) -> ToolResult<serde_json::Value> {
        self.call_raw(
            "browser_fill_form",
            json!({
                "selector": selector,
                "value": value
            }),
        )
        .await
    }

    pub async fn type_text(&self, text: &str) -> ToolResult<serde_json::Value> {
        self.call_raw("browser_type", json!({ "text": text })).await
    }

    pub async fn press_key(&self, key: &str) -> ToolResult<serde_json::Value> {
        self.call_raw("browser_press_key", json!({ "key": key })).await
    }

    pub async fn list_tabs(&self) -> ToolResult<serde_json::Value> {
        self.call_raw("browser_tabs", json!({ "action": "list" })).await
    }

    pub async fn upload_file(&self, selector: &str, file_path: &Path) -> ToolResult<serde_json::Value> {
        // Enforce boundary check
        let abs_path = if file_path.is_relative() {
            let root = self.security.get_workspace_root().await;
            root.join(file_path)
        } else {
            file_path.to_path_buf()
        };

        self.security
            .check_permission(
                PermissionTier::BrowserAutomation,
                &format!("Browser file upload: {}", abs_path.display()),
                Some(&abs_path),
                json!({ "selector": selector, "file": abs_path.display().to_string() }),
            )
            .await
            .map_err(ToolError::Security)?;

        self.call_raw(
            "browser_file_upload",
            json!({
                "selector": selector,
                "file_paths": [abs_path.to_string_lossy()]
            }),
        )
        .await
    }

    pub async fn take_screenshot(&self) -> ToolResult<serde_json::Value> {
        self.call_raw("browser_take_screenshot", json!({})).await
    }

    pub async fn close(&self) -> ToolResult<()> {
        if self.is_connected().await {
            let _ = self.call_raw("browser_close", json!({})).await;
            self.disconnect().await;
        }
        Ok(())
    }
}

pub struct DynamicBrowserMcpTool {
    manager: Arc<BrowserMcpManager>,
    raw_name: String,
    definition: ToolDefinition,
    security: SharedSecurityEngine,
}

impl DynamicBrowserMcpTool {
    pub fn new(
        manager: Arc<BrowserMcpManager>,
        raw_name: String,
        definition: ToolDefinition,
        security: SharedSecurityEngine,
    ) -> Self {
        Self {
            manager,
            raw_name,
            definition,
            security,
        }
    }
}

#[async_trait]
impl Tool for DynamicBrowserMcpTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(
        &self,
        call_id: &str,
        params: serde_json::Value,
    ) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();

        // Enforce browser automation permission check
        self.security
            .check_permission(
                PermissionTier::BrowserAutomation,
                &format!("Playwright MCP tool '{}'", self.raw_name),
                None,
                params.clone(),
            )
            .await
            .map_err(ToolError::Security)?;

        let result = self.manager.call_raw(&self.raw_name, params).await?;

        let stdout = result.to_string();
        let is_error = result.get("isError").and_then(|v| v.as_bool()).unwrap_or(false);

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: self.definition.name.clone(),
            stdout,
            stderr: String::new(),
            exit_code: if is_error { Some(1) } else { Some(0) },
            duration_ms: start.elapsed().as_millis() as u64,
            is_error,
            truncated: false,
        })
    }
}
