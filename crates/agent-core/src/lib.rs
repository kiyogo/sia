pub mod agent_loop;
pub mod business;
pub mod coding_loop;
pub mod data;
pub mod diagnostics;
pub mod document;
pub mod error;
pub mod model;
pub mod orchestration;
pub mod research;
pub mod token_optimizer;
pub mod tool_router;
pub mod verifier;

pub use agent_loop::{AgentEvent, AgentLoop};
pub use business::*;
pub use coding_loop::{CodingMode, CodingWorkflowEngine};
pub use data::*;
pub use diagnostics::{deduplicate_diagnostics, DeduplicatedDiagnostics};
pub use document::*;
pub use error::{CoreError, CoreResult};
pub use model::{
    ChatMessage, FunctionCall, MockModelProvider, ModelProvider, ModelRequest, ModelResponse,
    ModelUsage, OpenAiCompatibleProvider, ToolCall,
};
pub use orchestration::*;
pub use research::*;
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

    #[tokio::test]
    async fn test_task_dag_validation_and_resolution() {
        let mut dag = TaskDag::new();
        let t1 = Uuid::new_v4();
        let t2 = Uuid::new_v4();
        let t3 = Uuid::new_v4();

        dag.add_node(DagNode {
            task_id: t1,
            title: "Task 1".to_string(),
            description: "Desc".to_string(),
            assigned_agent: AgentType::Coding,
            priority: 10,
            inputs: serde_json::json!({}),
            acceptance_criteria: "".to_string(),
        }).expect("add t1");

        dag.add_node(DagNode {
            task_id: t2,
            title: "Task 2".to_string(),
            description: "Desc".to_string(),
            assigned_agent: AgentType::Browser,
            priority: 5,
            inputs: serde_json::json!({}),
            acceptance_criteria: "".to_string(),
        }).expect("add t2");

        dag.add_node(DagNode {
            task_id: t3,
            title: "Task 3".to_string(),
            description: "Desc".to_string(),
            assigned_agent: AgentType::Supervisor,
            priority: 1,
            inputs: serde_json::json!({}),
            acceptance_criteria: "".to_string(),
        }).expect("add t3");

        // t2 depends on t1, t3 depends on t2
        dag.add_edge(t1, t2, "Success").expect("t1->t2");
        dag.add_edge(t2, t3, "Success").expect("t2->t3");

        // Topological sort
        let order = dag.topological_sort().expect("topo sort");
        assert_eq!(order, vec![t1, t2, t3]);

        // Ready tasks initially: only t1 has no prereqs
        let completed = std::collections::HashSet::new();
        let in_progress = std::collections::HashSet::new();
        let ready = dag.get_ready_tasks(&completed, &in_progress);
        assert_eq!(ready, vec![t1]);

        // Complete t1 -> t2 becomes ready
        let mut completed = std::collections::HashSet::new();
        completed.insert(t1);
        let ready = dag.get_ready_tasks(&completed, &in_progress);
        assert_eq!(ready, vec![t2]);

        // Cycle injection must fail validation
        let cycle_err = dag.add_edge(t3, t1, "Success");
        assert!(cycle_err.is_err());
    }

    #[tokio::test]
    async fn test_resource_lock_manager() {
        let lock_mgr = ResourceLockManager::new();
        let wf_id = Uuid::new_v4();
        let task_id1 = Uuid::new_v4();
        let task_id2 = Uuid::new_v4();

        // Agent 1 acquires desktop
        let lease1 = lock_mgr
            .acquire_lease("desktop:primary", "ComputerAgent", wf_id, task_id1, 30)
            .await
            .expect("lease1");
        assert_eq!(lease1.owner_agent, "ComputerAgent");

        // Agent 2 trying to acquire desktop concurrently is rejected
        let lease2_res = lock_mgr
            .acquire_lease("desktop:primary", "AnotherAgent", wf_id, task_id2, 30)
            .await;
        assert!(lease2_res.is_err());

        // File lock conflict test
        let file_path = std::path::PathBuf::from("src/main.rs");
        let token = lock_mgr.lock_file(&file_path, "CodingAgent1", task_id1).await.expect("lock file");
        assert!(token > 0);

        let conflict = lock_mgr.lock_file(&file_path, "CodingAgent2", task_id2).await;
        assert!(conflict.is_err());

        // Release file lock
        lock_mgr.unlock_file(&file_path).await;
        let lock_again = lock_mgr.lock_file(&file_path, "CodingAgent2", task_id2).await;
        assert!(lock_again.is_ok());
    }

    #[test]
    fn test_research_source_and_evidence_contradiction() {
        let mut registry = ResearchSourceRegistry::new();
        let (src1_id, _) = registry.register_source(
            "https://market-report-a.com".to_string(),
            "Source A Market 2026".to_string(),
            agent_storage::SourceType::Web,
            Some("2026-01-10".to_string()),
            "Market size is projected at $100B with steady growth.",
        );

        let (src2_id, _) = registry.register_source(
            "https://market-report-b.com".to_string(),
            "Source B Market 2026".to_string(),
            agent_storage::SourceType::Web,
            Some("2026-02-15".to_string()),
            "Market size is estimated at $250B due to unprecedented adoption.",
        );

        let mut store = EvidenceStore::new();
        store.add_evidence(
            src1_id,
            None,
            "Market size is 100B in 2026".to_string(),
            agent_storage::EvidenceType::Fact,
            Some("p. 1".to_string()),
            0.9,
        );
        store.add_evidence(
            src2_id,
            None,
            "Market size is 250B in 2026".to_string(),
            agent_storage::EvidenceType::Fact,
            Some("Executive Summary".to_string()),
            0.92,
        );

        let contradictions = store.detect_contradictions();
        assert_eq!(contradictions.len(), 1);
        assert_eq!(contradictions[0].entity_or_metric, "market");
    }

    #[test]
    fn test_document_docx_and_pdf_generation_and_parsing() {
        let temp = tempdir().expect("tempdir");
        let docx_path = temp.path().join("report.docx");
        let pdf_path = temp.path().join("report.pdf");

        let content = DocumentContent {
            title: "Q3 Business Execution Report".to_string(),
            elements: vec![
                DocumentElementType::Heading { level: 1, text: "Executive Summary".to_string() },
                DocumentElementType::Paragraph { text: "All quarterly milestones were achieved on schedule.".to_string() },
                DocumentElementType::Table {
                    headers: vec!["Metric".to_string(), "Target".to_string(), "Actual".to_string()],
                    rows: vec![
                        vec!["Revenue".to_string(), "$10M".to_string(), "$11.2M".to_string()],
                        vec!["NPS".to_string(), "65".to_string(), "72".to_string()],
                    ],
                },
            ],
        };

        // 1. Generate DOCX
        DocumentGenerator::generate_docx(&content, &docx_path).expect("generate docx");
        assert!(docx_path.exists());
        assert!(std::fs::metadata(&docx_path).unwrap().len() > 0);

        // Parse DOCX
        let parsed_docx = DocumentParser::parse_file(&docx_path).expect("parse docx");
        assert_eq!(parsed_docx.format, "DOCX");
        assert!(parsed_docx.raw_text.contains("All quarterly milestones were achieved"));

        // 2. Generate PDF
        DocumentGenerator::generate_pdf(&content, &pdf_path).expect("generate pdf");
        assert!(pdf_path.exists());
        assert!(std::fs::metadata(&pdf_path).unwrap().len() > 0);

        // Parse PDF
        let parsed_pdf = DocumentParser::parse_file(&pdf_path).expect("parse pdf");
        assert_eq!(parsed_pdf.format, "PDF");
    }

    #[test]
    fn test_data_analysis_summary_and_chart_generation() {
        let csv_data = r#"Category,Revenue,Units
Software,12000,50
Services,8500,20
Software,15000,60
Hardware,22000,10
Services,9200,22
"#;
        let table = SpreadsheetReader::parse_csv_content(csv_data, "Sales", ',').expect("parse csv");
        assert_eq!(table.total_rows, 5);

        // Summarize Revenue
        let summary = DataAnalyzer::summarize_column(&table, "Revenue").expect("summarize revenue");
        assert_eq!(summary.count, 5);
        assert_eq!(summary.min, 8500.0);
        assert_eq!(summary.max, 22000.0);
        assert_eq!(summary.sum, 66700.0);

        // Group by Category
        let grouped = DataAnalyzer::group_by(&table, "Category", "Revenue", "sum").expect("group by");
        assert_eq!(grouped.len(), 3);
        assert_eq!(grouped[0].key, "Software");
        assert_eq!(grouped[0].aggregate_value, 27000.0);

        // Chart generation
        let series = vec![
            ChartSeries { label: "Software".to_string(), value: 27000.0 },
            ChartSeries { label: "Hardware".to_string(), value: 22000.0 },
            ChartSeries { label: "Services".to_string(), value: 17700.0 },
        ];
        let svg = ChartGenerator::generate_bar_chart_svg("Revenue by Category", &series, 600, 400);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Revenue by Category"));
        assert!(svg.contains("Software"));
    }

    #[tokio::test]
    async fn test_email_and_calendar_adapters() {
        let email_provider = MockTestEmailProvider::new();
        assert!(email_provider.is_connected().await);

        // Search messages
        let msgs = email_provider.search_messages("market", 10).await.expect("search msgs");
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].subject.contains("Market Research"));

        // Draft message
        let draft_req = EmailDraftRequest {
            to: vec!["recipient@example.com".to_string()],
            cc: vec![],
            bcc: vec![],
            subject: "Re: Market Research".to_string(),
            body: "Here is the completed research analysis.".to_string(),
            operation_id: "op_test_12345".to_string(),
        };
        let draft_id = email_provider.create_draft(&draft_req).await.expect("create draft");

        // Sending without approval MUST be rejected
        let rejected = email_provider.send_message_with_approval(&draft_id, "invalid_token").await;
        assert!(rejected.is_err());

        // With valid approval token, sending succeeds
        email_provider.add_valid_approval_token("token_approved_admin").await;
        let sent_id = email_provider
            .send_message_with_approval(&draft_id, "token_approved_admin")
            .await
            .expect("send with approval");
        assert!(sent_id.starts_with("sent_"));

        // Duplicate send with same operation ID must be blocked (Idempotency)
        let dup = email_provider
            .send_message_with_approval(&draft_id, "token_approved_admin")
            .await;
        assert!(dup.is_err());
        assert!(dup.unwrap_err().contains("Duplicate send prevented"));
    }
}
