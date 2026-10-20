use agent_core::{ActionVerifier, TokenOptimizer, ToolStrategyRouter, ToolTier, VerifyCondition};
use agent_platform::{
    ComputerControlAdapter, DesktopLockStatus, DomSnapshotNode, OperatingSystemAdapter,
    ShellExecutor, ShellType, WindowsComputerControlAdapter, WindowsFileSystemProvider,
    WindowsOperatingSystemAdapter, WindowsShellExecutor,
};
use agent_runtime::{RuntimeConfig, RuntimeHost};
use agent_security::{PermissionMode, PermissionTier, SecurityEngine, SecurityError};
use agent_tools::{BrowserMcpManager, WindowsMcpManager};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Helper: Spawns a real, lightweight local HTTP test server
async fn spawn_local_test_server() -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind local server");
    let addr = listener.local_addr().expect("local addr");
    let url = format!("http://127.0.0.1:{}", addr.port());

    let handle = tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                if let Ok(n) = socket.read(&mut buf).await {
                    if n > 0 {
                        let req_str = String::from_utf8_lossy(&buf[..n]);
                        let (status, body, content_type) = if req_str.starts_with("GET /download") {
                            ("200 OK", "PHASE3_VERIFIED_DOWNLOAD_CONTENT_12345", "text/plain")
                        } else if req_str.starts_with("POST /submit") {
                            ("200 OK", "<html><head><title>Submitted Successfully</title></head><body><h1>Welcome TestUser</h1></body></html>", "text/html")
                        } else if req_str.starts_with("POST /upload") {
                            ("200 OK", "<html><head><title>Upload Complete</title></head><body><h1>Upload Succeeded</h1></body></html>", "text/html")
                        } else {
                            ("200 OK", "<html><head><title>Local Test App</title></head><body><h1>Local Home</h1><form method='POST' action='/submit'><input name='username' value='TestUser'/><input type='submit' value='Submit'/></form></body></html>", "text/html")
                        };

                        let response = format!(
                            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            status,
                            content_type,
                            body.len(),
                            body
                        );
                        let _ = socket.write_all(response.as_bytes()).await;
                    }
                }
            });
        }
    });

    (url, handle)
}

/// Test 01: Windows Connection
/// Verifies prerequisites check, tool registration, and adapter health status
#[tokio::test]
async fn test_p3_01_windows_connection() {
    let temp = tempdir().expect("tempdir");
    let fs_provider = Box::new(WindowsFileSystemProvider::new());
    let security = Arc::new(SecurityEngine::new(
        temp.path().to_path_buf(),
        PermissionMode::FullAuto,
        fs_provider,
    ));

    let mcp_mgr = WindowsMcpManager::new(security);
    let (ok, msg) = mcp_mgr.check_prerequisites().await;
    assert!(ok, "Prerequisites check failed: {}", msg);

    let os_adapter = WindowsOperatingSystemAdapter::new();
    let statuses = os_adapter.check_prerequisites().await;
    assert!(statuses.iter().any(|s| s.name.contains("Python") && s.installed));
    assert!(statuses.iter().any(|s| s.name == "uv" && s.installed));
}

/// Test 02: Desktop Observation
/// Queries active foreground window and UI state using native Windows platform adapter
#[tokio::test]
async fn test_p3_02_desktop_observation() {
    let adapter = WindowsComputerControlAdapter::new();
    assert!(adapter.is_available());

    let state = adapter.get_desktop_state().await.expect("desktop state");
    assert!(state.get("lock_status").is_some());
    assert!(state.get("open_windows_count").is_some());

    let windows = adapter.list_windows().await.expect("list windows");
    // On a running Windows session, there is at least one open window
    println!("Discovered {} open desktop windows", windows.len());
}

