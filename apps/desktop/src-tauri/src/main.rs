// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use agent_platform::{
    ComputerControlAdapter, OperatingSystemAdapter, ProcessOutput, WindowsComputerControlAdapter,
    WindowsOperatingSystemAdapter,
};
use agent_security::{ApprovalRequest, PermissionTier};
use agent_storage::{Checkpoint, Conversation, Message, Task};
use agent_tools::ToolDefinition;
use agent_runtime::{DashboardStats, RecoveredTaskInfo, RuntimeConfig, RuntimeHost};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Emitter, State};
use tracing::info;
use uuid::Uuid;

struct AppState {
    host: Arc<RuntimeHost>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceFileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[tauri::command]
async fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    state.host.get_dashboard_stats().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_system_prerequisites() -> Result<Vec<agent_platform::PrerequisiteStatus>, String> {
    let os = WindowsOperatingSystemAdapter::new();
    Ok(os.check_prerequisites().await)
}

#[tauri::command]
async fn list_conversations(state: State<'_, AppState>) -> Result<Vec<Conversation>, String> {
    state.host.database().list_conversations().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_conversation(title: String, state: State<'_, AppState>) -> Result<Conversation, String> {
    state.host.database().create_conversation(&title).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_messages(conversation_id: String, state: State<'_, AppState>) -> Result<Vec<Message>, String> {
    let id = Uuid::parse_str(&conversation_id).map_err(|e| e.to_string())?;
    state.host.database().get_messages(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn send_chat_message(
    conversation_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let conv_id = Uuid::parse_str(&conversation_id).map_err(|e| e.to_string())?;

    // Record user message
    let user_msg = Message {
        id: Uuid::new_v4(),
        conversation_id: conv_id,
        role: "user".to_string(),
        content: content.clone(),
        tool_calls: None,
        tool_call_id: None,
        token_count: agent_core::TokenOptimizer::estimate_tokens(&content),
        created_at: chrono::Utc::now(),
    };
    state.host.database().add_message(&user_msg).await.map_err(|e| e.to_string())?;

    // Start task for this message
    let title = format!("Chat Turn: {}", content.chars().take(30).collect::<String>());
    let task_id = state
        .host
        .start_task(&title, "Respond to user message with tools if required", &content, Some(conv_id))
        .await
        .map_err(|e| e.to_string())?;

    Ok(task_id.to_string())
}

#[tauri::command]
async fn list_tasks(state: State<'_, AppState>) -> Result<Vec<Task>, String> {
    state.host.database().list_tasks().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_task(
    title: String,
    goal: String,
    prompt: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let task_id = state
        .host
        .start_task(&title, &goal, &prompt, None)
        .await
        .map_err(|e| e.to_string())?;
    Ok(task_id.to_string())
}

#[tauri::command]
async fn cancel_task(task_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let id = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;
    state.host.cancel_task(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_task_checkpoints(task_id: String, state: State<'_, AppState>) -> Result<Vec<Checkpoint>, String> {
    let id = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;
    let latest = state.host.database().get_latest_checkpoint(id).await.map_err(|e| e.to_string())?;
    Ok(latest.into_iter().collect())
}

#[tauri::command]
async fn recover_tasks(state: State<'_, AppState>) -> Result<Vec<RecoveredTaskInfo>, String> {
    state.host.recover_tasks_from_checkpoints().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_workspace_path(state: State<'_, AppState>) -> Result<String, String> {
    let cfg = state.host.get_config().await;
    Ok(cfg.workspace_path.display().to_string())
}

#[tauri::command]
async fn set_workspace_path(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut cfg = state.host.get_config().await;
    cfg.workspace_path = PathBuf::from(path);
    state.host.update_config(cfg).await;
    Ok(())
}

#[tauri::command]
async fn list_workspace_files(
    subpath: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<WorkspaceFileInfo>, String> {
    let root = state.host.security().get_workspace_root().await;
    let target = match subpath {
        Some(p) => root.join(p),
        None => root.clone(),
    };

    if !target.exists() {
        let _ = std::fs::create_dir_all(&target);
    }

    let mut list = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&target) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let is_dir = path.is_dir();
            let name = entry.file_name().to_string_lossy().to_string();
            let size = if is_dir { 0 } else { entry.metadata().map(|m| m.len()).unwrap_or(0) };
            let rel = path.strip_prefix(&root).unwrap_or(&path).to_string_lossy().to_string();

            list.push(WorkspaceFileInfo {
                name,
                path: rel,
                is_dir,
                size,
            });
        }
    }

    Ok(list)
}

#[tauri::command]
async fn read_workspace_file(file_path: String, state: State<'_, AppState>) -> Result<String, String> {
    let root = state.host.security().get_workspace_root().await;
    let path = root.join(&file_path);
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))
}

#[tauri::command]
async fn write_workspace_file(
    file_path: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let root = state.host.security().get_workspace_root().await;
    let path = root.join(&file_path);

    state
        .host
        .security()
        .check_permission(PermissionTier::WorkspaceWrite, &format!("Edit file {file_path}"), Some(&path), json!({}))
        .await
        .map_err(|e| e.to_string())?;

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    std::fs::write(&path, &content).map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
    Ok(format!("Saved {}", path.display()))
}

#[tauri::command]
async fn execute_terminal_command(
    command: String,
    shell: String,
    state: State<'_, AppState>,
) -> Result<ProcessOutput, String> {
    let tool_res = state
        .host
        .registry()
        .execute(
            "terminal_execute",
            "tauri_manual",
            json!({
                "command": command,
                "shell": shell,
            }),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(ProcessOutput {
        stdout: tool_res.stdout,
        stderr: tool_res.stderr,
        exit_code: tool_res.exit_code,
        duration_ms: tool_res.duration_ms,
        truncated: tool_res.truncated,
    })
}

#[tauri::command]
async fn check_mcp_status(state: State<'_, AppState>) -> Result<(bool, bool, String), String> {
    let is_connected = state.host.mcp_manager().is_connected().await;
    let (prereq_ok, hint) = state.host.mcp_manager().check_prerequisites().await;
    Ok((is_connected, prereq_ok, hint))
}

#[tauri::command]
async fn connect_mcp(state: State<'_, AppState>) -> Result<Vec<ToolDefinition>, String> {
    state.host.mcp_manager().connect(None).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn disconnect_mcp(state: State<'_, AppState>) -> Result<(), String> {
    state.host.mcp_manager().disconnect().await;
    Ok(())
}

#[tauri::command]
async fn list_pending_approvals(state: State<'_, AppState>) -> Result<Vec<ApprovalRequest>, String> {
    Ok(state.host.security().list_pending_approvals().await)
}

#[tauri::command]
async fn resolve_approval(id: String, approved: bool, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.host.security().resolve_approval(uuid, approved).await;
    Ok(())
}

#[tauri::command]
async fn trigger_emergency_stop(state: State<'_, AppState>) -> Result<(), String> {
    state.host.security().trigger_emergency_stop();
    Ok(())
}

#[tauri::command]
async fn reset_emergency_stop(state: State<'_, AppState>) -> Result<(), String> {
    state.host.security().reset_emergency_stop();
    Ok(())
}

#[tauri::command]
async fn get_runtime_config(state: State<'_, AppState>) -> Result<RuntimeConfig, String> {
    Ok(state.host.get_config().await)
}

#[tauri::command]
async fn save_runtime_config(
    config: RuntimeConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.host.update_config(config).await;
    Ok(())
}

#[tauri::command]
async fn test_provider_connection(state: State<'_, AppState>) -> Result<bool, String> {
    let stats = state.host.get_dashboard_stats().await.map_err(|e| e.to_string())?;
    Ok(stats.provider_connected)
}

#[tauri::command]
async fn detect_project(subpath: Option<String>, state: State<'_, AppState>) -> Result<Option<agent_platform::ProjectInfo>, String> {
    Ok(state.host.detect_project(subpath.as_deref()).await)
}

#[tauri::command]
async fn get_repo_map(max_chars: Option<u64>, subdir: Option<String>, state: State<'_, AppState>) -> Result<String, String> {
    state.host.get_repo_map(max_chars, subdir.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn preview_diff(path: String, new_content: String, state: State<'_, AppState>) -> Result<String, String> {
    state.host.preview_diff(&path, &new_content).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_build(command: Option<String>, path: Option<String>, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    state.host.run_build(command.as_deref(), path.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_test(command: Option<String>, filter: Option<String>, path: Option<String>, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    state.host.run_test(command.as_deref(), filter.as_deref(), path.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_coding_task(
    title: String,
    goal: String,
    user_prompt: String,
    mode: String,
    conversation_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let conv_id = match conversation_id {
        Some(s) if !s.is_empty() => Some(Uuid::parse_str(&s).map_err(|e| e.to_string())?),
        _ => None,
    };
    let coding_mode = match mode.to_lowercase().as_str() {
        "ask" => agent_core::CodingMode::Ask,
        "plan" => agent_core::CodingMode::Plan,
        _ => agent_core::CodingMode::Build,
    };
    let task_id = state
        .host
        .start_coding_task(&title, &goal, &user_prompt, coding_mode, conv_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(task_id.to_string())
}

#[tauri::command]
async fn get_ownership_status(state: State<'_, AppState>) -> Result<agent_security::OwnershipStatusReport, String> {
    Ok(state.host.security().ownership().get_ownership_status().await)
}

#[tauri::command]
async fn acquire_desktop_lease(agent_name: String, timeout_secs: u64, state: State<'_, AppState>) -> Result<String, String> {
    state.host.security().ownership().acquire_lease(&agent_name, timeout_secs).await.map(|id| id.to_string()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn release_desktop_lease(lease_id: String, state: State<'_, AppState>) -> Result<bool, String> {
    let id = Uuid::parse_str(&lease_id).map_err(|e| e.to_string())?;
    Ok(state.host.security().ownership().release_lease(id).await)
}

#[tauri::command]
async fn user_takeover_desktop(state: State<'_, AppState>) -> Result<(), String> {
    state.host.security().ownership().user_takeover().await;
    Ok(())
}

#[tauri::command]
async fn release_user_control_desktop(state: State<'_, AppState>) -> Result<(), String> {
    state.host.security().ownership().release_user_control().await;
    Ok(())
}

#[tauri::command]
async fn list_desktop_windows() -> Result<Vec<agent_platform::WindowInfo>, String> {
    let adapter = WindowsComputerControlAdapter::new();
    adapter.list_windows().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn switch_desktop_window(title: String) -> Result<bool, String> {
    let adapter = WindowsComputerControlAdapter::new();
    adapter.switch_window(&title).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_desktop_lock_status() -> Result<String, String> {
    let adapter = WindowsComputerControlAdapter::new();
    let status = adapter.get_lock_status().await.map_err(|e| e.to_string())?;
    Ok(format!("{:?}", status))
}

#[tauri::command]
async fn take_desktop_screenshot() -> Result<String, String> {
    let adapter = WindowsComputerControlAdapter::new();
    let bytes = adapter.take_screenshot().await.map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

#[tauri::command]
async fn connect_browser(state: State<'_, AppState>) -> Result<Vec<ToolDefinition>, String> {
    state.host.browser_mcp().connect(None).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn disconnect_browser(state: State<'_, AppState>) -> Result<(), String> {
    state.host.browser_mcp().disconnect().await;
    Ok(())
}

#[tauri::command]
async fn navigate_browser(url: String, state: State<'_, AppState>) -> Result<Value, String> {
    state.host.browser_mcp().navigate(&url).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_browser_snapshot(state: State<'_, AppState>) -> Result<Value, String> {
    state.host.browser_mcp().snapshot().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn click_browser_element(selector: String, state: State<'_, AppState>) -> Result<Value, String> {
    state.host.browser_mcp().click(&selector).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn fill_browser_element(selector: String, value: String, state: State<'_, AppState>) -> Result<Value, String> {
    state.host.browser_mcp().fill(&selector, &value).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_browser_tabs(state: State<'_, AppState>) -> Result<Value, String> {
    state.host.browser_mcp().list_tabs().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_workflow(
    title: String,
    goal: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let budget = agent_core::WorkflowBudgetConfig::default();
    let (wf_id, _dag) = state
        .host
        .workflow_scheduler()
        .create_workflow(&title, &goal, budget)
        .await
        .map_err(|e| e.to_string())?;
    Ok(wf_id.to_string())
}

#[tauri::command]
async fn list_workflows(state: State<'_, AppState>) -> Result<Vec<agent_storage::Workflow>, String> {
    state.host.database().list_workflows().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_workflow(
    workflow_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let id = Uuid::parse_str(&workflow_id).map_err(|e| e.to_string())?;
    let status = state
        .host
        .workflow_scheduler()
        .resume_workflow(id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(status.to_string())
}

#[tauri::command]
async fn pause_workflow(
    workflow_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let id = Uuid::parse_str(&workflow_id).map_err(|e| e.to_string())?;
    Ok(state.host.workflow_scheduler().pause_workflow(id).await)
}

#[tauri::command]
async fn cancel_workflow(
    workflow_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let id = Uuid::parse_str(&workflow_id).map_err(|e| e.to_string())?;
    Ok(state.host.workflow_scheduler().cancel_workflow(id).await)
}

#[tauri::command]
async fn list_workflow_tasks(
    workflow_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<agent_storage::WorkflowTask>, String> {
    let id = Uuid::parse_str(&workflow_id).map_err(|e| e.to_string())?;
    state.host.database().list_workflow_tasks(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_workflow_artifacts(
    workflow_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<agent_storage::SharedArtifact>, String> {
    let id = Uuid::parse_str(&workflow_id).map_err(|e| e.to_string())?;
    state.host.database().list_shared_artifacts(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_registered_agents(
    state: State<'_, AppState>,
) -> Result<Vec<agent_core::AgentDefinition>, String> {
    let reg = state.host.agent_registry().read().await;
    Ok(reg.list_agents())
}

#[tauri::command]
async fn toggle_agent_status(
    agent_id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let mut reg = state.host.agent_registry().write().await;
    Ok(reg.set_enabled(&agent_id, enabled))
}

#[tauri::command]
async fn set_agent_concurrency(
    agent_id: String,
    limit: usize,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let mut reg = state.host.agent_registry().write().await;
    Ok(reg.set_concurrency(&agent_id, limit))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Initializing Universal Autonomous AI Workspace Host");

    let app_dir = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("AutonomousAiWorkspace");
    let _ = std::fs::create_dir_all(&app_dir);

    let db_path = app_dir.join("workspace.db");
    let mut config = RuntimeConfig::default();
    config.workspace_path = app_dir.join("default_workspace");
    let _ = std::fs::create_dir_all(&config.workspace_path);

    let host = Arc::new(
        RuntimeHost::new(&db_path, config)
            .await
            .expect("Failed to initialize RuntimeHost"),
    );

    // Auto-recover tasks on startup
    let _ = host.recover_tasks_from_checkpoints().await;

    let host_for_events = host.clone();

    tauri::Builder::default()
        .manage(AppState { host })
        .setup(move |app| {
            let handle = app.handle().clone();
            let mut event_rx = host_for_events.subscribe_events();

            // Background task forwarding runtime events to Tauri webview
            tauri::async_runtime::spawn(async move {
                while let Ok((task_id, evt)) = event_rx.recv().await {
                    let payload = json!({
                        "task_id": task_id.to_string(),
                        "event": evt
                    });
                    let _ = handle.emit("agent-event", payload);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard_stats,
            get_system_prerequisites,
            list_conversations,
            create_conversation,
            get_messages,
            send_chat_message,
            list_tasks,
            create_task,
            cancel_task,
            get_task_checkpoints,
            recover_tasks,
            get_workspace_path,
            set_workspace_path,
            list_workspace_files,
            read_workspace_file,
            write_workspace_file,
            execute_terminal_command,
            check_mcp_status,
            connect_mcp,
            disconnect_mcp,
            list_pending_approvals,
            resolve_approval,
            trigger_emergency_stop,
            reset_emergency_stop,
            get_runtime_config,
            save_runtime_config,
            test_provider_connection,
            detect_project,
            get_repo_map,
            preview_diff,
            run_build,
            run_test,
            start_coding_task,
            get_ownership_status,
            acquire_desktop_lease,
            release_desktop_lease,
            user_takeover_desktop,
            release_user_control_desktop,
            list_desktop_windows,
            switch_desktop_window,
            get_desktop_lock_status,
            take_desktop_screenshot,
            connect_browser,
            disconnect_browser,
            navigate_browser,
            get_browser_snapshot,
            click_browser_element,
            fill_browser_element,
            list_browser_tabs,
            create_workflow,
            list_workflows,
            run_workflow,
            pause_workflow,
            cancel_workflow,
            list_workflow_tasks,
            list_workflow_artifacts,
            list_registered_agents,
            toggle_agent_status,
            set_agent_concurrency
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
