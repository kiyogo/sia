use agent_storage::{Database, JobExecution, ScheduledJob, StorageError};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggeredJobInfo {
    pub job_id: Uuid,
    pub job_name: String,
    pub execution_id: Uuid,
    pub workflow_template: String,
    pub triggered_at: DateTime<Utc>,
}

pub struct AutonomousScheduler {
    db: Arc<Database>,
    running_jobs: RwLock<HashSet<Uuid>>,
}

impl AutonomousScheduler {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            running_jobs: RwLock::new(HashSet::new()),
        }
    }

    /// Evaluates scheduled jobs in SQLite and triggers due executions according to missed-run and overlap policies
    pub async fn evaluate_and_trigger_due_jobs(&self) -> Result<Vec<TriggeredJobInfo>, StorageError> {
        let now = Utc::now();
        let jobs = self.db.list_scheduled_jobs().await?;
        let mut triggered = Vec::new();

        for job in jobs {
            if job.status != "Active" {
                continue;
            }

            let next_run = match job.next_run_at {
                Some(t) => t,
                None => continue,
            };

            if now >= next_run {
                // Job is due!
                let is_running = {
                    let running = self.running_jobs.read().await;
                    running.contains(&job.id)
                };

                // Check overlap policy
                if is_running {
                    match job.overlap_policy.as_str() {
                        "Skip" => {
                            // Skip this cycle, advance next_run
                            let next = self.compute_next_run(&job, now);
                            let _ = self.db.update_scheduled_job_execution(job.id, now, next).await;
                            continue;
                        }
                        "Queue" => {
                            // Defer until next scheduler tick
                            continue;
                        }
                        "Replace" => {
                            // In a real replacement, cancel active run; here we proceed
                        }
                        _ => {}
                    }
                }

                // Check missed run policy if downtime occurred
                let missed_seconds = (now - next_run).num_seconds();
                if missed_seconds > 600 && job.missed_run_policy == "Skip" {
                    // Machine was off for > 10 mins and policy is Skip -> Do not execute, jump to next future run
                    let next = self.compute_next_run(&job, now);
                    let _ = self.db.update_scheduled_job_execution(job.id, now, next).await;
                    continue;
                }

                // Trigger execution
                let exec_id = Uuid::new_v4();
                let exec = JobExecution {
                    id: exec_id,
                    job_id: job.id,
                    workflow_id: None,
                    scheduled_for: next_run,
                    started_at: now,
                    completed_at: None,
                    status: "Running".to_string(),
                    exit_reason: None,
                };
                self.db.record_job_execution(&exec).await?;

                // Mark as running in scheduler
                {
                    let mut running = self.running_jobs.write().await;
                    running.insert(job.id);
                }

                // Compute next execution
                let next = self.compute_next_run(&job, now);
                self.db.update_scheduled_job_execution(job.id, now, next).await?;

                triggered.push(TriggeredJobInfo {
                    job_id: job.id,
                    job_name: job.name.clone(),
                    execution_id: exec_id,
                    workflow_template: job.workflow_template.clone(),
                    triggered_at: now,
                });
            }
        }

        Ok(triggered)
    }

    /// Marks a job execution as completed, clearing active running status
    pub async fn mark_job_completed(&self, job_id: Uuid, exec_id: Uuid, success: bool, reason: &str) {
        let now = Utc::now();
        let status = if success { "Completed" } else { "Failed" };
        let _ = self.db.update_job_execution_status(exec_id, status, Some(now), Some(reason)).await;

        let mut running = self.running_jobs.write().await;
        running.remove(&job_id);
    }

    /// Calculates the next run timestamp based on schedule type and interval
    pub fn compute_next_run(&self, job: &ScheduledJob, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match job.schedule_type.as_str() {
            "once" => None, // One-shot job does not repeat
            "interval" => {
                let secs = job.interval_seconds.unwrap_or(3600).max(1);
                Some(from + Duration::seconds(secs))
            }
            "daily" => Some(from + Duration::days(1)),
            "weekly" => Some(from + Duration::weeks(1)),
            "monthly" => Some(from + Duration::days(30)),
            "cron" => {
                // Standard 1-hour fallback for mock/cron simulation
                let secs = job.interval_seconds.unwrap_or(3600).max(60);
                Some(from + Duration::seconds(secs))
            }
            _ => None,
        }
    }
}
