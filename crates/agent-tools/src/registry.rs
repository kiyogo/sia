use crate::build_test::{BuildRunnerTool, TestRunnerTool};
use crate::code_edit::{CodeEditTool, CodePatchTool, DiffPreviewTool};
use crate::code_intelligence::{CodeSearchTool, FileOutlineTool, ProjectDetectTool, RepoMapTool};
use crate::filesystem::{FileListTool, FilePatchTool, FileReadTool, FileSearchTool, FileWriteTool};
use crate::git::{GitCommitTool, GitDiffTool, GitLogTool, GitStatusTool};
use crate::native_windows_mcp::{
    DesktopAppTool, DesktopClickTool, DesktopClipboardTool, DesktopMoveTool, DesktopScreenshotTool,
    DesktopScrollTool, DesktopShortcutTool, DesktopSnapshotTool, DesktopSwitchWindowTool,
    DesktopTypeTool,
};
use crate::terminal::TerminalTool;
use crate::todo::TodoWriteTool;
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
        if let Some(t) = map.get(name) {
            return Some(t.clone());
        }
        // OpenCode native tool alias resolution
        let alias = match name.to_lowercase().as_str() {
            "read" => Some("file_read"),
            "write" => Some("file_write"),
            "edit" => Some("code_edit"),
            "patch" => Some("code_patch"),
            "glob" => Some("file_list"),
            "grep" => Some("code_search"),
            "shell" => Some("terminal_execute"),
            "todo" | "todowrite" => Some("todowrite"),
            // CursorTouch / Windows-MCP native aliases
            "click" | "desktop_click" => Some("desktop_click"),
            "type" | "desktop_type" => Some("desktop_type"),
            "scroll" | "desktop_scroll" => Some("desktop_scroll"),
            "move" | "desktop_move" => Some("desktop_move"),
            "shortcut" | "desktop_shortcut" => Some("desktop_shortcut"),
            "app" | "launch" | "desktop_app" => Some("desktop_app"),
            "clipboard" | "desktop_clipboard" => Some("desktop_clipboard"),
            "snapshot" | "state" | "desktop_snapshot" | "desktop_state" => Some("desktop_snapshot"),
            "screenshot" | "desktop_screenshot" => Some("desktop_screenshot"),
            "switchwindow" | "switch_window" | "desktop_switch_window" => Some("desktop_switch_window"),
            _ => None,
        };
        if let Some(target) = alias {
            return map.get(target).cloned();
        }
        None
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
    let executor_arc: Arc<dyn ShellExecutor> = Arc::from(shell_executor);

    let terminal_tool = Arc::new(TerminalTool::new_arc(executor_arc.clone(), security.clone()));
    let file_read_tool = Arc::new(FileReadTool::new(security.clone()));
    let file_write_tool = Arc::new(FileWriteTool::new(security.clone()));
    let file_list_tool = Arc::new(FileListTool::new(security.clone()));
    let file_search_tool = Arc::new(FileSearchTool::new(security.clone()));
    let file_patch_tool = Arc::new(FilePatchTool::new(security.clone()));

    // Git Tools
    let git_status_tool = Arc::new(GitStatusTool::new(executor_arc.clone(), security.clone()));
    let git_diff_tool = Arc::new(GitDiffTool::new(executor_arc.clone(), security.clone()));
    let git_log_tool = Arc::new(GitLogTool::new(executor_arc.clone(), security.clone()));
    let git_commit_tool = Arc::new(GitCommitTool::new(executor_arc.clone(), security.clone()));

    // Code Intelligence Tools
    let project_detect_tool = Arc::new(ProjectDetectTool::new(security.clone()));
    let repo_map_tool = Arc::new(RepoMapTool::new(security.clone()));
    let code_search_tool = Arc::new(CodeSearchTool::new(security.clone()));
    let file_outline_tool = Arc::new(FileOutlineTool::new(security.clone()));

    // Code Mutation & Diff Tools (OpenCode-style)
    let code_edit_tool = Arc::new(CodeEditTool::new(security.clone()));
    let code_patch_tool = Arc::new(CodePatchTool::new(security.clone()));
    let diff_preview_tool = Arc::new(DiffPreviewTool::new(security.clone()));

    // Build & Test Tools
    let build_runner_tool = Arc::new(BuildRunnerTool::new(executor_arc.clone(), security.clone()));
    let test_runner_tool = Arc::new(TestRunnerTool::new(executor_arc.clone(), security.clone()));

    // OpenCode Planning & Todo Tool
    let todo_tool = Arc::new(TodoWriteTool::new(security.clone()));

    // CursorTouch / Windows-MCP Native Desktop Automation Tools
    let desktop_click = Arc::new(DesktopClickTool::new(security.clone()));
    let desktop_type = Arc::new(DesktopTypeTool::new(security.clone()));
    let desktop_scroll = Arc::new(DesktopScrollTool::new(security.clone()));
    let desktop_move = Arc::new(DesktopMoveTool::new(security.clone()));
    let desktop_shortcut = Arc::new(DesktopShortcutTool::new(security.clone()));
    let desktop_app = Arc::new(DesktopAppTool::new(security.clone()));
    let desktop_clipboard = Arc::new(DesktopClipboardTool::new(security.clone()));
    let desktop_snapshot = Arc::new(DesktopSnapshotTool::new(security.clone()));
    let desktop_screenshot = Arc::new(DesktopScreenshotTool::new(security.clone()));
    let desktop_switch = Arc::new(DesktopSwitchWindowTool::new(security.clone()));

    let mut map = HashMap::new();
    map.insert(terminal_tool.definition().name, terminal_tool as Arc<dyn Tool>);
    map.insert(file_read_tool.definition().name, file_read_tool as Arc<dyn Tool>);
    map.insert(file_write_tool.definition().name, file_write_tool as Arc<dyn Tool>);
    map.insert(file_list_tool.definition().name, file_list_tool as Arc<dyn Tool>);
    map.insert(file_search_tool.definition().name, file_search_tool as Arc<dyn Tool>);
    map.insert(file_patch_tool.definition().name, file_patch_tool as Arc<dyn Tool>);

    map.insert(git_status_tool.definition().name, git_status_tool as Arc<dyn Tool>);
    map.insert(git_diff_tool.definition().name, git_diff_tool as Arc<dyn Tool>);
    map.insert(git_log_tool.definition().name, git_log_tool as Arc<dyn Tool>);
    map.insert(git_commit_tool.definition().name, git_commit_tool as Arc<dyn Tool>);

    map.insert(project_detect_tool.definition().name, project_detect_tool as Arc<dyn Tool>);
    map.insert(repo_map_tool.definition().name, repo_map_tool as Arc<dyn Tool>);
    map.insert(code_search_tool.definition().name, code_search_tool as Arc<dyn Tool>);
    map.insert(file_outline_tool.definition().name, file_outline_tool as Arc<dyn Tool>);

    map.insert(code_edit_tool.definition().name, code_edit_tool as Arc<dyn Tool>);
    map.insert(code_patch_tool.definition().name, code_patch_tool as Arc<dyn Tool>);
    map.insert(diff_preview_tool.definition().name, diff_preview_tool as Arc<dyn Tool>);

    map.insert(build_runner_tool.definition().name, build_runner_tool as Arc<dyn Tool>);
    map.insert(test_runner_tool.definition().name, test_runner_tool as Arc<dyn Tool>);
    map.insert(todo_tool.definition().name, todo_tool as Arc<dyn Tool>);

    // Windows-MCP native tools insertion
    map.insert(desktop_click.definition().name, desktop_click as Arc<dyn Tool>);
    map.insert(desktop_type.definition().name, desktop_type as Arc<dyn Tool>);
    map.insert(desktop_scroll.definition().name, desktop_scroll as Arc<dyn Tool>);
    map.insert(desktop_move.definition().name, desktop_move as Arc<dyn Tool>);
    map.insert(desktop_shortcut.definition().name, desktop_shortcut as Arc<dyn Tool>);
    map.insert(desktop_app.definition().name, desktop_app as Arc<dyn Tool>);
    map.insert(desktop_clipboard.definition().name, desktop_clipboard as Arc<dyn Tool>);
    map.insert(desktop_snapshot.definition().name, desktop_snapshot as Arc<dyn Tool>);
    map.insert(desktop_screenshot.definition().name, desktop_screenshot as Arc<dyn Tool>);
    map.insert(desktop_switch.definition().name, desktop_switch as Arc<dyn Tool>);

    ToolRegistry {
        tools: RwLock::new(map),
    }
}

pub type SharedToolRegistry = Arc<ToolRegistry>;
