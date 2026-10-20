pub mod agent_loop;
pub mod coding_loop;
pub mod diagnostics;
pub mod error;
pub mod model;
pub mod token_optimizer;

pub use agent_loop::{AgentEvent, AgentLoop};
pub use coding_loop::{CodingMode, CodingWorkflowEngine};
pub use diagnostics::{deduplicate_diagnostics, DeduplicatedDiagnostics};
pub use error::{CoreError, CoreResult};
pub use model::{
    ChatMessage, FunctionCall, MockModelProvider, ModelProvider, ModelRequest, ModelResponse,
    ModelUsage, OpenAiCompatibleProvider, ToolCall,
};
pub use token_optimizer::{ContextBudgetConfig, TokenOptimizer};

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
        let (agent, mut rx) = AgentLoop::new(provider, registry, sec, db.clone(), optimizer, 5);

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
}
