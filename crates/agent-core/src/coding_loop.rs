use crate::agent_loop::{AgentEvent, AgentLoop};
use crate::error::CoreResult;
use crate::model::ModelProvider;
use crate::token_optimizer::TokenOptimizer;
use agent_platform::{LanguageToolingAdapter, MultiLanguageAdapter};
use agent_security::SharedSecurityEngine;
use agent_storage::{Database, Task};
use agent_tools::SharedToolRegistry;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CodingMode {
    Ask,
    Plan,
    Build,
}

impl std::fmt::Display for CodingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodingMode::Ask => write!(f, "ask"),
            CodingMode::Plan => write!(f, "plan"),
            CodingMode::Build => write!(f, "build"),
        }
    }
}

pub struct CodingWorkflowEngine {
    agent_loop: AgentLoop,
    registry: SharedToolRegistry,
    security: SharedSecurityEngine,
    event_sender: broadcast::Sender<AgentEvent>,
}

impl CodingWorkflowEngine {
    pub fn new(
        provider: Arc<dyn ModelProvider>,
        registry: SharedToolRegistry,
        security: SharedSecurityEngine,
        database: Database,
        optimizer: Arc<TokenOptimizer>,
        max_loops: u32,
    ) -> (Self, broadcast::Receiver<AgentEvent>) {
        let (tx, rx) = broadcast::channel(100);
        let (agent_loop, _) = AgentLoop::new(
            provider,
            registry.clone(),
            security.clone(),
            database,
            optimizer,
            max_loops,
        );

        (
            Self {
                agent_loop,
                registry,
                security,
                event_sender: tx,
            },
            rx,
        )
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.event_sender.subscribe()
    }

    /// Build dynamic system prompt with OpenCode-inspired <env> and tool guidelines
    pub async fn build_system_prompt(&self, mode: CodingMode) -> String {
        let workspace = self.security.get_workspace_root().await;
        let adapter = MultiLanguageAdapter::new();
        let info = adapter.detect_project(&workspace);

        let lang = info
            .as_ref()
            .and_then(|i| i.primary_language.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        let build_cmd = info
            .as_ref()
            .and_then(|i| i.build_systems.first().cloned())
            .unwrap_or_else(|| "None".to_string());
        let test_cmd = info
            .as_ref()
            .and_then(|i| i.test_runners.first().cloned())
            .unwrap_or_else(|| "None".to_string());
        let branch = info
            .as_ref()
            .and_then(|i| i.git_branch.clone())
            .unwrap_or_else(|| "main".to_string());

        let mode_instructions = match mode {
            CodingMode::Ask => {
                "- Mode: ASK (Read-only comprehension and analysis).\n- DO NOT edit files or modify code.\n- Answer questions, explain architecture, find symbols and provide code walkthroughs."
            }
            CodingMode::Plan => {
                "- Mode: PLAN (Implementation planning only).\n- Research the codebase and produce a structured Markdown implementation plan.\n- Specify file paths, changes needed, dependencies, and testing steps.\n- DO NOT edit or modify files during the planning mode."
            }
            CodingMode::Build => {
                "- Mode: BUILD (Autonomous implementation, testing, and self-correction).\n- Formulate a clear plan, modify code using `code_edit` or `code_patch`.\n- Always run `run_build` and `run_test` after making changes to verify correctness.\n- If compilation errors occur, inspect the root cause error, apply targeted fixes, and re-run build until tests pass."
            }
        };

        format!(
            r#"You are an expert autonomous software engineer operating inside the Universal Autonomous AI Workspace.
Your workflow is inspired by OpenCode: systematic, token-efficient, and safety-conscious.

<env>
  Working directory: {}
  Primary Language: {}
  Build System: {}
  Test Runner: {}
  Git Branch: {}
  Active Mode: {}
</env>

# Core Guidelines:
{}
- Prefer `code_edit` for targeted string replacements. Ensure `old_string` matches unique context.
- Use `repo_map` to get an architectural overview within token budget before reading individual files.
- Use `code_search` for fast ripgrep-like search across files.
- Keep responses concise, objective, and professional."#,
            workspace.display(),
            lang,
            build_cmd,
            test_cmd,
            branch,
            mode,
            mode_instructions
        )
    }

    /// Selects allowed tools based on CodingMode to prevent accidental mutation in Ask/Plan modes
    pub async fn get_tools_for_mode(&self, mode: CodingMode) -> Vec<agent_tools::ToolDefinition> {
        let all_tools = self.registry.list_definitions().await;
        match mode {
            CodingMode::Ask => all_tools
                .into_iter()
                .filter(|t| {
                    matches!(
                        t.name.as_str(),
                        "repo_map"
                            | "code_search"
                            | "file_outline"
                            | "file_read"
                            | "file_list"
                            | "project_detect"
                            | "git_status"
                            | "git_log"
                            | "git_diff"
                    )
                })
                .collect(),
            CodingMode::Plan => all_tools
                .into_iter()
                .filter(|t| {
                    matches!(
                        t.name.as_str(),
                        "repo_map"
                            | "code_search"
                            | "file_outline"
                            | "file_read"
                            | "file_list"
                            | "project_detect"
                            | "git_status"
                            | "git_log"
                            | "git_diff"
                            | "diff_preview"
                    )
                })
                .collect(),
            CodingMode::Build => all_tools,
        }
    }

    /// Runs the specialized coding workflow
    pub async fn run(
        &self,
        task: &mut Task,
        mode: CodingMode,
        user_request: &str,
        cancel_token: Arc<AtomicBool>,
    ) -> CoreResult<String> {
        let system_prompt = self.build_system_prompt(mode).await;
        self.agent_loop
            .run_task(task, &system_prompt, user_request, cancel_token)
            .await
    }
}
