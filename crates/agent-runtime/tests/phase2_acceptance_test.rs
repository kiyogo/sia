use agent_core::{
    deduplicate_diagnostics, ChatMessage, CodingMode, CodingWorkflowEngine,
    ContextBudgetConfig, FunctionCall, MockModelProvider, ModelResponse, ModelUsage,
    TokenOptimizer, ToolCall,
};
use agent_platform::{
    DiagnosticMessage, DiagnosticReport, DiagnosticSeverity, LanguageToolingAdapter,
    MultiLanguageAdapter, WindowsFileSystemProvider, WindowsShellExecutor,
};
use agent_runtime::{RuntimeConfig, RuntimeHost};
use agent_security::{PermissionMode, SecurityEngine};
use agent_storage::Database;
use agent_tools::{compute_unified_diff, create_default_registry};
use serde_json::json;
use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tempfile::tempdir;

/// Test 01: Project Discovery & Intelligence (Rust, Git branch, entry points)
#[tokio::test]
async fn test_p2_01_project_discovery() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path();

    // Setup dummy Rust project with Git
    fs::write(root.join("Cargo.toml"), "[package]\nname=\"sample\"\nversion=\"0.1.0\"\n").expect("write");
    let src = root.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(src.join("main.rs"), "fn main() {}\n").expect("write main");

    let git_dir = root.join(".git");
    fs::create_dir_all(&git_dir).expect("mkdir git");
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/feature/autonomous-agent\n").expect("write head");

    let db_path = temp.path().join("db.sqlite");
    let mut config = RuntimeConfig::default();
    config.workspace_path = root.to_path_buf();
    let host = RuntimeHost::new(&db_path, config).await.expect("host");

    let info = host.detect_project(None).await.expect("detect project");
    assert_eq!(info.primary_language.as_deref(), Some("Rust"));
    assert!(info.languages.contains(&"Rust".to_string()));
    assert!(info.build_systems.contains(&"cargo".to_string()));
    assert_eq!(info.is_git, true);
    assert_eq!(info.git_branch.as_deref(), Some("feature/autonomous-agent"));
    assert!(info.entry_points.iter().any(|p| p.ends_with("main.rs")));
}

/// Test 02: Compact Repo Map within token budget
#[tokio::test]
async fn test_p2_02_compact_repo_map() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path();

    // Create real code files and ignored build caches
    let src = root.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(src.join("auth.rs"), "pub struct AuthSession;\npub fn login() {}\n").expect("write auth");
    fs::write(src.join("user.rs"), "pub struct UserProfile;\npub fn get_user() {}\n").expect("write user");

    // Target directory (should be skipped)
    let target = root.join("target").join("debug");
    fs::create_dir_all(&target).expect("mkdir target");
    fs::write(target.join("ignored.rs"), "pub fn should_not_appear() {}\n").expect("write target");

    let db_path = temp.path().join("db.sqlite");
    let mut config = RuntimeConfig::default();
    config.workspace_path = root.to_path_buf();
    let host = RuntimeHost::new(&db_path, config).await.expect("host");

    let repo_map = host.get_repo_map(Some(2000), None).await.expect("repo map");
    assert!(repo_map.contains("auth.rs"));
    assert!(repo_map.contains("AuthSession"));
    assert!(repo_map.contains("user.rs"));
    assert!(!repo_map.contains("ignored.rs"));
}

/// Test 03: Symbol Search and File Outline
#[tokio::test]
async fn test_p2_03_symbol_search_and_outline() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().to_path_buf();
    let fs_prov = Box::new(WindowsFileSystemProvider::new());
    let sec = Arc::new(SecurityEngine::new(root.clone(), PermissionMode::FullAuto, fs_prov));
    let executor = Box::new(WindowsShellExecutor::new());
    let registry = create_default_registry(executor, sec);

    let file_path = root.join("service.rs");
    fs::write(
        &file_path,
        "pub struct OrderService;\nimpl OrderService {\n    pub fn process_payment(amount: u64) -> bool {\n        true\n    }\n}\n",
    )
    .expect("write service");

    // Outline
    let outline_res = registry
        .execute("file_outline", "call_1", json!({ "file": "service.rs" }))
        .await
        .expect("outline");
    assert!(outline_res.stdout.contains("OrderService"));
    assert!(outline_res.stdout.contains("process_payment"));

    // Code Search
    let search_res = registry
        .execute("code_search", "call_2", json!({ "pattern": "process_payment" }))
        .await
        .expect("search");
    assert!(search_res.stdout.contains("service.rs"));
    assert!(search_res.stdout.contains("process_payment"));
}

