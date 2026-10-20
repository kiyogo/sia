use agent_core::{
    AgentEvent, AgentLoop, ContextBudgetConfig, ModelProvider, OpenAiCompatibleProvider,
    TokenOptimizer,
};
use agent_platform::{get_platform_adapters, OperatingSystemAdapter};
use agent_security::{PermissionMode, SecurityEngine, SharedSecurityEngine};
use agent_storage::{Checkpoint, Database, Task, TaskStatus};
use agent_tools::{create_default_registry, SharedToolRegistry, WindowsMcpManager};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Core error: {0}")]
    Core(#[from] agent_core::CoreError),

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
    provider: Arc<RwLock<Option<Arc<dyn ModelProvider>>>>,
    optimizer: Arc<TokenOptimizer>,
    active_tasks: Arc<RwLock<HashMap<Uuid, TaskControl>>>,
    event_bus: broadcast::Sender<(Uuid, AgentEvent)>,
}

impl RuntimeHost {
    pub async fn new<P: AsRef<Path>>(db_path: P, config: RuntimeConfig) -> RuntimeResult<Self> {
        let database = Database::open(db_path)?;
        let (os_adapter, shell_executor, fs_provider) = get_platform_adapters();

        let security = Arc::new(SecurityEngine::new(
            config.workspace_path.clone(),
            config.permission_mode,
            fs_provider,
        ));

        let registry = Arc::new(create_default_registry(shell_executor, security.clone()));
        let mcp_manager = Arc::new(WindowsMcpManager::new(security.clone()));
        let optimizer = Arc::new(TokenOptimizer::new(config.budget_config.clone()));

        let mut provider_opt: Option<Arc<dyn ModelProvider>> = None;
        if !config.provider_url.is_empty() {
            provider_opt = Some(Arc::new(OpenAiCompatibleProvider::new(
                config.provider_url.clone(),
                config.api_key.clone(),
                config.model_name.clone(),
            )));
        }

        let (event_bus, _) = broadcast::channel(500);

        let host = Self {
            config: Arc::new(RwLock::new(config)),
            database,
            security,
            registry,
            mcp_manager,
            provider: Arc::new(RwLock::new(provider_opt)),
            optimizer,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
        };

        Ok(host)
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

    pub async fn get_config(&self) -> RuntimeConfig {
        self.config.read().await.clone()
    }

    /// Checkpoint recovery on startup
    pub async fn recover_tasks_from_checkpoints(&self) -> RuntimeResult<Vec<RecoveredTaskInfo>> {
        let all_tasks = self.database.list_tasks().await?;
        let mut recovered = Vec::new();

        for mut task in all_tasks {
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
            if let Some(prov) = prov_lock.as_ref() {
                prov.test_connection().await.unwrap_or(false)
            } else {
                false
            }
        };

        Ok(DashboardStats {
            runtime_active: true,
            provider_connected,
            provider_name: cfg.provider_url.clone(),
            model_name: cfg.model_name.clone(),
            windows_mcp_connected: mcp_connected,
            windows_mcp_prerequisites: prereq_ok,
            windows_mcp_hint: prereq_hint,
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
            let system_prompt = format!(
                "You are an expert Autonomous AI Workspace Engineer on Windows. Active workspace: {}. You have native tools for terminal commands, filesystem operations, and desktop automation. Fulfill the user's instructions completely and accurately.",
                cfg.workspace_path.display()
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
