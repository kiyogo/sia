pub mod browser_mcp;
pub mod build_test;
pub mod code_edit;
pub mod code_intelligence;
pub mod filesystem;
pub mod git;
pub mod mcp_client;
pub mod registry;
pub mod terminal;
pub mod todo;
pub mod types;
pub mod windows_mcp;

pub use browser_mcp::{BrowserMcpManager, DynamicBrowserMcpTool};
pub use build_test::{BuildRunnerTool, TestRunnerTool};
pub use code_edit::{compute_unified_diff, CodeEditTool, CodePatchTool, DiffPreviewTool};
pub use code_intelligence::{CodeSearchTool, FileOutlineTool, ProjectDetectTool, RepoMapTool};
pub use filesystem::{FileListTool, FilePatchTool, FileReadTool, FileSearchTool, FileWriteTool};
pub use git::{GitCommitTool, GitDiffTool, GitLogTool, GitStatusTool};
pub use mcp_client::{McpClient, McpToolInfo};
pub use registry::{create_default_registry, SharedToolRegistry, ToolRegistry};
pub use terminal::TerminalTool;
pub use todo::{TodoItem, TodoWriteTool};
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

    #[tokio::test]
    async fn test_coding_tools_edit_and_search() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().to_path_buf();
        let fs = Box::new(WindowsFileSystemProvider::new());
        let sec = Arc::new(SecurityEngine::new(root.clone(), PermissionMode::FullAuto, fs));
        let executor = Box::new(WindowsShellExecutor::new());
        let registry = create_default_registry(executor, sec);

        // 1. Write source file
        std::fs::write(root.join("main.rs"), "fn calculate(x: i32) -> i32 {\n    x + 10\n}\n").expect("write");

        // 2. OpenCode style code_edit
        let edit_res = registry
            .execute(
                "code_edit",
                "call_e",
                json!({
                    "path": "main.rs",
                    "old_string": "x + 10",
                    "new_string": "x * 20"
                }),
            )
            .await
            .expect("code edit");
        assert!(edit_res.stdout.contains("diff"));

        let content = std::fs::read_to_string(root.join("main.rs")).expect("read");
        assert!(content.contains("x * 20"));

        // 3. Code search
        let search_res = registry
            .execute(
                "code_search",
                "call_cs",
                json!({
                    "pattern": "calculate"
                }),
            )
            .await
            .expect("code search");
        assert!(search_res.stdout.contains("calculate"));

        // 4. Repo map
        let map_res = registry
            .execute("repo_map", "call_rm", json!({ "max_chars": 2000 }))
            .await
            .expect("repo map");
        assert!(map_res.stdout.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_code_patch_atomic_rollback() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().to_path_buf();
        let fs = Box::new(WindowsFileSystemProvider::new());
        let sec = Arc::new(SecurityEngine::new(root.clone(), PermissionMode::FullAuto, fs));
        let executor = Box::new(WindowsShellExecutor::new());
        let registry = create_default_registry(executor, sec);

        std::fs::write(root.join("a.txt"), "Original A\n").expect("write a");

        // Attempt batch patch where second file targets an illegal directory outside workspace
        let res = registry
            .execute(
                "code_patch",
                "call_p_fail",
                json!({
                    "edits": [
                        { "path": "a.txt", "content": "Modified A\n" },
                        { "path": "../../outside_sandbox.txt", "content": "Hacked\n" }
                    ]
                }),
            )
            .await;

        assert!(res.is_err());
        // Verify a.txt was NOT modified due to validation or was rolled back
        let content_a = std::fs::read_to_string(root.join("a.txt")).expect("read a");
        assert_eq!(content_a, "Original A\n");
    }

    #[tokio::test]
    async fn test_opencode_todowrite_and_alias_resolution() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().to_path_buf();
        let fs = Box::new(WindowsFileSystemProvider::new());
        let sec = Arc::new(SecurityEngine::new(root.clone(), PermissionMode::FullAuto, fs));
        let executor = Box::new(WindowsShellExecutor::new());
        let registry = create_default_registry(executor, sec);

        // 1. OpenCode todowrite execution
        let res = registry
            .execute(
                "todowrite",
                "call_todo_1",
                json!({
                    "todos": [
                        { "content": "Inspect architecture", "status": "completed" },
                        { "content": "Port OpenCode algorithms to Rust", "status": "in_progress" },
                        { "content": "Run tests and verify", "status": "pending" }
                    ]
                }),
            )
            .await
            .expect("todowrite should succeed");

        assert!(res.stdout.contains("1/3 completed"));
        assert!(res.stdout.contains("1 in progress"));
        assert!(res.stdout.contains("Port OpenCode algorithms to Rust"));

        // 2. OpenCode alias resolution: 'read' should resolve to FileReadTool
        std::fs::write(root.join("hello.txt"), "Line 1\nLine 2\n").expect("write");
        let read_res = registry
            .execute(
                "read",
                "call_read_alias",
                json!({ "path": "hello.txt" }),
            )
            .await
            .expect("read alias should resolve");
        assert!(read_res.stdout.contains("Line 1"));

        // 3. OpenCode alias resolution: 'edit' with oldString / newString should resolve to CodeEditTool
        let edit_res = registry
            .execute(
                "edit",
                "call_edit_alias",
                json!({
                    "path": "hello.txt",
                    "oldString": "Line 2",
                    "newString": "Line Two"
                }),
            )
            .await
            .expect("edit alias should resolve and apply");
        assert!(edit_res.stdout.contains("hello.txt"));
        assert!(edit_res.stdout.contains("replacements"));
    }
}
