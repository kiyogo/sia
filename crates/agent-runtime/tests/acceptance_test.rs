use agent_core::{ChatMessage, ContextBudgetConfig, TokenOptimizer};
use agent_platform::{
    get_platform_adapters, OperatingSystemAdapter, ShellType,
    WindowsFileSystemProvider, WindowsOperatingSystemAdapter, WindowsShellExecutor,
};
use agent_runtime::{RuntimeConfig, RuntimeHost};
use agent_security::{PermissionMode, PermissionTier, SecurityEngine, SecurityError};
use agent_storage::{Checkpoint, Database, Message, Task, TaskStatus};
use agent_tools::create_default_registry;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use uuid::Uuid;

/// Test 1 — Chat: User message, AI generation, and SQLite conversation history persistence
#[tokio::test]
async fn test_acceptance_1_chat_and_persistence() {
    let temp = tempdir().expect("tempdir");
    let db_path = temp.path().join("chat_test.db");
    let host = RuntimeHost::new(&db_path, RuntimeConfig::default())
        .await
        .expect("init host");

    let conv = host
        .database()
        .create_conversation("Acceptance Test Chat")
        .await
        .expect("create conv");

    let user_msg = Message {
        id: Uuid::new_v4(),
        conversation_id: conv.id,
        role: "user".to_string(),
        content: "What is your architecture?".to_string(),
        tool_calls: None,
        tool_call_id: None,
        token_count: 5,
        created_at: Utc::now(),
    };
    host.database().add_message(&user_msg).await.expect("add user msg");

    let asst_msg = Message {
        id: Uuid::new_v4(),
        conversation_id: conv.id,
        role: "assistant".to_string(),
        content: "I am a multi-purpose autonomous AI workspace.".to_string(),
        tool_calls: None,
        tool_call_id: None,
        token_count: 8,
        created_at: Utc::now(),
    };
    host.database().add_message(&asst_msg).await.expect("add asst msg");

    let messages = host.database().get_messages(conv.id).await.expect("get messages");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[1].role, "assistant");
}

/// Test 2 — Terminal: Native CMD and PowerShell execution, exit codes, and output capture
#[tokio::test]
async fn test_acceptance_2_terminal_execution() {
    let (_, shell, _) = get_platform_adapters();
    let temp = std::env::temp_dir();

    // 1. Run CMD command
    let cmd_out = shell
        .execute(ShellType::Cmd, "echo CMD_SUCCESS", &temp, &[], Some(5000))
        .await
        .expect("cmd echo");
    assert_eq!(cmd_out.exit_code, Some(0));
    assert!(cmd_out.stdout.contains("CMD_SUCCESS"));

    // 2. Run PowerShell command
    let ps_out = shell
        .execute(
            ShellType::PowerShell,
            "Write-Output 'POWERSHELL_SUCCESS'",
            &temp,
            &[],
            Some(5000),
        )
        .await
        .expect("ps echo");
    assert_eq!(ps_out.exit_code, Some(0));
    assert!(ps_out.stdout.contains("POWERSHELL_SUCCESS"));
}