/// Test 04: Git Status & Diff Inspection (Safety Checks)
#[tokio::test]
async fn test_p2_04_git_safety_and_diff() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().to_path_buf();
    let fs_prov = Box::new(WindowsFileSystemProvider::new());
    let sec = Arc::new(SecurityEngine::new(root.clone(), PermissionMode::FullAuto, fs_prov));
    let executor = Box::new(WindowsShellExecutor::new());
    let registry = create_default_registry(executor, sec);

    // Call git_status on non-git dir (should handle gracefully)
    let status_res = registry.execute("git_status", "call_git", json!({})).await.expect("git status");
    // Should run command and report exit status without crashing
    assert!(status_res.stdout.contains("is_clean") || !status_res.stderr.is_empty());
}

/// Test 05: OpenCode-style Exact Match Code Edit
#[tokio::test]
async fn test_p2_05_opencode_exact_edit() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().to_path_buf();
    let fs_prov = Box::new(WindowsFileSystemProvider::new());
    let sec = Arc::new(SecurityEngine::new(root.clone(), PermissionMode::FullAuto, fs_prov));
    let executor = Box::new(WindowsShellExecutor::new());
    let registry = create_default_registry(executor, sec);

    let file_path = root.join("calculator.ts");
    fs::write(
        &file_path,
        "function add(a: number, b: number): number {\n  return a - b;\n}\n",
    )
    .expect("write calc");

    // Exact string replacement
    let edit_res = registry
        .execute(
            "code_edit",
            "call_edit",
            json!({
                "path": "calculator.ts",
                "old_string": "return a - b;",
                "new_string": "return a + b;"
            }),
        )
        .await
        .expect("edit");

    assert!(edit_res.stdout.contains("diff"));
    assert!(edit_res.stdout.contains("+  return a + b;"));

    let content = fs::read_to_string(&file_path).expect("read");
    assert!(content.contains("return a + b;"));

    // Multi-occurrence ambiguity error check
    fs::write(&file_path, "let x = 1;\nlet x = 1;\n").expect("write dup");
    let ambig_res = registry
        .execute(
            "code_edit",
            "call_ambig",
            json!({
                "path": "calculator.ts",
                "old_string": "let x = 1;",
                "new_string": "let x = 2;",
                "replace_all": false
            }),
        )
        .await;

    assert!(ambig_res.is_err());
    let err_msg = ambig_res.unwrap_err().to_string();
    assert!(err_msg.contains("occurs 2 times") || err_msg.contains("unique"));
}

/// Test 06: Atomic Multi-File Patch with Complete Rollback
#[tokio::test]
async fn test_p2_06_atomic_patch_rollback() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().to_path_buf();
    let fs_prov = Box::new(WindowsFileSystemProvider::new());
    let sec = Arc::new(SecurityEngine::new(root.clone(), PermissionMode::FullAuto, fs_prov));
    let executor = Box::new(WindowsShellExecutor::new());
    let registry = create_default_registry(executor, sec);

    let file_a = root.join("alpha.rs");
    let file_b = root.join("beta.rs");
    fs::write(&file_a, "Alpha Original\n").expect("write a");
    fs::write(&file_b, "Beta Original\n").expect("write b");

    // Patch attempts to write file_a and an illegal outside path
    let patch_res = registry
        .execute(
            "code_patch",
            "call_patch",
            json!({
                "edits": [
                    { "path": "alpha.rs", "content": "Alpha Corrupted\n" },
                    { "path": "../../outside_sandbox.rs", "content": "Exploit\n" }
                ]
            }),
        )
        .await;

    assert!(patch_res.is_err());
    // Verify alpha.rs is strictly preserved in original form
    let content_a = fs::read_to_string(&file_a).expect("read a");
    assert_eq!(content_a, "Alpha Original\n");
}

/// Test 07: Rust Compiler Error Parsing & Diagnostic Extraction
#[tokio::test]
async fn test_p2_07_rust_compiler_diagnostics() {
    let adapter = MultiLanguageAdapter::new();
    let rustc_stderr = r#"
error[E0308]: mismatched types
  --> src/main.rs:24:18
   |
24 |     let x: u32 = "string";
   |            ---   ^^^^^^^^ expected `u32`, found `&str`
   |            |
   |            expected due to this
"#;

    let report = adapter.parse_diagnostics("", rustc_stderr);
    assert_eq!(report.total_errors, 1);
    let root = report.root_cause.expect("root cause");
    assert_eq!(root.file, "src/main.rs");
    assert_eq!(root.line, 24);
    assert_eq!(root.column, 18);
    assert_eq!(root.code.as_deref(), Some("E0308"));
    assert!(root.message.contains("mismatched types"));
}