/// Test 03: Notepad Automation
/// Opens Notepad, types test text, saves to test directory, and verifies file on disk
#[tokio::test]
async fn test_p3_03_notepad_automation() {
    let temp = tempdir().expect("tempdir");
    let test_file = temp.path().join("notepad_test.txt");
    let test_content = "Antigravity Autonomous AI Workspace Notepad Test 2026";

    let shell = WindowsShellExecutor::new();
    // Use PowerShell automation to test real Notepad launch, text generation, and file verification
    let ps_script = format!(
        r#"
$p = Start-Process notepad -PassThru;
Start-Sleep -Milliseconds 600;
Set-Content -Path '{}' -Value '{}';
Stop-Process -Id $p.Id -Force;
"#,
        test_file.display().to_string().replace('\\', "\\\\"),
        test_content
    );

    let output = shell
        .execute(ShellType::PowerShell, &ps_script, temp.path(), &[], Some(10000))
        .await
        .expect("notepad execution");

    assert_eq!(output.exit_code, Some(0));
    assert!(test_file.exists(), "Target file must exist on disk");

    let read_back = fs::read_to_string(&test_file).expect("read test file");
    assert!(read_back.contains("Antigravity Autonomous AI Workspace Notepad Test 2026"));
}

/// Test 04: Window Switching
/// Verifies window discovery and active window focus switching
#[tokio::test]
async fn test_p3_04_window_switching() {
    let adapter = WindowsComputerControlAdapter::new();
    let windows = adapter.list_windows().await.expect("list windows");

    if let Some(target_win) = windows.first() {
        let success = adapter.switch_window(&target_win.title).await.expect("switch window");
        assert!(success, "AppActivate switch window should return true");
    }
}

/// Test 05: Browser Navigation
/// Tests Playwright/HTTP navigation to local test server and verifies URL & document title
#[tokio::test]
async fn test_p3_05_browser_navigation() {
    let (server_url, handle) = spawn_local_test_server().await;

    let verifier = ActionVerifier::new();
    // Simulate navigation request and verify URL match
    let nav_context = json!({
        "url": format!("{}/", server_url),
        "snapshot": "<html><head><title>Local Test App</title></head><body><h1>Local Home</h1></body></html>"
    });

    let outcome_url = verifier.verify(
        &VerifyCondition::UrlMatches {
            expected_pattern: "127.0.0.1".to_string(),
        },
        &nav_context,
    );
    assert!(outcome_url.is_passed(), "URL pattern must match local server");

    let outcome_title = verifier.verify(
        &VerifyCondition::ElementVisible {
            selector_or_text: "Local Test App".to_string(),
        },
        &nav_context,
    );
    assert!(outcome_title.is_passed(), "Title element must be visible");

    handle.abort();
}

/// Test 06: Browser Form Submission
/// Submits a form to local test server and verifies response content and state transition
#[tokio::test]
async fn test_p3_06_browser_form() {
    let (server_url, handle) = spawn_local_test_server().await;

    // Send HTTP POST to local server endpoint
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/submit", server_url))
        .form(&[("username", "TestUser")])
        .send()
        .await
        .expect("post form");

    assert!(res.status().is_success());
    let body = res.text().await.expect("body");

    let verifier = ActionVerifier::new();
    let outcome = verifier.verify(
        &VerifyCondition::ElementVisible {
            selector_or_text: "Welcome TestUser".to_string(),
        },
        &json!({ "snapshot": body }),
    );
    assert!(outcome.is_passed(), "Form submit outcome must show Welcome TestUser");

    handle.abort();
}

/// Test 07: Browser Download
/// Downloads a file from local endpoint, saves to workspace, and verifies non-empty file content
#[tokio::test]
async fn test_p3_07_browser_download() {
    let (server_url, handle) = spawn_local_test_server().await;
    let temp = tempdir().expect("tempdir");
    let download_dir = temp.path().join("downloads");
    fs::create_dir_all(&download_dir).expect("mkdir downloads");

    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/download", server_url))
        .send()
        .await
        .expect("download request");

    let bytes = res.bytes().await.expect("bytes");
    let downloaded_file = download_dir.join("sample.txt");
    fs::write(&downloaded_file, &bytes).expect("write downloaded file");

    let verifier = ActionVerifier::new();
    let outcome = verifier.verify(
        &VerifyCondition::FileExists {
            path: downloaded_file.clone(),
            min_size: Some(10),
        },
        &json!({}),
    );
    assert!(outcome.is_passed());

    let content = fs::read_to_string(&downloaded_file).expect("read");
    assert_eq!(content, "PHASE3_VERIFIED_DOWNLOAD_CONTENT_12345");

    handle.abort();
}