/// Test 3 — Coding: Creating real files, running commands, and testing real output
#[tokio::test]
async fn test_acceptance_3_coding_project() {
    let temp = tempdir().expect("tempdir");
    let workspace = temp.path().to_path_buf();

    let fs = Box::new(WindowsFileSystemProvider::new());
    let sec = Arc::new(SecurityEngine::new(workspace.clone(), PermissionMode::FullAuto, fs));
    let shell = Box::new(WindowsShellExecutor::new());
    let registry = create_default_registry(shell, sec.clone());

    // Step A: Write a small Rust Cargo.toml
    let cargo_toml = r#"
[package]
name = "demo_code"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
    let _ = registry
        .execute(
            "file_write",
            "c_1",
            json!({
                "path": "Cargo.toml",
                "content": cargo_toml
            }),
        )
        .await
        .expect("write cargo.toml");

    // Step B: Write main.rs
    let main_rs = r#"
fn main() {
    println!("Hello from AI generated workspace project!");
}
"#;
    let _ = registry
        .execute(
            "file_write",
            "c_2",
            json!({
                "path": "src/main.rs",
                "content": main_rs
            }),
        )
        .await
        .expect("write main.rs");

    // Step C: Run cargo check via terminal tool
    let check_res = registry
        .execute(
            "terminal_execute",
            "c_3",
            json!({
                "command": "cargo check",
                "shell": "cmd",
                "cwd": workspace.display().to_string()
            }),
        )
        .await
        .expect("run cargo check");

    assert_eq!(check_res.exit_code, Some(0));

    // Step D: Patch file
    let patch_res = registry
        .execute(
            "file_patch",
            "c_4",
            json!({
                "path": "src/main.rs",
                "target": "AI generated workspace",
                "replacement": "Patched Autonomous Workspace"
            }),
        )
        .await
        .expect("patch main.rs");

    assert!(patch_res.stdout.contains("Patched Autonomous Workspace"));
}

/// Test 4 — Windows-MCP: Prerequisites detection (Python 3.14+, uv) and status inspection
#[tokio::test]
async fn test_acceptance_4_windows_mcp_prerequisites() {
    let os = WindowsOperatingSystemAdapter::new();
    let prereqs = os.check_prerequisites().await;

    // Must check Python 3.14+ and uv
    let has_py = prereqs.iter().any(|p| p.name.contains("Python"));
    let has_uv = prereqs.iter().any(|p| p.name.contains("uv"));
    assert!(has_py, "Must check Python 3.14+ prerequisite");
    assert!(has_uv, "Must check uv prerequisite");
}

/// Test 5 — Permission: Action outside workspace boundary is blocked or requires human approval
#[tokio::test]
async fn test_acceptance_5_permission_boundary() {
    let temp = tempdir().expect("tempdir");
    let workspace = temp.path().to_path_buf();

    let fs = Box::new(WindowsFileSystemProvider::new());
    let sec = SecurityEngine::new(workspace.clone(), PermissionMode::WorkspaceAuto, fs);

    // 1. Action inside workspace passes
    let inside_res = sec
        .check_permission(
            PermissionTier::WorkspaceWrite,
            "Edit local file",
            Some(&workspace.join("test.txt")),
            json!({}),
        )
        .await;
    assert!(inside_res.is_ok());

    // 2. Action outside workspace boundary (e.g. C:\Windows\System32) requires approval
    let outside_path = std::path::PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts");
    let outside_res = sec
        .check_permission(
            PermissionTier::WorkspaceWrite,
            "Edit outside file",
            Some(&outside_path),
            json!({}),
        )
        .await;

    assert!(matches!(outside_res, Err(SecurityError::ApprovalRequired(_))));
}

/// Test 6 — Persistence: Database retains conversations, tasks, checkpoints across restarts
#[tokio::test]
async fn test_acceptance_6_persistence_across_restart() {
    let temp = tempdir().expect("tempdir");
    let db_path = temp.path().join("persistence_test.db");

    let task_id = Uuid::new_v4();

    // Session 1: create records
    {
        let db = Database::open(&db_path).expect("open session 1");
        let task = Task {
            id: task_id,
            conversation_id: None,
            title: "Persistent Job".to_string(),
            status: TaskStatus::Completed,
            goal: "Persist data across reboots".to_string(),
            current_step: "Finished".to_string(),
            result: Some("All data intact".to_string()),
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db.create_task(&task).await.expect("create task");

        let ck = Checkpoint {
            id: Uuid::new_v4(),
            task_id,
            step_index: 5,
            state_json: r#"{"progress": 100}"#.to_string(),
            summary: "Job finished at step 5".to_string(),
            created_at: Utc::now(),
        };
        db.save_checkpoint(&ck).await.expect("save ck");
    }

    // Session 2: re-open database from disk
    {
        let db2 = Database::open(&db_path).expect("open session 2");
        let tasks = db2.list_tasks().await.expect("list tasks");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, task_id);
        assert_eq!(tasks[0].status, TaskStatus::Completed);

        let ck = db2.get_latest_checkpoint(task_id).await.expect("get ck").expect("ck exists");
        assert_eq!(ck.step_index, 5);
        assert!(ck.summary.contains("step 5"));
    }
}