/// Test 08: TypeScript and Python Diagnostic Parsers
#[tokio::test]
async fn test_p2_08_ts_and_python_diagnostics() {
    let adapter = MultiLanguageAdapter::new();

    // TypeScript tsc output
    let ts_output = "src/components/Header.tsx:18:5 - error TS2554: Expected 2 arguments, but got 1.\n";
    let ts_report = adapter.parse_diagnostics(ts_output, "");
    assert_eq!(ts_report.total_errors, 1);
    let ts_root = ts_report.root_cause.expect("ts root");
    assert_eq!(ts_root.file, "src/components/Header.tsx");
    assert_eq!(ts_root.line, 18);
    assert_eq!(ts_root.column, 5);
    assert_eq!(ts_root.code.as_deref(), Some("TS2554"));

    // Python Traceback / SyntaxError
    let py_output = "Traceback (most recent call last):\n  File \"app/server.py\", line 42, in start\nKeyError: 'PORT'\n";
    let py_report = adapter.parse_diagnostics(py_output, "");
    assert_eq!(py_report.total_errors, 1);
    let py_root = py_report.root_cause.expect("py root");
    assert_eq!(py_root.file, "app/server.py");
    assert_eq!(py_root.line, 42);
    assert!(py_root.message.contains("KeyError"));
}

/// Test 09: Token Optimization — Error Deduplication
#[tokio::test]
async fn test_p2_09_token_optimization_dedup() {
    let report = DiagnosticReport {
        diagnostics: vec![
            DiagnosticMessage {
                file: "src/engine.rs".to_string(),
                line: 50,
                column: 8,
                severity: DiagnosticSeverity::Error,
                message: "cannot find type `EngineConfig` in this scope".to_string(),
                code: Some("E0412".to_string()),
                context_snippet: Some("config: EngineConfig".to_string()),
            },
            // Duplicate cascading error at same site
            DiagnosticMessage {
                file: "src/engine.rs".to_string(),
                line: 50,
                column: 8,
                severity: DiagnosticSeverity::Error,
                message: "type `EngineConfig` is not defined".to_string(),
                code: Some("E0412".to_string()),
                context_snippet: None,
            },
            DiagnosticMessage {
                file: "src/engine.rs".to_string(),
                line: 82,
                column: 4,
                severity: DiagnosticSeverity::Error,
                message: "unresolved import".to_string(),
                code: Some("E0432".to_string()),
                context_snippet: None,
            },
        ],
        root_cause: Some(DiagnosticMessage {
            file: "src/engine.rs".to_string(),
            line: 50,
            column: 8,
            severity: DiagnosticSeverity::Error,
            message: "cannot find type `EngineConfig` in this scope".to_string(),
            code: Some("E0412".to_string()),
            context_snippet: Some("config: EngineConfig".to_string()),
        }),
        total_errors: 3,
        total_warnings: 2,
        raw_output: String::new(),
    };

    let dedup = deduplicate_diagnostics(&report, 5);
    // Duplicate at line 50 was filtered out
    assert_eq!(dedup.primary_errors.len(), 2);
    assert!(dedup.summary_prompt.contains("cannot find type `EngineConfig`"));
    assert!(dedup.summary_prompt.contains("unresolved import"));
    assert!(dedup.summary_prompt.contains("2 warning(s) omitted"));
}

/// Test 10: Coding Modes Tool Exposure Isolation (Ask vs Plan vs Build)
#[tokio::test]
async fn test_p2_10_coding_modes_isolation() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().to_path_buf();
    let fs_prov = Box::new(WindowsFileSystemProvider::new());
    let sec = Arc::new(SecurityEngine::new(root.clone(), PermissionMode::FullAuto, fs_prov));
    let executor = Box::new(WindowsShellExecutor::new());
    let registry = Arc::new(create_default_registry(executor, sec.clone()));
    let db = Database::open_in_memory().expect("db");
    let optimizer = Arc::new(TokenOptimizer::new(ContextBudgetConfig::default()));
    let provider = Arc::new(MockModelProvider::new(vec![]));

    let (engine, _) = CodingWorkflowEngine::new(provider, registry, sec, db, optimizer, 10);

    // Ask mode: inspection only
    let ask_tools = engine.get_tools_for_mode(CodingMode::Ask).await;
    assert!(ask_tools.iter().any(|t| t.name == "repo_map"));
    assert!(ask_tools.iter().any(|t| t.name == "code_search"));
    assert!(!ask_tools.iter().any(|t| t.name == "code_edit"));
    assert!(!ask_tools.iter().any(|t| t.name == "code_patch"));
    assert!(!ask_tools.iter().any(|t| t.name == "run_build"));

    // Plan mode: inspection + diff preview
    let plan_tools = engine.get_tools_for_mode(CodingMode::Plan).await;
    assert!(plan_tools.iter().any(|t| t.name == "diff_preview"));
    assert!(!plan_tools.iter().any(|t| t.name == "code_edit"));
    assert!(!plan_tools.iter().any(|t| t.name == "run_build"));

    // Build mode: full toolset
    let build_tools = engine.get_tools_for_mode(CodingMode::Build).await;
    assert!(build_tools.iter().any(|t| t.name == "code_edit"));
    assert!(build_tools.iter().any(|t| t.name == "code_patch"));
    assert!(build_tools.iter().any(|t| t.name == "run_build"));
    assert!(build_tools.iter().any(|t| t.name == "run_test"));
}

