use agent_core::{
    ActionVerifier, AgentEvent, AgentLoop, AgentRegistry, CodingMode, CodingWorkflowEngine,
    ContextBudgetConfig, ModelProvider, OpenAiCompatibleProvider, ResourceLockManager,
    SharedArtifactStore, TokenOptimizer, ToolStrategyRouter,
};
use agent_platform::{
    get_platform_adapters, LanguageToolingAdapter, MultiLanguageAdapter, ProjectInfo,
};
use agent_security::{PermissionMode, SecurityEngine, SharedSecurityEngine};
use agent_storage::{Database, Task, TaskStatus};
use agent_tools::{create_default_registry, BrowserMcpManager, SharedToolRegistry, WindowsMcpManager};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info};
use uuid::Uuid;

pub mod business_coordinator;
pub mod recovery;
pub mod scheduler;
pub mod service;
pub mod supervisor;
pub mod watchdog;
pub mod worker_dispatcher;
pub mod workflow_scheduler;

pub use business_coordinator::*;
pub use recovery::*;
pub use scheduler::*;
pub use service::*;
pub use supervisor::*;
pub use watchdog::*;
pub use worker_dispatcher::*;
pub use workflow_scheduler::*;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Core error: {0}")]
    Core(#[from] agent_core::CoreError),

    #[error("DAG error: {0}")]
    Dag(#[from] agent_core::DagError),

    #[error("Storage error: {0}")]
    Storage(#[from] agent_storage::StorageError),

    #[error("Tool error: {0}")]
    Tool(#[from] agent_tools::ToolError),

    #[error("Security error: {0}")]
    Security(#[from] agent_security::SecurityError),

    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),

    #[error("AI Provider not configured")]
    ProviderNotConfigured,
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub provider_url: String,
    pub api_key: String,
    pub model_name: String,
    pub workspace_path: PathBuf,
    pub permission_mode: PermissionMode,
    pub max_loops: u32,
    pub budget_config: ContextBudgetConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        let temp_workspace = std::env::temp_dir().join("aai-workspace");
        let _ = std::fs::create_dir_all(&temp_workspace);
        Self {
            provider_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model_name: "gpt-4o".to_string(),
            workspace_path: temp_workspace,
            permission_mode: PermissionMode::WorkspaceAuto,
            max_loops: 25,
            budget_config: ContextBudgetConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub runtime_active: bool,
    pub provider_connected: bool,
    pub provider_name: String,
    pub model_name: String,
    pub windows_mcp_connected: bool,
    pub windows_mcp_prerequisites: bool,
    pub windows_mcp_hint: String,
    pub browser_connected: bool,
    pub desktop_controller: String,
    pub desktop_remaining_secs: i64,
    pub is_emergency_stopped: bool,
    pub tasks_running: u32,
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub total_conversations: u32,
    pub pending_approvals: u32,
    pub workspace_path: String,
    pub permission_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveredTaskInfo {
    pub task_id: Uuid,
    pub title: String,
    pub previous_status: TaskStatus,
    pub new_status: TaskStatus,
    pub checkpoint_step: Option<u32>,
    pub checkpoint_summary: Option<String>,
}

struct TaskControl {
    cancel_flag: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct RuntimeHost {
    config: Arc<RwLock<RuntimeConfig>>,
    database: Database,
    security: SharedSecurityEngine,
    registry: SharedToolRegistry,
    mcp_manager: Arc<WindowsMcpManager>,
    browser_mcp: Arc<BrowserMcpManager>,
    tool_router: Arc<ToolStrategyRouter>,
    verifier: Arc<ActionVerifier>,
    provider: Arc<RwLock<Option<Arc<dyn ModelProvider>>>>,
    optimizer: Arc<TokenOptimizer>,
    agent_registry: Arc<RwLock<AgentRegistry>>,
    artifact_store: Arc<SharedArtifactStore>,
    lock_manager: Arc<ResourceLockManager>,
    supervisor: Arc<SupervisorEngine>,
    workflow_scheduler: Arc<WorkflowScheduler>,
    service: Arc<AgentRuntimeService>,
    active_tasks: Arc<RwLock<HashMap<Uuid, TaskControl>>>,
    event_bus: broadcast::Sender<(Uuid, AgentEvent)>,
}

impl RuntimeHost {
    pub async fn new<P: AsRef<Path>>(db_path: P, config: RuntimeConfig) -> RuntimeResult<Self> {
        let database = Database::open(db_path)?;
        let (_os_adapter, shell_executor, fs_provider) = get_platform_adapters();

        let security = Arc::new(SecurityEngine::new(
            config.workspace_path.clone(),
            config.permission_mode,
            fs_provider,
        ));

        let registry = Arc::new(create_default_registry(shell_executor, security.clone()));
        let mcp_manager = Arc::new(WindowsMcpManager::new(security.clone()));
        let browser_mcp = Arc::new(BrowserMcpManager::new(security.clone()));
        let tool_router = Arc::new(ToolStrategyRouter::new());
        let verifier = Arc::new(ActionVerifier::new());
        let optimizer = Arc::new(TokenOptimizer::new(config.budget_config.clone()));

        let mut provider_opt: Option<Arc<dyn ModelProvider>> = None;
        if !config.provider_url.is_empty() {
            provider_opt = Some(Arc::new(OpenAiCompatibleProvider::new(
                config.provider_url.clone(),
                config.api_key.clone(),
                config.model_name.clone(),
            )));
        }

        let agent_registry = Arc::new(RwLock::new(AgentRegistry::new()));
        let artifact_store = Arc::new(SharedArtifactStore::new());
        let lock_manager = Arc::new(ResourceLockManager::new());
        let supervisor = Arc::new(SupervisorEngine::new(provider_opt.clone()));

        let dispatcher = Arc::new(WorkerDispatcher::new(
            config.workspace_path.clone(),
            security.clone(),
            registry.clone(),
            artifact_store.clone(),
            lock_manager.clone(),
            mcp_manager.clone(),
            browser_mcp.clone(),
            verifier.clone(),
        ));

        let workflow_scheduler = Arc::new(WorkflowScheduler::new(
            database.clone(),
            dispatcher,
            supervisor.clone(),
            agent_registry.clone(),
            artifact_store.clone(),
            lock_manager.clone(),
        ));

        let (event_bus, _) = broadcast::channel(500);

        let service = Arc::new(AgentRuntimeService::new(Arc::new(database.clone())));

        let host = Self {
            config: Arc::new(RwLock::new(config)),
            database,
            security,
            registry,
            mcp_manager,
            browser_mcp,
            tool_router,
            verifier,
            provider: Arc::new(RwLock::new(provider_opt)),
            optimizer,
            agent_registry,
            artifact_store,
            lock_manager,
            supervisor,
            workflow_scheduler,
            service,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
        };

        Ok(host)
    }

    pub fn service(&self) -> &Arc<AgentRuntimeService> {
        &self.service
    }

    pub fn workflow_scheduler(&self) -> &Arc<WorkflowScheduler> {
        &self.workflow_scheduler
    }

    pub fn agent_registry(&self) -> &Arc<RwLock<AgentRegistry>> {
        &self.agent_registry
    }

    pub fn artifact_store(&self) -> &Arc<SharedArtifactStore> {
        &self.artifact_store
    }

    pub fn lock_manager(&self) -> &Arc<ResourceLockManager> {
        &self.lock_manager
    }

    pub fn supervisor(&self) -> &Arc<SupervisorEngine> {
        &self.supervisor
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<(Uuid, AgentEvent)> {
        self.event_bus.subscribe()
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn security(&self) -> &SharedSecurityEngine {
        &self.security
    }

    pub fn registry(&self) -> &SharedToolRegistry {
        &self.registry
    }

    pub fn mcp_manager(&self) -> &Arc<WindowsMcpManager> {
        &self.mcp_manager
    }

    pub fn browser_mcp(&self) -> &Arc<BrowserMcpManager> {
        &self.browser_mcp
    }

    pub fn tool_router(&self) -> &Arc<ToolStrategyRouter> {
        &self.tool_router
    }

    pub fn verifier(&self) -> &Arc<ActionVerifier> {
        &self.verifier
    }

    pub async fn update_config(&self, new_cfg: RuntimeConfig) {
        self.security.set_workspace_root(new_cfg.workspace_path.clone()).await;
        self.security.set_mode(new_cfg.permission_mode).await;

        let mut prov = self.provider.write().await;
        *prov = Some(Arc::new(OpenAiCompatibleProvider::new(
            new_cfg.provider_url.clone(),
            new_cfg.api_key.clone(),
            new_cfg.model_name.clone(),
        )));

        let mut cfg = self.config.write().await;
        *cfg = new_cfg;
    }

    pub async fn set_custom_provider(&self, provider: Arc<dyn ModelProvider>) {
        let mut prov = self.provider.write().await;
        *prov = Some(provider);
    }

    pub async fn test_provider_connection(&self) -> RuntimeResult<bool> {
        let prov = {
            let lock = self.provider.read().await;
            lock.clone()
        };
        if let Some(p) = prov {
            p.test_connection().await.map_err(RuntimeError::Core)
        } else {
            Err(RuntimeError::ProviderNotConfigured)
        }
    }

    pub async fn get_config(&self) -> RuntimeConfig {
        self.config.read().await.clone()
    }

    /// Checkpoint recovery on startup
    pub async fn recover_tasks_from_checkpoints(&self) -> RuntimeResult<Vec<RecoveredTaskInfo>> {
        let all_tasks = self.database.list_tasks().await?;
        let mut recovered = Vec::new();

        for task in all_tasks {
            // Find tasks that were in progress when app stopped
            if task.status == TaskStatus::Running || task.status == TaskStatus::WaitingApproval {
                let latest_ck = self.database.get_latest_checkpoint(task.id).await?;
                let (step, summary) = match &latest_ck {
                    Some(ck) => (Some(ck.step_index), Some(ck.summary.clone())),
                    None => (None, None),
                };

                // Transition to Paused for safe human verification (never re-run side-effects blindly)
                let new_status = TaskStatus::Paused;
                let pause_step = format!(
                    "Recovered from restart at step {}. Paused for verification.",
                    step.unwrap_or(0)
                );

                self.database
                    .update_task_status(task.id, new_status, &pause_step, None, None)
                    .await?;

                recovered.push(RecoveredTaskInfo {
                    task_id: task.id,
                    title: task.title.clone(),
                    previous_status: task.status,
                    new_status,
                    checkpoint_step: step,
                    checkpoint_summary: summary,
                });
            }
        }

        info!("Checkpoint recovery completed: {} tasks recovered", recovered.len());
        Ok(recovered)
    }

    pub async fn get_dashboard_stats(&self) -> RuntimeResult<DashboardStats> {
        let tasks = self.database.list_tasks().await?;
        let convs = self.database.list_conversations().await?;
        let pending_approvals = self.security.list_pending_approvals().await;
        let cfg = self.config.read().await;

        let running_count = tasks.iter().filter(|t| t.status == TaskStatus::Running).count() as u32;
        let completed_count = tasks.iter().filter(|t| t.status == TaskStatus::Completed).count() as u32;
        let failed_count = tasks.iter().filter(|t| t.status == TaskStatus::Failed).count() as u32;

        let (prereq_ok, prereq_hint) = self.mcp_manager.check_prerequisites().await;
        let mcp_connected = self.mcp_manager.is_connected().await;

        let provider_connected = {
            let prov_lock = self.provider.read().await;
            prov_lock.is_some()
        };

        let browser_connected = self.browser_mcp.is_connected().await;
        let ownership_report = self.security.ownership().get_ownership_status().await;

        Ok(DashboardStats {
            runtime_active: true,
            provider_connected,
            provider_name: cfg.provider_url.clone(),
            model_name: cfg.model_name.clone(),
            windows_mcp_connected: mcp_connected,
            windows_mcp_prerequisites: prereq_ok,
            windows_mcp_hint: prereq_hint,
            browser_connected,
            desktop_controller: format!("{:?}", ownership_report.current_controller),
            desktop_remaining_secs: ownership_report.remaining_secs,
            is_emergency_stopped: ownership_report.is_emergency_stopped,
            tasks_running: running_count,
            tasks_completed: completed_count,
            tasks_failed: failed_count,
            total_conversations: convs.len() as u32,
            pending_approvals: pending_approvals.len() as u32,
            workspace_path: cfg.workspace_path.display().to_string(),
            permission_mode: format!("{:?}", cfg.permission_mode),
        })
    }

    pub async fn start_task(
        &self,
        title: &str,
        goal: &str,
        user_prompt: &str,
        conversation_id: Option<Uuid>,
    ) -> RuntimeResult<Uuid> {
        let task_id = Uuid::new_v4();
        let mut task = Task {
            id: task_id,
            conversation_id,
            title: title.to_string(),
            status: TaskStatus::Pending,
            goal: goal.to_string(),
            current_step: "Pending start".to_string(),
            result: None,
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.database.create_task(&task).await?;

        let cancel_flag = Arc::new(AtomicBool::new(false));
        {
            let mut active = self.active_tasks.write().await;
            active.insert(
                task_id,
                TaskControl {
                    cancel_flag: cancel_flag.clone(),
                },
            );
        }

        let provider = {
            let prov = self.provider.read().await;
            prov.clone().ok_or(RuntimeError::ProviderNotConfigured)?
        };

        let cfg = self.config.read().await.clone();
        let (agent_loop, mut rx) = AgentLoop::new(
            provider,
            self.registry.clone(),
            self.security.clone(),
            self.database.clone(),
            self.optimizer.clone(),
            cfg.max_loops,
        );

        let event_bus = self.event_bus.clone();
        let active_tasks = self.active_tasks.clone();
        let user_prompt_owned = user_prompt.to_string();

        // Forward agent events to runtime event bus
        tokio::spawn(async move {
            while let Ok(evt) = rx.recv().await {
                let _ = event_bus.send((task_id, evt));
            }
        });

        // Spawn agent loop task
        tokio::spawn(async move {
            let mut target_dir = cfg.workspace_path.clone();

            // Scan user prompt for an explicit Windows path (e.g. "E:\DATA\source\Nas\LocalDrive" or "D:\...")
            for word in user_prompt_owned.split_whitespace() {
                let clean = word.trim_matches(|c| c == '\'' || c == '"' || c == '`' || c == '<' || c == '>' || c == '(' || c == ')' || c == ',');
                let candidate = PathBuf::from(clean);
                if candidate.is_absolute() && candidate.exists() {
                    if candidate.is_dir() {
                        target_dir = candidate;
                        break;
                    } else if let Some(parent) = candidate.parent() {
                        if parent.exists() && parent.is_dir() {
                            target_dir = parent.to_path_buf();
                            break;
                        }
                    }
                }
            }

            let adapter = MultiLanguageAdapter::new();
            let info = adapter.detect_project(&target_dir);

            let lang = info
                .as_ref()
                .and_then(|i| i.primary_language.clone())
                .unwrap_or_else(|| "General Workspace".to_string());
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

            // OpenCode Project Instruction Discovery: automatically honor AGENTS.md, CLAUDE.md, etc.
            let mut project_instructions = String::new();
            let candidate_files = [
                "AGENTS.md",
                "AGIAGENTS.md",
                "CLAUDE.md",
                ".opencode/instructions",
                ".ai/instructions.md",
            ];
            for filename in candidate_files {
                let p = target_dir.join(filename);
                if p.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&p) {
                        let preview: String = content.chars().take(3000).collect();
                        project_instructions.push_str(&format!(
                            "\n<project_instructions file=\"{}\">\n{}\n</project_instructions>\n",
                            filename, preview.trim()
                        ));
                        break;
                    }
                }
            }

            let system_prompt = format!(
                r#"You are SIA — an elite Autonomous AI Workspace & Software Engineer on Windows, powered by OpenCode algorithms and Google Antigravity design.
Your objective is to fulfill the user's instructions completely, accurately, and autonomously.

<env>
  Active Project / Workspace: {}
  Detected Language / Tech: {}
  Build System: {}
  Test Runner: {}
  Git Branch: {}
</env>
{}
# CRITICAL ACTION DIRECTIVE (NEVER JUST GIVE MANUAL INSTRUCTIONS):
- You are an AUTONOMOUS AGENT with real Windows operating system control, NOT a passive chatbot.
- When the user asks you to perform an action (e.g. "mo app zalo", "mở zalo", "mở chrome", "mở notepad", "chạy lệnh...", "tạo file...", "sửa code..."):
  NEVER merely explain how the user can do it manually!
  YOU MUST DIRECTLY INVOKE THE APPROPRIATE TOOL TO EXECUTE IT FOR THE USER!
- Specifically, to launch or open any application on Windows: IMMEDIATELY invoke `desktop_app` with `{{"name": "<app_name>"}}`.
- If your model proxy does not support function calling schemas, output your tool call formatted as:
  <tool_call>
  {{"name": "desktop_app", "arguments": {{"name": "zalo"}}}}
  </tool_call>

# System-Wide Local Access & Guidelines:
- You have native tool permissions to inspect, read, search, and analyze files and directories across ALL local Windows drives (C:\, D:\, E:\, etc.).
- When the user asks about or references any local path (e.g. `E:\DATA\source\Nas\LocalDrive`), you have full access to inspect it using tools like `file_list`, `project_detect`, `file_read`, and `code_search` with that path.
- NEVER refuse by claiming a path is outside your workspace. You must proactively explore the files (such as README.md, package.json, go.mod, Cargo.toml, project files) and answer the user thoroughly.

# OpenCode Tooling & Automation Workflow:
1. PLANNING & WORKING TODOS:
   - For any multi-step task (3 or more steps), proactively call `todowrite` with full list of items: `[{{ "content": "...", "status": "pending" | "in_progress" | "completed" }}]`.
   - Update `todowrite` whenever progress changes so the user sees real-time milestones.

2. EXPLORATION & DISCOVERY:
   - Use `read` (or `file_read`) to inspect files with line ranges. `read` also lists directory entries if given a folder path.
   - Use `glob` (or `file_list`) to discover files matching patterns.
   - Use `grep` (or `code_search`) to search for text patterns or function definitions.
   - For broad codebase structure, prefer `repo_map` to conserve tokens.

3. IMPLEMENTATION & MUTATION:
   - Use `edit` (or `code_edit`) for surgical, exact-match code replacements (`oldString`, `newString`).
   - Use `write` (or `file_write`) to create new files or rewrite full components.
   - Use `patch` (or `code_patch`) for multi-file atomic changes with automatic rollback on error.
   - Keep designs ultra-clean, minimal, and high-performance.

4. EXECUTION & VERIFICATION:
   - Use `shell` (or `terminal_execute`) for PowerShell / Cmd tasks, dependency installs, or running builds/tests.
   - Verify changes where applicable.

# CursorTouch / Windows-MCP Native Desktop Automation:
- You have native, built-in Windows desktop automation tools (no external Python setup needed):
  - `Click` (or `desktop_click`): Mouse click/hover at `loc: [x, y]`, `button: 'left' | 'right' | 'middle'`, `clicks: 0 | 1 | 2`.
  - `Type` (or `desktop_type`): Types text at `loc: [x, y]` or focused element. Set `clear: true` to clear field first, `press_enter: true` to submit.
  - `Scroll` (or `desktop_scroll`): Scrolls mouse at `loc: [x, y]`, `direction: 'up' | 'down' | 'left' | 'right'`, `wheel_times: N`.
  - `Move` (or `desktop_move`): Moves cursor to `loc: [x, y]`. Set `drag: true` and optional `from_loc: [x, y]` to drag-and-drop.
  - `Shortcut` (or `desktop_shortcut`): Presses keyboard combinations (e.g. `'ctrl+c'`, `'ctrl+v'`, `'alt+tab'`, `'win+r'`, `'enter'`, `'escape'`).
  - `App` (or `desktop_app`, `Launch`): Open or switch applications (`mode: 'launch' | 'switch' | 'launch_executable'`, `name: '...'`).
  - `Clipboard` (or `desktop_clipboard`): Read or write Windows clipboard (`mode: 'get' | 'set'`).
  - `Snapshot` (or `desktop_snapshot`, `State`): Inspect active window, open window list, resolution, lock status, and cursor coordinates.
  - `Screenshot` (or `desktop_screenshot`): Capture screen to file and return metadata.
  - `SwitchWindow` (or `desktop_switch_window`): Bring any open window into foreground focus by matching title.

5. COMMUNICATION & REASONING:
   - Express deep technical reasoning, architecture choices, visual language, or component breakdowns in your thoughts.
   - Respond in the language used by the user (Vietnamese if requested in Vietnamese, English if in English).
   - Be direct, professional, and concise. Avoid fluff or generic disclaimers."#,
                target_dir.display(),
                lang,
                build_cmd,
                test_cmd,
                branch,
                project_instructions
            );

            let res = agent_loop
                .run_task(&mut task, &system_prompt, &user_prompt_owned, cancel_flag)
                .await;

            if let Err(e) = res {
                error!("Task {} ended with error: {}", task_id, e);
            }

            let mut active = active_tasks.write().await;
            active.remove(&task_id);
        });

        Ok(task_id)
    }

    pub async fn start_coding_task(
        &self,
        title: &str,
        goal: &str,
        user_prompt: &str,
        mode: CodingMode,
        conversation_id: Option<Uuid>,
    ) -> RuntimeResult<Uuid> {
        let task_id = Uuid::new_v4();
        let mut task = Task {
            id: task_id,
            conversation_id,
            title: format!("[{mode}] {title}"),
            status: TaskStatus::Pending,
            goal: goal.to_string(),
            current_step: format!("Initializing coding workflow in {mode} mode"),
            result: None,
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.database.create_task(&task).await?;

        let cancel_flag = Arc::new(AtomicBool::new(false));
        {
            let mut active = self.active_tasks.write().await;
            active.insert(
                task_id,
                TaskControl {
                    cancel_flag: cancel_flag.clone(),
                },
            );
        }

        let provider = {
            let prov = self.provider.read().await;
            prov.clone().ok_or(RuntimeError::ProviderNotConfigured)?
        };

        let cfg = self.config.read().await.clone();
        let (engine, mut rx) = CodingWorkflowEngine::new(
            provider,
            self.registry.clone(),
            self.security.clone(),
            self.database.clone(),
            self.optimizer.clone(),
            cfg.max_loops,
        );

        let event_bus = self.event_bus.clone();
        let active_tasks = self.active_tasks.clone();
        let user_prompt_owned = user_prompt.to_string();

        tokio::spawn(async move {
            while let Ok(evt) = rx.recv().await {
                let _ = event_bus.send((task_id, evt));
            }
        });

        tokio::spawn(async move {
            let res = engine
                .run(&mut task, mode, &user_prompt_owned, cancel_flag)
                .await;

            if let Err(e) = res {
                error!("Coding task {} ended with error: {}", task_id, e);
            }

            let mut active = active_tasks.write().await;
            active.remove(&task_id);
        });

        Ok(task_id)
    }

    pub async fn detect_project(&self, subpath: Option<&str>) -> Option<ProjectInfo> {
        let workspace = self.security.get_workspace_root().await;
        let target_dir = match subpath {
            Some(s) if !s.is_empty() => workspace.join(s),
            _ => workspace,
        };
        let adapter = MultiLanguageAdapter::new();
        adapter.detect_project(&target_dir)
    }

    pub async fn get_repo_map(&self, max_chars: Option<u64>, subdir: Option<&str>) -> RuntimeResult<String> {
        let res = self
            .registry
            .execute(
                "repo_map",
                "direct_call",
                serde_json::json!({
                    "max_chars": max_chars.unwrap_or(4000),
                    "subdir": subdir.unwrap_or("")
                }),
            )
            .await?;
        Ok(res.stdout)
    }

    pub async fn preview_diff(&self, path: &str, new_content: &str) -> RuntimeResult<String> {
        let res = self
            .registry
            .execute(
                "diff_preview",
                "direct_call",
                serde_json::json!({
                    "path": path,
                    "new_content": new_content
                }),
            )
            .await?;
        Ok(res.stdout)
    }

    pub async fn run_build(&self, command: Option<&str>, path: Option<&str>) -> RuntimeResult<serde_json::Value> {
        let mut params = serde_json::Map::new();
        if let Some(cmd) = command {
            params.insert("command".to_string(), serde_json::Value::String(cmd.to_string()));
        }
        if let Some(p) = path {
            params.insert("path".to_string(), serde_json::Value::String(p.to_string()));
        }

        let res = self
            .registry
            .execute("run_build", "direct_call", serde_json::Value::Object(params))
            .await?;

        let val: serde_json::Value = serde_json::from_str(&res.stdout).unwrap_or(serde_json::json!({
            "raw_output": res.stdout,
            "stderr": res.stderr
        }));
        Ok(val)
    }

    pub async fn run_test(&self, command: Option<&str>, filter: Option<&str>, path: Option<&str>) -> RuntimeResult<serde_json::Value> {
        let mut params = serde_json::Map::new();
        if let Some(cmd) = command {
            params.insert("command".to_string(), serde_json::Value::String(cmd.to_string()));
        }
        if let Some(f) = filter {
            params.insert("filter".to_string(), serde_json::Value::String(f.to_string()));
        }
        if let Some(p) = path {
            params.insert("path".to_string(), serde_json::Value::String(p.to_string()));
        }

        let res = self
            .registry
            .execute("run_test", "direct_call", serde_json::Value::Object(params))
            .await?;

        let val: serde_json::Value = serde_json::from_str(&res.stdout).unwrap_or(serde_json::json!({
            "raw_output": res.stdout,
            "stderr": res.stderr
        }));
        Ok(val)
    }

    pub async fn cancel_task(&self, task_id: Uuid) -> RuntimeResult<()> {
        let active = self.active_tasks.read().await;
        if let Some(ctrl) = active.get(&task_id) {
            ctrl.cancel_flag.store(true, Ordering::SeqCst);
            self.database
                .update_task_status(
                    task_id,
                    TaskStatus::Cancelled,
                    "Cancelled by user request",
                    None,
                    None,
                )
                .await?;
            let _ = self.event_bus.send((
                task_id,
                AgentEvent::TaskStatusChanged {
                    status: TaskStatus::Cancelled,
                },
            ));
            Ok(())
        } else {
            Err(RuntimeError::TaskNotFound(task_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::MockModelProvider;
    use agent_storage::Checkpoint;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_runtime_host_lifecycle() {
        let temp = tempdir().expect("tempdir");
        let db_path = temp.path().join("runtime_test.db");
        let host = RuntimeHost::new(&db_path, RuntimeConfig::default())
            .await
            .expect("init runtime");

        let stats = host.get_dashboard_stats().await.expect("stats");
        assert!(stats.runtime_active);
        assert_eq!(stats.tasks_running, 0);

        // Test custom mock provider
        host.set_custom_provider(Arc::new(MockModelProvider::new(vec![])))
            .await;

        let task_id = host
            .start_task("Test task", "Goal", "Echo something", None)
            .await
            .expect("start task");

        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        let tasks = host.database().list_tasks().await.expect("list");
        assert!(tasks.iter().any(|t| t.id == task_id));
    }

    #[tokio::test]
    async fn test_recovery_from_checkpoints() {
        let temp = tempdir().expect("tempdir");
        let db_path = temp.path().join("recovery_test.db");
        let host = RuntimeHost::new(&db_path, RuntimeConfig::default())
            .await
            .expect("init runtime");

        // Create an unfinished task as if the app crashed
        let task_id = Uuid::new_v4();
        let task = Task {
            id: task_id,
            conversation_id: None,
            title: "Crashed Build Task".to_string(),
            status: TaskStatus::Running,
            goal: "Compile source code".to_string(),
            current_step: "Compiling crate 3".to_string(),
            result: None,
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        host.database().create_task(&task).await.expect("create");

        let ck = Checkpoint {
            id: Uuid::new_v4(),
            task_id,
            step_index: 3,
            state_json: "{}".to_string(),
            summary: "Finished crate 2".to_string(),
            created_at: Utc::now(),
        };
        host.database().save_checkpoint(&ck).await.expect("ck");

        // Run recovery
        let recovered = host.recover_tasks_from_checkpoints().await.expect("recovery");
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].new_status, TaskStatus::Paused);
        assert_eq!(recovered[0].checkpoint_step, Some(3));
    }
}