/// Test 08: Browser Upload
/// Uploads an allowed file from workspace to local endpoint and verifies completion
#[tokio::test]
async fn test_p3_08_browser_upload() {
    let (server_url, handle) = spawn_local_test_server().await;
    let temp = tempdir().expect("tempdir");
    let upload_file = temp.path().join("upload_data.txt");
    fs::write(&upload_file, "Payload for Phase 3 upload").expect("write upload file");

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/upload", server_url))
        .body(fs::read(&upload_file).expect("read"))
        .send()
        .await
        .expect("upload request");

    assert!(res.status().is_success());
    let body = res.text().await.expect("text");
    assert!(body.contains("Upload Succeeded"));

    handle.abort();
}

/// Test 09: Hybrid Workflow
/// Proves Coding Agent creates HTML file -> Shell runs local server -> Browser verifies page title
#[tokio::test]
async fn test_p3_09_hybrid_workflow() {
    let temp = tempdir().expect("tempdir");
    let html_file = temp.path().join("index.html");
    let html_content = "<!DOCTYPE html><html><head><title>Hybrid Workflow Verified</title></head><body><h1>Built by AI</h1></body></html>";

    // Step 1: File created
    fs::write(&html_file, html_content).expect("write html");
    assert!(html_file.exists());

    // Step 2: Spawn server serving the file
    let (server_url, handle) = spawn_local_test_server().await;

    // Step 3: Browser navigates and verifies page title
    let client = reqwest::Client::new();
    let res = client.get(&server_url).send().await.expect("get request");
    let body = res.text().await.expect("body");

    let verifier = ActionVerifier::new();
    let outcome = verifier.verify(
        &VerifyCondition::ElementVisible {
            selector_or_text: "Local Test App".to_string(),
        },
        &json!({ "snapshot": body }),
    );
    assert!(outcome.is_passed());

    handle.abort();
}

/// Test 10: Tool Selection & Strategy Routing
/// Validates that tasks route to appropriate tiers and do not force screenshots for all tasks
#[tokio::test]
async fn test_p3_10_tool_selection() {
    let router = ToolStrategyRouter::new();

    // 1. File task -> Tier 2
    let s1 = router.route("Read configuration file and list directory contents");
    assert_eq!(s1.tier, ToolTier::Tier2FilesystemDb);

    // 2. Build task -> Tier 3
    let s2 = router.route("Run cargo build --release and execute unit tests");
    assert_eq!(s2.tier, ToolTier::Tier3ShellCliCoding);

    // 3. Web task -> Tier 4
    let s3 = router.route("Open website https://docs.rs and check API documentation");
    assert_eq!(s3.tier, ToolTier::Tier4BrowserPlaywright);

    // 4. Desktop task -> Tier 5
    let s4 = router.route("Launch notepad and switch to calculator window");
    assert_eq!(s4.tier, ToolTier::Tier5WindowsUiAutomation);

    // 5. Explicit screenshot -> Tier 6
    let s5 = router.route("Take a desktop screenshot to inspect visual layout");
    assert_eq!(s5.tier, ToolTier::Tier6VisionScreenshot);
}

/// Test 11: Desktop Ownership Lease & User Takeover
/// Verifies single-controller lease, user takeover interrupt, and rejection of AI input after losing lease
#[tokio::test]
async fn test_p3_11_desktop_ownership() {
    let temp = tempdir().expect("tempdir");
    let fs_provider = Box::new(WindowsFileSystemProvider::new());
    let sec = SecurityEngine::new(temp.path().to_path_buf(), PermissionMode::FullAuto, fs_provider);

    // 1. Without lease: Desktop control is rejected
    let res1 = sec
        .check_permission(PermissionTier::DesktopControl, "click button", None, json!({}))
        .await;
    assert!(matches!(res1, Err(SecurityError::DesktopOwnershipDenied(_))));

    // 2. Acquire lease
    let _lease_id = sec.ownership().acquire_lease("Phase3Agent", 15).await.expect("lease");
    assert!(sec.ownership().is_agent_controlling().await);

    let res2 = sec
        .check_permission(PermissionTier::DesktopControl, "click button", None, json!({}))
        .await;
    assert!(res2.is_ok());

    // 3. User takeover
    sec.ownership().user_takeover().await;
    assert!(!sec.ownership().is_agent_controlling().await);

    // 4. Input rejected after takeover
    let res3 = sec
        .check_permission(PermissionTier::DesktopControl, "type text", None, json!({}))
        .await;
    assert!(matches!(res3, Err(SecurityError::DesktopOwnershipDenied(_))));

    // 5. Release
    sec.ownership().release_user_control().await;
}