/// Test 11: Scoped Workspace Sandbox Enforcement
#[tokio::test]
async fn test_p2_11_scoped_workspace_sandbox() {
    let temp = tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(&workspace).expect("mkdir");

    let outside_file = temp.path().join("forbidden_system_file.txt");
    fs::write(&outside_file, "Sensitive Data\n").expect("write outside");

    let fs_prov = Box::new(WindowsFileSystemProvider::new());
    let sec = Arc::new(SecurityEngine::new(workspace.clone(), PermissionMode::Strict, fs_prov));
    let executor = Box::new(WindowsShellExecutor::new());
    let registry = create_default_registry(executor, sec);

    let res = registry
        .execute(
            "code_edit",
            "call_hack",
            json!({
                "path": "../forbidden_system_file.txt",
                "old_string": "Sensitive Data",
                "new_string": "Hacked Data"
            }),
        )
        .await;

    assert!(res.is_err());
    // File outside workspace must not be modified
    let content = fs::read_to_string(&outside_file).expect("read");
    assert_eq!(content, "Sensitive Data\n");
}

/// Test 12: Autonomous Self-Correction Loop
#[tokio::test]
async fn test_p2_12_autonomous_self_correction() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().to_path_buf();

    // Create a buggy file
    let bug_file = root.join("math.rs");
    fs::write(&bug_file, "pub fn multiply(a: i32, b: i32) -> i32 {\n    a + b\n}\n").expect("write");

    let fs_prov = Box::new(WindowsFileSystemProvider::new());
    let sec = Arc::new(SecurityEngine::new(root.clone(), PermissionMode::FullAuto, fs_prov));
    let executor = Box::new(WindowsShellExecutor::new());
    let registry = Arc::new(create_default_registry(executor, sec.clone()));
    let db = Database::open_in_memory().expect("db");
    let optimizer = Arc::new(TokenOptimizer::new(ContextBudgetConfig::default()));

    // Program Mock Model:
    // Turn 1: Inspects bug and calls code_edit to replace `a + b` with `a * b`
    // Turn 2: Returns final confirmation that the fix succeeded
    let turn1 = ModelResponse {
        content: Some("I have identified the bug in multiply function. Applying targeted fix.".to_string()),
        tool_calls: vec![ToolCall {
            id: "call_fix".to_string(),
            r#type: "function".to_string(),
            function: FunctionCall {
                name: "code_edit".to_string(),
                arguments: json!({
                    "path": "math.rs",
                    "old_string": "a + b",
                    "new_string": "a * b"
                })
                .to_string(),
            },
        }],
        usage: ModelUsage { prompt_tokens: 30, completion_tokens: 25, total_tokens: 55, cached_tokens: None },
    };

    let turn2 = ModelResponse {
        content: Some("Code has been successfully fixed and verified.".to_string()),
        tool_calls: vec![],
        usage: ModelUsage { prompt_tokens: 45, completion_tokens: 15, total_tokens: 60, cached_tokens: None },
    };

    let provider = Arc::new(MockModelProvider::new(vec![turn1, turn2]));
    let (engine, _) = CodingWorkflowEngine::new(provider, registry, sec, db.clone(), optimizer, 5);

    let mut task = agent_storage::Task {
        id: uuid::Uuid::new_v4(),
        conversation_id: None,
        title: "Self-Correction Test".to_string(),
        status: agent_storage::TaskStatus::Pending,
        goal: "Fix multiplication bug".to_string(),
        current_step: "Pending".to_string(),
        result: None,
        error: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    db.create_task(&task).await.expect("create task");

    let final_answer = engine
        .run(&mut task, CodingMode::Build, "Fix math.rs multiply function", Arc::new(AtomicBool::new(false)))
        .await
        .expect("run coding task");

    assert!(final_answer.contains("successfully fixed"));
    let updated_content = fs::read_to_string(&bug_file).expect("read");
    assert!(updated_content.contains("a * b"));
}
