use crate::types::{ToolError, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::Mutex;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

pub struct McpClient {
    child: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<ChildStdin>>,
    reader: Arc<Mutex<BufReader<tokio::process::ChildStdout>>>,
    request_id: AtomicU64,
}

impl McpClient {
    pub async fn spawn(command: &str, args: &[&str], envs: &[(String, String)]) -> ToolResult<Self> {
        let mut cmd = Command::new(command);
        #[cfg(windows)]
        cmd.creation_flags(0x08000000);
        cmd.args(args);
        cmd.envs(envs.iter().cloned());
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        info!("Spawning MCP server process: {} {:?}", command, args);

        let mut child = cmd.spawn().map_err(|e| {
            ToolError::Mcp(format!(
                "Failed to spawn MCP server '{}': {}. Verify command/Python/uv installation.",
                command, e
            ))
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| ToolError::Mcp("Failed to capture MCP stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ToolError::Mcp("Failed to capture MCP stdout".to_string()))?;

        // Background task to log stderr
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    debug!("[MCP-SERVER-STDERR] {}", line);
                }
            });
        }

        let client = Self {
            child: Arc::new(Mutex::new(child)),
            stdin: Arc::new(Mutex::new(stdin)),
            reader: Arc::new(Mutex::new(BufReader::new(stdout))),
            request_id: AtomicU64::new(1),
        };

        // Initialize MCP connection
        client.initialize().await?;

        Ok(client)
    }

    async fn send_request(&self, method: &str, params: Value) -> ToolResult<Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let req = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        let mut req_str = req.to_string();
        req_str.push('\n');

        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(req_str.as_bytes()).await.map_err(|e| {
                ToolError::Mcp(format!("Failed to write request to MCP stdin: {e}"))
            })?;
            stdin.flush().await.map_err(|e| {
                ToolError::Mcp(format!("Failed to flush MCP stdin: {e}"))
            })?;
        }

        // Read response line
        let mut reader = self.reader.lock().await;
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = tokio::time::timeout(
                std::time::Duration::from_secs(45),
                reader.read_line(&mut line),
            )
            .await
            .map_err(|_| ToolError::Mcp("Timeout waiting for MCP response".to_string()))?
            .map_err(|e| ToolError::Mcp(format!("Failed to read line from MCP stdout: {e}")))?;

            if bytes_read == 0 {
                return Err(ToolError::Mcp(
                    "MCP server closed output stream unexpectedly".to_string(),
                ));
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Ok(resp) = serde_json::from_str::<Value>(trimmed) {
                // Check if this response matches our request ID
                if resp.get("id").and_then(|v| v.as_u64()) == Some(id) {
                    if let Some(err) = resp.get("error") {
                        return Err(ToolError::Mcp(format!("MCP Server returned error: {err}")));
                    }
                    return resp
                        .get("result")
                        .cloned()
                        .ok_or_else(|| ToolError::Mcp("Missing result in MCP response".to_string()));
                }
            }
        }
    }

    async fn send_notification(&self, method: &str, params: Value) -> ToolResult<()> {
        let note = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        let mut note_str = note.to_string();
        note_str.push('\n');

        let mut stdin = self.stdin.lock().await;
        stdin.write_all(note_str.as_bytes()).await.map_err(|e| {
            ToolError::Mcp(format!("Failed to write notification to MCP stdin: {e}"))
        })?;
        stdin.flush().await.map_err(|e| {
            ToolError::Mcp(format!("Failed to flush MCP stdin: {e}"))
        })?;
        Ok(())
    }

    pub async fn initialize(&self) -> ToolResult<Value> {
        let init_params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "aai-windows-desktop",
                "version": "0.1.0"
            }
        });

        let res = self.send_request("initialize", init_params).await?;
        self.send_notification("notifications/initialized", json!({}))
            .await?;
        info!("MCP initialization completed successfully");
        Ok(res)
    }

    pub async fn list_tools(&self) -> ToolResult<Vec<McpToolInfo>> {
        let res = self.send_request("tools/list", json!({})).await?;
        let tools_val = res
            .get("tools")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ToolError::Mcp("Invalid tools/list response format".to_string()))?;

        let mut list = Vec::new();
        for item in tools_val {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                let description = item
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let input_schema = item
                    .get("inputSchema")
                    .cloned()
                    .unwrap_or_else(|| json!({"type": "object"}));

                list.push(McpToolInfo {
                    name: name.to_string(),
                    description,
                    input_schema,
                });
            }
        }

        info!("Discovered {} MCP tools from server", list.len());
        Ok(list)
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> ToolResult<Value> {
        let params = json!({
            "name": name,
            "arguments": arguments
        });

        self.send_request("tools/call", params).await
    }

    pub async fn shutdown(&self) {
        let mut child = self.child.lock().await;
        let _ = child.kill().await;
    }
}