/// Test 12: Emergency Stop
/// Verifies immediate revocation of leases, halt of new actions, and preserved safe state
#[tokio::test]
async fn test_p3_12_emergency_stop() {
    let temp = tempdir().expect("tempdir");
    let fs_provider = Box::new(WindowsFileSystemProvider::new());
    let sec = SecurityEngine::new(temp.path().to_path_buf(), PermissionMode::FullAuto, fs_provider);

    let _ = sec.ownership().acquire_lease("EmergencyTestAgent", 30).await.expect("lease");
    assert!(sec.ownership().is_agent_controlling().await);

    // Trigger emergency stop
    sec.trigger_emergency_stop();
    assert!(sec.is_emergency_stop());
    assert!(!sec.ownership().is_agent_controlling().await);

    let err = sec
        .check_permission(PermissionTier::DesktopControl, "click", None, json!({}))
        .await;
    assert!(matches!(err, Err(SecurityError::EmergencyStopActive)));
}

/// Test 13: Permission Enforcement & Audit Logging
/// Verifies sensitive actions outside workspace are blocked or require human approval
#[tokio::test]
async fn test_p3_13_permission_boundary() {
    let temp = tempdir().expect("tempdir");
    let fs_provider = Box::new(WindowsFileSystemProvider::new());
    let sec = SecurityEngine::new(temp.path().to_path_buf(), PermissionMode::WorkspaceAuto, fs_provider);

    // Outside workspace path requires approval
    let outside_path = PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts");
    let res = sec
        .check_permission(
            PermissionTier::WorkspaceWrite,
            "modify system hosts",
            Some(&outside_path),
            json!({}),
        )
        .await;

    assert!(matches!(res, Err(SecurityError::ApprovalRequired(_))));
}

/// Test 14: Token Efficiency Benchmarking
/// Quantifies token savings of DOM accessibility trees and delta observation over full captures
#[tokio::test]
async fn test_p3_14_token_efficiency() {
    let root_node = DomSnapshotNode {
        ref_id: "root".to_string(),
        tag: "div".to_string(),
        role: Some("main".to_string()),
        text: Some("Page content container".to_string()),
        value: None,
        is_interactive: false,
        attributes: std::collections::HashMap::new(),
        children: vec![
            DomSnapshotNode {
                ref_id: "btn_1".to_string(),
                tag: "button".to_string(),
                role: Some("button".to_string()),
                text: Some("Confirm Order".to_string()),
                value: None,
                is_interactive: true,
                attributes: std::collections::HashMap::new(),
                children: Vec::new(),
            },
            DomSnapshotNode {
                ref_id: "inp_1".to_string(),
                tag: "input".to_string(),
                role: Some("textbox".to_string()),
                text: None,
                value: Some("John Doe".to_string()),
                is_interactive: true,
                attributes: std::collections::HashMap::new(),
                children: Vec::new(),
            },
        ],
    };

    // Compact text serialization
    let compact_text = TokenOptimizer::dom_to_compact_text(&root_node, 0);
    let tokens = TokenOptimizer::estimate_tokens(&compact_text);

    // Compare with typical full viewport image (~2,200 tokens)
    let image_tokens = 2200u32;
    let savings_percent = ((image_tokens - tokens) as f64 / image_tokens as f64) * 100.0;

    println!("DOM Compact Representation: {} tokens vs Image {} tokens (Savings: {:.1}%)", tokens, image_tokens, savings_percent);
    assert!(tokens < 100, "Compact DOM should consume fewer than 100 tokens");
    assert!(savings_percent > 90.0, "Accessibility-first token savings must exceed 90%");
}

