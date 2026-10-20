pub mod agent_loop;
pub mod coding_loop;
pub mod diagnostics;
pub mod error;
pub mod model;
pub mod token_optimizer;
pub mod tool_router;
pub mod verifier;

pub use agent_loop::{AgentEvent, AgentLoop};
pub use coding_loop::{CodingMode, CodingWorkflowEngine};
pub use diagnostics::{deduplicate_diagnostics, DeduplicatedDiagnostics};
pub use error::{CoreError, CoreResult};
pub use model::{
    ChatMessage, FunctionCall, MockModelProvider, ModelProvider, ModelRequest, ModelResponse,
    ModelUsage, OpenAiCompatibleProvider, ToolCall,
};
pub use token_optimizer::{ContextBudgetConfig, TokenOptimizer};
pub use tool_router::{TaskStrategy, ToolStrategyRouter, ToolTier};
pub use verifier::{ActionVerifier, VerificationOutcome, VerifyCondition};

#[cfg(test)]
mod tests {
    use super::*;
    use agent_platform::{WindowsFileSystemProvider, WindowsShellExecutor};
    use agent_security::{PermissionMode, SecurityEngine};
    use agent_storage::{Database, Task, TaskStatus};
    use agent_tools::create_default_registry;
    use chrono::Utc;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use tempfile::tempdir;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_agent_loop_with_mock_and_tools() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().to_path_buf();
        let fs = Box::new(WindowsFileSystemProvider::new());
        let sec = Arc::new(SecurityEngine::new(root.clone(), PermissionMode::FullAuto, fs));
        let executor = Box::new(WindowsShellExecutor::new());
        let registry = Arc::new(create_default_registry(executor, sec.clone()));
        let db = Database::open_in_memory().expect("db");
        let optimizer = Arc::new(TokenOptimizer::new(ContextBudgetConfig::default()));

        // Program mock model to:
        // Turn 1: call terminal_execute with echo HelloAgent
        // Turn 2: return final answer
        let turn1 = ModelResponse {
            content: Some("I will run an echo command.".to_string()),
            tool_calls: vec![ToolCall {
                id: "call_abc".to_string(),
                r#type: "function".to_string(),
                function: FunctionCall {
                    name: "terminal_execute".to_string(),
                    arguments: r#"{"command":"echo HelloAgent","shell":"cmd"}"#.to_string(),
                },
            }],
            usage: ModelUsage {
                prompt_tokens: 20,
                completion_tokens: 15,
                total_tokens: 35,
                cached_tokens: None,
            },
        };

        let turn2 = ModelResponse {
            content: Some("Echo succeeded and task is complete.".to_string()),
            tool_calls: Vec::new(),
            usage: ModelUsage {
                prompt_tokens: 40,
                completion_tokens: 10,
                total_tokens: 50,
                cached_tokens: None,
            },
        };

        let provider = Arc::new(MockModelProvider::new(vec![turn1, turn2]));
        let (agent, _rx) = AgentLoop::new(provider, registry, sec, db.clone(), optimizer, 5);

        let mut task = Task {
            id: Uuid::new_v4(),
            conversation_id: None,
            title: "Test Echo Agent".to_string(),
            status: TaskStatus::Pending,
            goal: "Run echo".to_string(),
            current_step: "Pending".to_string(),
            result: None,
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db.create_task(&task).await.expect("create task");

        let cancel = Arc::new(AtomicBool::new(false));
        let res = agent
            .run_task(&mut task, "System instructions", "Please echo HelloAgent", cancel)
            .await
            .expect("agent run");

        assert!(res.contains("complete"));
        assert_eq!(task.status, TaskStatus::Completed);

        // Verify task and checkpoint in database
        let tasks = db.list_tasks().await.expect("list tasks");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::Completed);

        let ck = db.get_latest_checkpoint(task.id).await.expect("ck").expect("has ck");
        assert!(ck.summary.contains("Completed"));
    }

    #[test]
    fn test_tool_strategy_router_selection() {
        let router = ToolStrategyRouter::new();

        // Web task -> Tier 4
        let web_strategy = router.route("Navigate to https://example.com and check header");
        assert_eq!(web_strategy.tier, ToolTier::Tier4BrowserPlaywright);
        assert!(web_strategy.recommended_tools.iter().any(|t| t.contains("playwright")));

        // Desktop task -> Tier 5
        let desktop_strategy = router.route("Open notepad and type test message");
        assert_eq!(desktop_strategy.tier, ToolTier::Tier5WindowsUiAutomation);
        assert!(desktop_strategy.recommended_tools.iter().any(|t| t.contains("windows_mcp")));

        // File task -> Tier 2
        let file_strategy = router.route("Read file src/main.rs and list directory");
        assert_eq!(file_strategy.tier, ToolTier::Tier2FilesystemDb);

        // Build task -> Tier 3
        let build_strategy = router.route("Run cargo test and compile build");
        assert_eq!(build_strategy.tier, ToolTier::Tier3ShellCliCoding);

        // Screenshot task -> Tier 6
        let vision_strategy = router.route("Take a screenshot of current screen");
        assert_eq!(vision_strategy.tier, ToolTier::Tier6VisionScreenshot);
    }

    #[test]
    fn test_action_verifier_conditions() {
        let verifier = ActionVerifier::new();

        // 1. URL condition
        let url_cond = VerifyCondition::UrlMatches {
            expected_pattern: "dashboard".to_string(),
        };
        let outcome1 = verifier.verify(&url_cond, &serde_json::json!({ "url": "https://myapp.com/dashboard/home" }));
        assert!(outcome1.is_passed());

        let outcome2 = verifier.verify(&url_cond, &serde_json::json!({ "url": "https://myapp.com/login" }));
        assert!(!outcome2.is_passed());

        // 2. Element condition
        let el_cond = VerifyCondition::ElementVisible {
            selector_or_text: "Welcome User".to_string(),
        };
        let outcome3 = verifier.verify(&el_cond, &serde_json::json!({ "snapshot": "<div><h1>Welcome User</h1></div>" }));
        assert!(outcome3.is_passed());

        // 3. Exit code condition
        let outcome4 = verifier.verify(&VerifyCondition::ExitCodeZero, &serde_json::json!({ "exit_code": 0 }));
        assert!(outcome4.is_passed());
    }

    #[test]
    fn test_dom_token_optimization_and_delta() {
        use agent_platform::DomSnapshotNode;
        use std::collections::HashMap;

        let node1 = DomSnapshotNode {
            ref_id: "e1".to_string(),
            tag: "button".to_string(),
            role: Some("button".to_string()),
            text: Some("Submit".to_string()),
            value: None,
            is_interactive: true,
            attributes: HashMap::new(),
            children: Vec::new(),
        };

        let node2 = DomSnapshotNode {
            ref_id: "e1".to_string(),
            tag: "button".to_string(),
            role: Some("button".to_string()),
            text: Some("Submitted!".to_string()),
            value: None,
            is_interactive: true,
            attributes: HashMap::new(),
            children: Vec::new(),
        };

        let compact = TokenOptimizer::dom_to_compact_text(&node1, 0);
        assert!(compact.contains("[e1] button \"Submit\" [interactive]"));

        let delta = TokenOptimizer::compute_dom_delta(&node1, &node2);
        assert!(delta.contains("changed/added nodes"));
        assert!(delta.contains("Submitted!"));
    }
}
