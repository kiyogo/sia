pub mod filesystem;
pub mod mcp_client;
pub mod registry;
pub mod terminal;
pub mod types;
pub mod windows_mcp;

pub use filesystem::{FileListTool, FilePatchTool, FileReadTool, FileSearchTool, FileWriteTool};
pub use mcp_client::{McpClient, McpToolInfo};
pub use registry::{create_default_registry, SharedToolRegistry, ToolRegistry};
pub use terminal::TerminalTool;
pub use types::{Tool, ToolDefinition, ToolError, ToolExecutionResult, ToolResult};
pub use windows_mcp::{DynamicMcpTool, WindowsMcpManager};

#[cfg(test)]
mod tests {
    use super::*;
    use agent_platform::{WindowsFileSystemProvider, WindowsShellExecutor};
    use agent_security::{PermissionMode, SecurityEngine};
    use serde_json::json;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_terminal_tool_real_execution() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().to_path_buf();
        let fs = Box::new(WindowsFileSystemProvider::new());
        let sec = Arc::new(SecurityEngine::new(root.clone(), PermissionMode::FullAuto, fs));

        let executor = Box::new(WindowsShellExecutor::new());
        let registry = create_default_registry(executor, sec);

        let res = registry
            .execute(
                "terminal_execute",
                "call_1",
                json!({
                    "command": "echo HelloFromTerminalTool",
                    "shell": "cmd"
                }),
            )
            .await
            .expect("terminal execution");

        assert_eq!(res.exit_code, Some(0));
        assert!(res.stdout.contains("HelloFromTerminalTool"));
    }

    #[tokio::test]
    async fn test_filesystem_tools_crud() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().to_path_buf();
        let fs = Box::new(WindowsFileSystemProvider::new());
        let sec = Arc::new(SecurityEngine::new(root.clone(), PermissionMode::FullAuto, fs));

        let executor = Box::new(WindowsShellExecutor::new());
        let registry = create_default_registry(executor, sec);

        // 1. File write
        let write_res = registry
            .execute(
                "file_write",
                "call_w",
                json!({
                    "path": "test.txt",
                    "content": "Line 1\nLine 2 TargetString\nLine 3"
                }),
            )
            .await
            .expect("file write");
        assert_eq!(write_res.exit_code, Some(0));

        // 2. File read
        let read_res = registry
            .execute(
                "file_read",
                "call_r",
                json!({
                    "path": "test.txt"
                }),
            )
            .await
            .expect("file read");
        assert!(read_res.stdout.contains("TargetString"));

        // 3. File search
        let search_res = registry
            .execute(
                "file_search",
                "call_s",
                json!({
                    "query": "TargetString",
                    "content_search": true
                }),
            )
            .await
            .expect("file search");
        assert!(search_res.stdout.contains("test.txt"));

        // 4. File patch
        let patch_res = registry
            .execute(
                "file_patch",
                "call_p",
                json!({
                    "path": "test.txt",
                    "target": "TargetString",
                    "replacement": "ReplacedString"
                }),
            )
            .await
            .expect("file patch");
        assert!(patch_res.stdout.contains("ReplacedString"));
    }
}
