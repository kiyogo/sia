use agent_storage::{Database, RuntimeEventRecord, StorageError, WorkflowTaskStatus};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySummary {
    pub tasks_recovered_to_ready: usize,
    pub tasks_flagged_outcome_unknown: usize,
    pub tasks_marked_failed: usize,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UnknownReconciliationDecision {
    RetrySafeStep,
    MarkCompleted,
    Cancel,
}

pub struct CrashRecoveryEngine {
    db: Arc<Database>,
}

impl CrashRecoveryEngine {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Scans database on startup and safely reconciles in-flight tasks from a crash or reboot
    pub async fn reconcile_startup_tasks(&self) -> Result<RecoverySummary, StorageError> {
        let tasks = self.db.list_all_in_flight_tasks().await?;

        let mut summary = RecoverySummary {
            tasks_recovered_to_ready: 0,
            tasks_flagged_outcome_unknown: 0,
            tasks_marked_failed: 0,
            details: Vec::new(),
        };

        for mut task in tasks {
            let task_id = task.id;
            let title = task.title.clone();

            // Determine if the task had side effects
            let is_side_effect = task.assigned_agent == "Email"
                || title.to_lowercase().contains("send")
                || title.to_lowercase().contains("delete")
                || title.to_lowercase().contains("commit")
                || title.to_lowercase().contains("dispatch");

            if task.retry_count >= task.max_retries {
                // Max attempts reached -> Mark Failed
                task.status = WorkflowTaskStatus::Failed;
                self.db.update_workflow_task(&task).await?;
                summary.tasks_marked_failed += 1;
                summary.details.push(format!("Task {} ('{}') marked Failed (max attempts reached)", task_id, title));
            } else if is_side_effect {
                // Side-effect task in flight during crash -> MUST NOT blindly retry! Mark OutcomeUnknown
                task.status = WorkflowTaskStatus::OutcomeUnknown;
                self.db.update_workflow_task(&task).await?;
                summary.tasks_flagged_outcome_unknown += 1;
                summary.details.push(format!(
                    "Task {} ('{}') marked OutcomeUnknown (external side-effect requires verification)",
                    task_id, title
                ));

                // Record event
                let evt = RuntimeEventRecord {
                    id: Uuid::new_v4(),
                    event_type: "TaskOutcomeUnknown".to_string(),
                    correlation_id: format!("recov_{}", task_id),
                    workflow_id: Some(task.workflow_id),
                    task_id: Some(task_id),
                    worker_id: Some("CrashRecoveryEngine".to_string()),
                    payload_json: r#"{"reason":"In-flight side effect interrupted during crash"}"#.to_string(),
                    timestamp: Utc::now(),
                };
                let _ = self.db.record_runtime_event(&evt).await;
            } else {
                // Pure / idempotent task -> Safely recover to Ready
                task.status = WorkflowTaskStatus::Ready;
                task.retry_count += 1;
                self.db.update_workflow_task(&task).await?;
                summary.tasks_recovered_to_ready += 1;
                summary.details.push(format!("Task {} ('{}') recovered to Ready (attempt {})", task_id, title, task.retry_count));

                let evt = RuntimeEventRecord {
                    id: Uuid::new_v4(),
                    event_type: "TaskRecovered".to_string(),
                    correlation_id: format!("recov_{}", task_id),
                    workflow_id: Some(task.workflow_id),
                    task_id: Some(task_id),
                    worker_id: Some("CrashRecoveryEngine".to_string()),
                    payload_json: r#"{"status":"Ready","recovered":true}"#.to_string(),
                    timestamp: Utc::now(),
                };
                let _ = self.db.record_runtime_event(&evt).await;
            }
        }

        Ok(summary)
    }

    /// Reconciles a task in OutcomeUnknown state based on operator or supervisor verification
    pub async fn reconcile_unknown_outcome(
        &self,
        task_id: Uuid,
        decision: UnknownReconciliationDecision,
    ) -> Result<(), StorageError> {
        if let Some(mut task) = self.db.get_workflow_task(task_id).await? {
            match decision {
                UnknownReconciliationDecision::RetrySafeStep => {
                    task.status = WorkflowTaskStatus::Ready;
                    task.retry_count += 1;
                }
                UnknownReconciliationDecision::MarkCompleted => {
                    task.status = WorkflowTaskStatus::Completed;
                }
                UnknownReconciliationDecision::Cancel => {
                    task.status = WorkflowTaskStatus::Cancelled;
                }
            }
            self.db.update_workflow_task(&task).await?;

            let evt = RuntimeEventRecord {
                id: Uuid::new_v4(),
                event_type: "OutcomeReconciled".to_string(),
                correlation_id: format!("reconcile_{}", task_id),
                workflow_id: Some(task.workflow_id),
                task_id: Some(task_id),
                worker_id: Some("Operator".to_string()),
                payload_json: format!(r#"{{"decision":"{:?}"}}"#, decision),
                timestamp: Utc::now(),
            };
            let _ = self.db.record_runtime_event(&evt).await;
        }
        Ok(())
    }
}
