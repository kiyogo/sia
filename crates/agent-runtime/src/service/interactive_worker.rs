use agent_platform::{InteractiveSessionProvider, SessionInfo};
use agent_storage::{Database, WorkflowTaskStatus};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InteractiveWorkerResult {
    Executed,
    WaitingForSession(String),
    TaskNotFound,
}

pub struct WindowsInteractiveWorker {
    session_provider: Arc<dyn InteractiveSessionProvider>,
    db: Arc<Database>,
}

impl WindowsInteractiveWorker {
    pub fn new(session_provider: Arc<dyn InteractiveSessionProvider>, db: Arc<Database>) -> Self {
        Self {
            session_provider,
            db,
        }
    }

    /// Dispatches a GUI-dependent task only if an interactive user session is active and unlocked
    pub async fn dispatch_gui_task(&self, task_id: Uuid) -> Result<InteractiveWorkerResult, String> {
        let session = self.session_provider.get_current_session();

        // Check Session 0 Isolation
        if session.is_session_zero {
            let reason = "Service running in non-interactive Session 0: GUI automation deferred".to_string();
            self.set_waiting_session(task_id, &reason).await?;
            return Ok(InteractiveWorkerResult::WaitingForSession(reason));
        }

        // Check if user session is locked
        if session.is_locked {
            let reason = "Windows desktop is locked: GUI automation deferred until unlocked".to_string();
            self.set_waiting_session(task_id, &reason).await?;
            return Ok(InteractiveWorkerResult::WaitingForSession(reason));
        }

        // Check if session is inactive
        if !session.is_active {
            let reason = "No active user logged in to interactive session".to_string();
            self.set_waiting_session(task_id, &reason).await?;
            return Ok(InteractiveWorkerResult::WaitingForSession(reason));
        }

        // Session 1+ is active and unlocked -> Safe to execute!
        if let Some(mut task) = self.db.get_workflow_task(task_id).await.map_err(|e| e.to_string())? {
            task.status = WorkflowTaskStatus::Running;
            self.db.update_workflow_task(&task).await.map_err(|e| e.to_string())?;
            Ok(InteractiveWorkerResult::Executed)
        } else {
            Ok(InteractiveWorkerResult::TaskNotFound)
        }
    }

    async fn set_waiting_session(&self, task_id: Uuid, reason: &str) -> Result<(), String> {
        if let Some(mut task) = self.db.get_workflow_task(task_id).await.map_err(|e| e.to_string())? {
            task.status = WorkflowTaskStatus::WaitingForInteractiveSession;
            task.error = Some(reason.to_string());
            self.db.update_workflow_task(&task).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn get_session_info(&self) -> SessionInfo {
        self.session_provider.get_current_session()
    }
}
