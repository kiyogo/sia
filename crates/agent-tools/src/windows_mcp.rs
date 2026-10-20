use crate::mcp_client::McpClient;
use crate::types::{Tool, ToolDefinition, ToolError, ToolExecutionResult, ToolResult};
use agent_platform::{OperatingSystemAdapter, WindowsOperatingSystemAdapter};
use agent_security::{PermissionTier, SharedSecurityEngine};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::info;

pub struct WindowsMcpManager {
    client: Arc<RwLock<Option<McpClient>>>,
    security: SharedSecurityEngine,
    os_adapter: WindowsOperatingSystemAdapter,
}

impl WindowsMcpManager {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            security,
            os_adapter: WindowsOperatingSystemAdapter::new(),
        }
    }

    pub async fn check_prerequisites(&self) -> (bool, String) {
        let statuses = self.os_adapter.check_prerequisites().await;
        let mut missing = Vec::new();

        for s in &statuses {
            if !s.installed && (s.name.contains("Python") || s.name.contains("uv")) {
                missing.push(format!("• {}: {}", s.name, s.install_hint));
            }
        }

        if missing.is_empty() {
            (true, "All prerequisites (Python 3.14+, uv) are satisfied.".to_string())
        } else {
            (
                false,
                format!(
                    "Missing prerequisites for Windows-MCP:\n{}\n\nPlease install required components before launching Windows-MCP.",
                    missing.join("\n")
                ),
            )
        }
    }

    pub async fn is_connected(&self) -> bool {
        self.client.read().await.is_some()
    }

    pub async fn connect(&self, custom_command: Option<(&str, Vec<String>)>) -> ToolResult<Vec<ToolDefinition>> {
        let (ok, msg) = self.check_prerequisites().await;
        if !ok {
            return Err(ToolError::Mcp(msg));
        }

        // Determine launch command
        let (cmd, args_vec) = if let Some((c, a)) = custom_command {
            (c.to_string(), a)
        } else {
            (
                "uv".to_string(),
                vec![
                    "run".to_string(),
                    "--python".to_string(),
                    "3.14".to_string(),
                    "--with".to_string(),
                    "windows-mcp".to_string(),
                    "python".to_string(),
                    "-m".to_string(),
                    "windows_mcp".to_string(),
                    "serve".to_string(),
                ],
            )
        };

        let args_refs: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

        let client = McpClient::spawn(&cmd, &args_refs, &[]).await?;
        let discovered_tools = client.list_tools().await?;

        let mut definitions = Vec::new();
        for t in &discovered_tools {
            definitions.push(ToolDefinition {
                name: format!("windows_mcp_{}", t.name),
                description: format!(
                    "[Windows-MCP Native Control] {}",
                    t.description.as_deref().unwrap_or("Desktop Automation Tool")
                ),
                parameters: t.input_schema.clone(),
            });
        }

        let mut lock = self.client.write().await;
        *lock = Some(client);

        info!("Windows-MCP connected with {} tools registered", definitions.len());
        Ok(definitions)
    }

    pub async fn disconnect(&self) {
        let mut lock = self.client.write().await;
        if let Some(client) = lock.take() {
            client.shutdown().await;
            info!("Windows-MCP connection closed");
        }
    }

    pub async fn call_raw(&self, tool_name: &str, arguments: serde_json::Value) -> ToolResult<serde_json::Value> {
        let lock = self.client.read().await;
        let client = lock.as_ref().ok_or_else(|| {
            ToolError::Mcp("Windows-MCP server is not connected. Call connect() first.".to_string())
        })?;

        client.call_tool(tool_name, arguments).await
    }

    pub async fn launch_app(&self, app: &str) -> ToolResult<serde_json::Value> {
        self.security
            .check_permission(
                PermissionTier::DesktopControl,
                &format!("Launch desktop application '{app}'"),
                None,
                serde_json::json!({ "app": app }),
            )
            .await
            .map_err(ToolError::Security)?;

        self.call_raw("App", serde_json::json!({
            "mode": "launch",
            "name": app
        })).await
    }

    pub async fn switch_window(&self, title: &str) -> ToolResult<serde_json::Value> {
        self.security
            .check_permission(
                PermissionTier::DesktopControl,
                &format!("Switch window to '{title}'"),
                None,
                serde_json::json!({ "title": title }),
            )
            .await
            .map_err(ToolError::Security)?;

        self.call_raw("App", serde_json::json!({
            "mode": "switch",
            "name": title
        })).await
    }

    pub async fn click(&self, x: i32, y: i32, button: &str, click_type: u32) -> ToolResult<serde_json::Value> {
        self.security
            .check_permission(
                PermissionTier::DesktopControl,
                &format!("Mouse click at [{x}, {y}] with {button}"),
                None,
                serde_json::json!({ "x": x, "y": y, "button": button, "clicks": click_type }),
            )
            .await
            .map_err(ToolError::Security)?;

        self.call_raw("Click", serde_json::json!({
            "loc": [x, y],
            "button": button,
            "clicks": click_type
        })).await
    }

    pub async fn type_text(&self, text: &str, clear: bool, press_enter: bool) -> ToolResult<serde_json::Value> {
        self.security
            .check_permission(
                PermissionTier::DesktopControl,
                &format!("Type text (length: {})", text.len()),
                None,
                serde_json::json!({ "clear": clear, "press_enter": press_enter }),
            )
            .await
            .map_err(ToolError::Security)?;

        self.call_raw("Type", serde_json::json!({
            "text": text,
            "clear": clear,
            "press_enter": press_enter
        })).await
    }

    pub async fn shortcut(&self, combo: &str) -> ToolResult<serde_json::Value> {
        self.security
            .check_permission(
                PermissionTier::DesktopControl,
                &format!("Dispatch shortcut key '{combo}'"),
                None,
                serde_json::json!({ "keys": combo }),
            )
            .await
            .map_err(ToolError::Security)?;

        self.call_raw("Shortcut", serde_json::json!({
            "keys": combo
        })).await
    }

    pub async fn snapshot(&self, use_vision: bool) -> ToolResult<serde_json::Value> {
        self.call_raw("Snapshot", serde_json::json!({
            "use_vision": use_vision
        })).await
    }

    pub async fn screenshot(&self) -> ToolResult<serde_json::Value> {
        self.call_raw("Screenshot", serde_json::json!({})).await
    }

    pub async fn get_control_status(&self) -> ToolResult<serde_json::Value> {
        self.call_raw("ControlStatus", serde_json::json!({})).await
    }
}

pub struct DynamicMcpTool {
    manager: Arc<WindowsMcpManager>,
    raw_name: String,
    definition: ToolDefinition,
    security: SharedSecurityEngine,
}

impl DynamicMcpTool {
    pub fn new(
        manager: Arc<WindowsMcpManager>,
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
impl Tool for DynamicMcpTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();

        // Enforce desktop control permission check
        self.security.check_permission(
            PermissionTier::DesktopControl,
            &format!("Windows-MCP control tool '{}'", self.raw_name),
            None,
            params.clone(),
        ).await?;

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
