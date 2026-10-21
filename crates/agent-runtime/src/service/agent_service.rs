use crate::recovery::{CrashRecoveryEngine, RecoverySummary};
use crate::scheduler::AutonomousScheduler;
use crate::watchdog::ProcessSupervisor;
use agent_core::{BudgetGovernanceEngine, BudgetHierarchyConfig};
use agent_platform::{ServiceStatus, SystemResourceMonitor, SystemResourceSnapshot};
use agent_storage::Database;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AgentRuntimeService {
    db: Arc<Database>,
    scheduler: Arc<AutonomousScheduler>,
    watchdog: Arc<ProcessSupervisor>,
    recovery: Arc<CrashRecoveryEngine>,
    budget: Arc<BudgetGovernanceEngine>,
    resources: Arc<SystemResourceMonitor>,
    is_running: AtomicBool,
    status: RwLock<ServiceStatus>,
}

impl AgentRuntimeService {
    pub fn new(db: Arc<Database>) -> Self {
        let scheduler = Arc::new(AutonomousScheduler::new(db.clone()));
        let watchdog = Arc::new(ProcessSupervisor::new(db.clone(), 60));
        let recovery = Arc::new(CrashRecoveryEngine::new(db.clone()));
        let budget = Arc::new(BudgetGovernanceEngine::new(BudgetHierarchyConfig::default()));
        let resources = Arc::new(SystemResourceMonitor::new());

        Self {
            db,
            scheduler,
            watchdog,
            recovery,
            budget,
            resources,
            is_running: AtomicBool::new(false),
            status: RwLock::new(ServiceStatus::Stopped),
        }
    }

    /// Initializes and starts the background runtime service
    pub async fn start(&self) -> Result<RecoverySummary, String> {
        {
            let mut s = self.status.write().await;
            *s = ServiceStatus::StartPending;
        }

        // 1. Startup Crash Recovery: Reconcile any in-flight tasks from previous run/reboot
        let recovery_summary = self
            .recovery
            .reconcile_startup_tasks()
            .await
            .map_err(|e| format!("Recovery failed: {e}"))?;

        self.is_running.store(true, Ordering::SeqCst);
        {
            let mut s = self.status.write().await;
            *s = ServiceStatus::Running;
        }

        Ok(recovery_summary)
    }

    /// Graceful shutdown of the background runtime service
    pub async fn stop(&self) {
        {
            let mut s = self.status.write().await;
            *s = ServiceStatus::StopPending;
        }

        self.is_running.store(false, Ordering::SeqCst);

        // Sleep briefly to simulate draining active worker queues
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        {
            let mut s = self.status.write().await;
            *s = ServiceStatus::Stopped;
        }
    }

    pub fn database(&self) -> &Arc<Database> {
        &self.db
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub async fn get_status(&self) -> ServiceStatus {
        *self.status.read().await
    }

    pub fn scheduler(&self) -> &Arc<AutonomousScheduler> {
        &self.scheduler
    }

    pub fn watchdog(&self) -> &Arc<ProcessSupervisor> {
        &self.watchdog
    }

    pub fn recovery(&self) -> &Arc<CrashRecoveryEngine> {
        &self.recovery
    }

    pub fn budget(&self) -> &Arc<BudgetGovernanceEngine> {
        &self.budget
    }

    pub fn resources(&self) -> &Arc<SystemResourceMonitor> {
        &self.resources
    }

    pub fn get_resource_snapshot(&self) -> SystemResourceSnapshot {
        self.resources.get_snapshot()
    }
}