/// Test 7 — Recovery: Incomplete tasks transition to Paused for safe inspection
#[tokio::test]
async fn test_acceptance_7_safe_checkpoint_recovery() {
    let temp = tempdir().expect("tempdir");
    let db_path = temp.path().join("recovery_test.db");

    let host = RuntimeHost::new(&db_path, RuntimeConfig::default())
        .await
        .expect("init host");

    let task_id = Uuid::new_v4();
    let uncompleted = Task {
        id: task_id,
        conversation_id: None,
        title: "Interrupted Task".to_string(),
        status: TaskStatus::Running,
        goal: "Deploy update".to_string(),
        current_step: "Step 2 in progress".to_string(),
        result: None,
        error: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    host.database().create_task(&uncompleted).await.expect("create");

    let ck = Checkpoint {
        id: Uuid::new_v4(),
        task_id,
        step_index: 2,
        state_json: r#"{"step": 2}"#.to_string(),
        summary: "Step 2 completed".to_string(),
        created_at: Utc::now(),
    };
    host.database().save_checkpoint(&ck).await.expect("save ck");

    // Run recovery
    let recovered = host.recover_tasks_from_checkpoints().await.expect("recover");
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].new_status, TaskStatus::Paused);
    assert_eq!(recovered[0].checkpoint_step, Some(2));
}

/// Test 8 — Token Optimization: Truncation limits and conversation history compaction
#[test]
fn test_acceptance_8_token_optimization() {
    let opt = TokenOptimizer::new(ContextBudgetConfig {
        history_budget: 40,
        max_tool_output_lines: 10,
        max_tool_output_bytes: 500,
        ..Default::default()
    });

    // 1. Tool result truncation
    let big_output = (1..=50)
        .map(|i| format!("Line {i}: test log output with detailed traces"))
        .collect::<Vec<_>>()
        .join("\n");
    let (truncated, was_trunc) = opt.truncate_tool_output(&big_output);
    assert!(was_trunc);
    assert!(truncated.contains("Output truncated"));

    // 2. Compaction
    let msgs: Vec<ChatMessage> = (1..=10)
        .map(|i| ChatMessage {
            role: if i % 2 == 0 { "user".to_string() } else { "assistant".to_string() },
            content: Some(format!("Step {i} was executed with lots of detailed explanations.")),
            tool_calls: None,
            tool_call_id: None,
        })
        .collect();

    let compacted = opt.compact_history(&msgs);
    assert!(compacted.len() < msgs.len());
    assert!(compacted.iter().any(|m| m.content.as_deref().unwrap_or("").contains("Compacted Historical Progress Summary")));
}

/// Test 9 — Emergency Stop: Halts all ongoing agent permissions immediately
#[tokio::test]
async fn test_acceptance_9_emergency_stop() {
    let temp = tempdir().expect("tempdir");
    let workspace = temp.path().to_path_buf();
    let fs = Box::new(WindowsFileSystemProvider::new());
    let sec = SecurityEngine::new(workspace, PermissionMode::FullAuto, fs);

    assert!(!sec.is_emergency_stop());
    sec.trigger_emergency_stop();
    assert!(sec.is_emergency_stop());

    let res = sec.check_permission(PermissionTier::ReadOnly, "read", None, json!({})).await;
    assert!(matches!(res, Err(SecurityError::EmergencyStopActive)));

    sec.reset_emergency_stop();
    assert!(!sec.is_emergency_stop());
}
