// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use agent_core::{AgentEvent, ContextBudgetConfig};
use agent_platform::{OperatingSystemAdapter, ProcessOutput, ShellType, WindowsOperatingSystemAdapter};
use agent_security::{ApprovalRequest, PermissionMode, PermissionTier};
use agent_storage::{Checkpoint, Conversation, Message, Task};
use agent_tools::{McpToolInfo, ToolDefinition};
use agent_runtime::{DashboardStats, RecoveredTaskInfo, RuntimeConfig, RuntimeHost};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;
use tracing::{error, info};
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
            test_provider_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