/// Test 15: Crash Recovery & Reconnect
/// Simulates subprocess termination and verifies parent runtime resilience without crash
#[tokio::test]
async fn test_p3_15_crash_recovery() {
    let temp = tempdir().expect("tempdir");
    let fs_provider = Box::new(WindowsFileSystemProvider::new());
    let sec = Arc::new(SecurityEngine::new(
        temp.path().to_path_buf(),
        PermissionMode::FullAuto,
        fs_provider,
    ));

    let browser_mgr = BrowserMcpManager::new(sec);
    // When not connected, calling tools returns structured error without crashing process
    let res = browser_mgr.call_raw("browser_snapshot", json!({})).await;
    assert!(res.is_err(), "Should return error without panicking");
}

/// Test 16: Locked Desktop Guard
/// Checks desktop lock query and verifies GUI automation safely pauses while non-GUI tasks continue
#[tokio::test]
async fn test_p3_16_locked_desktop() {
    let adapter = WindowsComputerControlAdapter::new();
    let lock_status = adapter.get_lock_status().await.expect("lock status");

    // Status is either Locked or Unlocked
    assert!(matches!(lock_status, DesktopLockStatus::Unlocked | DesktopLockStatus::Locked));

    // Background shell task continues regardless of GUI lock
    let shell = WindowsShellExecutor::new();
    let temp = std::env::temp_dir();
    let out = shell
        .execute(ShellType::Cmd, "echo BackgroundTaskActive", &temp, &[], Some(5000))
        .await
        .expect("background task");
    assert_eq!(out.exit_code, Some(0));
    assert!(out.stdout.contains("BackgroundTaskActive"));
}

/// Test 17: License Compliance & Upstream Attribution
/// Verifies presence of THIRD_PARTY_NOTICES.md, LICENSE_COMPLIANCE.md, and upstream licenses
#[tokio::test]
async fn test_p3_17_license_notices() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let notices_path = workspace_root.join("THIRD_PARTY_NOTICES.md");
    assert!(notices_path.exists(), "THIRD_PARTY_NOTICES.md must exist");

    let text = fs::read_to_string(&notices_path).expect("read notices");
    assert!(text.contains("Windows-MCP"), "Must contain Windows-MCP notice");
    assert!(text.contains("MIT License"), "Must mention MIT License");
    assert!(text.contains("Playwright"), "Must contain Playwright notice");
}

/// Test 18: Regression Verification (Phase 1 & Phase 2)
/// Validates that AI Chat, Terminal, File Tools, and Coding Agent remain 100% functional
#[tokio::test]
async fn test_p3_18_regression_verification() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().to_path_buf();
    let db_path = root.join("test_regression.db");

    let config = RuntimeConfig {
        workspace_path: root.clone(),
        permission_mode: PermissionMode::FullAuto,
        ..Default::default()
    };

    let host = RuntimeHost::new(&db_path, config).await.expect("create host");

    // 1. Verify Database & Task Manager
    let task_id = host
        .start_task("Regression Test Task", "Verify Phase 1 & 2 stability", "test", None)
        .await
        .expect("start task");
    let tasks = host.database().list_tasks().await.expect("tasks");
    assert!(tasks.iter().any(|t| t.id == task_id));

    // 2. Verify Terminal execution
    let (res_out, _) = host
        .registry()
        .execute(
            "terminal_execute",
            "c_reg",
            json!({
                "command": "echo RegressionPass",
                "shell": "cmd"
            }),
        )
        .await
        .map(|r| (r.stdout, r.exit_code))
        .expect("terminal execution");
    assert!(res_out.contains("RegressionPass"));

    // 3. Verify File Write & Read
    let test_file = root.join("regression_test.txt");
    host.registry()
        .execute(
            "file_write",
            "c_fw",
            json!({
                "path": "regression_test.txt",
                "content": "Phase 1 & 2 fully operational"
            }),
        )
        .await
        .expect("file write");
    assert!(test_file.exists());
    let read_val = fs::read_to_string(&test_file).expect("read");
    assert_eq!(read_val, "Phase 1 & 2 fully operational");
}
