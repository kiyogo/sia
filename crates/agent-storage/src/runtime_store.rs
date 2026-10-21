use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

use crate::{Database, StorageError, StorageResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: Uuid,
    pub name: String,
    pub schedule_type: String, // "once", "interval", "daily", "weekly", "monthly", "cron"
    pub cron_expression: Option<String>,
    pub interval_seconds: Option<i64>,
    pub timezone: String,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub status: String, // "Active", "Paused", "Disabled"
    pub missed_run_policy: String, // "Skip", "RunOnce", "CatchUp", "RequireApproval"
    pub overlap_policy: String, // "Skip", "Queue", "Replace", "RunConcurrently"
    pub max_runtime_seconds: Option<i64>,
    pub budget_limit: Option<f64>,
    pub workflow_template: String,
    pub payload_json: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecution {
    pub id: Uuid,
    pub job_id: Uuid,
    pub workflow_id: Option<Uuid>,
    pub scheduled_for: DateTime<Utc>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String, // "Running", "Completed", "Failed", "Skipped"
    pub exit_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisedProcess {
    pub id: Uuid,
    pub process_id: u32,
    pub process_type: String, // "WindowsMcp", "PlaywrightMcp", "OpenCode", "PythonWorker", "BackgroundJob"
    pub workspace_path: String,
    pub task_id: Option<Uuid>,
    pub start_time: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub health_status: String, // "Healthy", "Unresponsive", "Crashed", "Terminated", "Quarantined"
    pub restart_count: u32,
    pub resource_usage_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetReservation {
    pub id: Uuid,
    pub entity_type: String, // "Workflow", "Task", "Agent", "Global"
    pub entity_id: String,
    pub reserved_tokens: u64,
    pub reserved_cost: f64,
    pub status: String, // "Pending", "Committed", "Released", "Expired"
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEventRecord {
    pub id: Uuid,
    pub event_type: String,
    pub correlation_id: String,
    pub workflow_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub worker_id: Option<String>,
    pub payload_json: String,
    pub timestamp: DateTime<Utc>,
}

fn parse_utc(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))
}

impl Database {
    // -------------------------------------------------------------------------
    // Scheduled Jobs
    // -------------------------------------------------------------------------

    pub async fn create_scheduled_job(&self, job: &ScheduledJob) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO scheduled_jobs (
                id, name, schedule_type, cron_expression, interval_seconds, timezone,
                next_run_at, last_run_at, status, missed_run_policy, overlap_policy,
                max_runtime_seconds, budget_limit, workflow_template, payload_json,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                job.id.to_string(),
                job.name,
                job.schedule_type,
                job.cron_expression,
                job.interval_seconds,
                job.timezone,
                job.next_run_at.map(|t| t.to_rfc3339()),
                job.last_run_at.map(|t| t.to_rfc3339()),
                job.status,
                job.missed_run_policy,
                job.overlap_policy,
                job.max_runtime_seconds,
                job.budget_limit,
                job.workflow_template,
                job.payload_json,
                job.created_at.to_rfc3339(),
                job.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub async fn get_scheduled_job(&self, id: Uuid) -> StorageResult<Option<ScheduledJob>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, schedule_type, cron_expression, interval_seconds, timezone,
                    next_run_at, last_run_at, status, missed_run_policy, overlap_policy,
                    max_runtime_seconds, budget_limit, workflow_template, payload_json,
                    created_at, updated_at
             FROM scheduled_jobs WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let next_run_str: Option<String> = row.get(6)?;
            let last_run_str: Option<String> = row.get(7)?;
            let created_str: String = row.get(15)?;
            let updated_str: String = row.get(16)?;

            Ok(Some(ScheduledJob {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                name: row.get(1)?,
                schedule_type: row.get(2)?,
                cron_expression: row.get(3)?,
                interval_seconds: row.get(4)?,
                timezone: row.get(5)?,
                next_run_at: next_run_str.as_deref().and_then(parse_utc),
                last_run_at: last_run_str.as_deref().and_then(parse_utc),
                status: row.get(8)?,
                missed_run_policy: row.get(9)?,
                overlap_policy: row.get(10)?,
                max_runtime_seconds: row.get(11)?,
                budget_limit: row.get(12)?,
                workflow_template: row.get(13)?,
                payload_json: row.get(14)?,
                created_at: parse_utc(&created_str).unwrap_or_else(Utc::now),
                updated_at: parse_utc(&updated_str).unwrap_or_else(Utc::now),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_scheduled_jobs(&self) -> StorageResult<Vec<ScheduledJob>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, schedule_type, cron_expression, interval_seconds, timezone,
                    next_run_at, last_run_at, status, missed_run_policy, overlap_policy,
                    max_runtime_seconds, budget_limit, workflow_template, payload_json,
                    created_at, updated_at
             FROM scheduled_jobs ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let next_run_str: Option<String> = row.get(6)?;
            let last_run_str: Option<String> = row.get(7)?;
            let created_str: String = row.get(15)?;
            let updated_str: String = row.get(16)?;

            Ok(ScheduledJob {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                name: row.get(1)?,
                schedule_type: row.get(2)?,
                cron_expression: row.get(3)?,
                interval_seconds: row.get(4)?,
                timezone: row.get(5)?,
                next_run_at: next_run_str.as_deref().and_then(parse_utc),
                last_run_at: last_run_str.as_deref().and_then(parse_utc),
                status: row.get(8)?,
                missed_run_policy: row.get(9)?,
                overlap_policy: row.get(10)?,
                max_runtime_seconds: row.get(11)?,
                budget_limit: row.get(12)?,
                workflow_template: row.get(13)?,
                payload_json: row.get(14)?,
                created_at: parse_utc(&created_str).unwrap_or_else(Utc::now),
                updated_at: parse_utc(&updated_str).unwrap_or_else(Utc::now),
            })
        })?;

        let mut jobs = Vec::new();
        for r in rows {
            jobs.push(r?);
        }
        Ok(jobs)
    }

    pub async fn update_scheduled_job_execution(
        &self,
        job_id: Uuid,
        last_run: DateTime<Utc>,
        next_run: Option<DateTime<Utc>>,
    ) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE scheduled_jobs
             SET last_run_at = ?1, next_run_at = ?2, updated_at = ?3
             WHERE id = ?4",
            params![
                last_run.to_rfc3339(),
                next_run.map(|t| t.to_rfc3339()),
                Utc::now().to_rfc3339(),
                job_id.to_string()
            ],
        )?;
        Ok(())
    }

    pub async fn set_scheduled_job_status(&self, job_id: Uuid, status: &str) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE scheduled_jobs SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, Utc::now().to_rfc3339(), job_id.to_string()],
        )?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Job Executions
    // -------------------------------------------------------------------------

    pub async fn record_job_execution(&self, exec: &JobExecution) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO job_executions (
                id, job_id, workflow_id, scheduled_for, started_at, completed_at, status, exit_reason
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                exec.id.to_string(),
                exec.job_id.to_string(),
                exec.workflow_id.map(|w| w.to_string()),
                exec.scheduled_for.to_rfc3339(),
                exec.started_at.to_rfc3339(),
                exec.completed_at.map(|c| c.to_rfc3339()),
                exec.status,
                exec.exit_reason,
            ],
        )?;
        Ok(())
    }

    pub async fn update_job_execution_status(
        &self,
        exec_id: Uuid,
        status: &str,
        completed_at: Option<DateTime<Utc>>,
        exit_reason: Option<&str>,
    ) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE job_executions
             SET status = ?1, completed_at = ?2, exit_reason = ?3
             WHERE id = ?4",
            params![
                status,
                completed_at.map(|t| t.to_rfc3339()),
                exit_reason,
                exec_id.to_string()
            ],
        )?;
        Ok(())
    }

    pub async fn list_job_executions(&self, job_id: Uuid) -> StorageResult<Vec<JobExecution>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, job_id, workflow_id, scheduled_for, started_at, completed_at, status, exit_reason
             FROM job_executions
             WHERE job_id = ?1
             ORDER BY started_at DESC",
        )?;

        let rows = stmt.query_map(params![job_id.to_string()], |row| {
            let scheduled_str: String = row.get(3)?;
            let started_str: String = row.get(4)?;
            let completed_str: Option<String> = row.get(5)?;
            let wf_id_str: Option<String> = row.get(2)?;

            Ok(JobExecution {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                job_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                workflow_id: wf_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
                scheduled_for: parse_utc(&scheduled_str).unwrap_or_else(Utc::now),
                started_at: parse_utc(&started_str).unwrap_or_else(Utc::now),
                completed_at: completed_str.as_deref().and_then(parse_utc),
                status: row.get(6)?,
                exit_reason: row.get(7)?,
            })
        })?;

        let mut execs = Vec::new();
        for r in rows {
            execs.push(r?);
        }
        Ok(execs)
    }

    // -------------------------------------------------------------------------
    // Supervised Processes
    // -------------------------------------------------------------------------

    pub async fn register_supervised_process(&self, proc: &SupervisedProcess) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO supervised_processes (
                id, process_id, process_type, workspace_path, task_id, start_time,
                last_heartbeat, exit_code, health_status, restart_count, resource_usage_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                process_id = excluded.process_id,
                last_heartbeat = excluded.last_heartbeat,
                health_status = excluded.health_status,
                restart_count = excluded.restart_count,
                resource_usage_json = excluded.resource_usage_json",
            params![
                proc.id.to_string(),
                proc.process_id,
                proc.process_type,
                proc.workspace_path,
                proc.task_id.map(|t| t.to_string()),
                proc.start_time.to_rfc3339(),
                proc.last_heartbeat.to_rfc3339(),
                proc.exit_code,
                proc.health_status,
                proc.restart_count,
                proc.resource_usage_json,
            ],
        )?;
        Ok(())
    }

    pub async fn update_process_heartbeat(
        &self,
        id: Uuid,
        health_status: &str,
        resource_json: Option<&str>,
    ) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE supervised_processes
             SET last_heartbeat = ?1, health_status = ?2, resource_usage_json = ?3
             WHERE id = ?4",
            params![Utc::now().to_rfc3339(), health_status, resource_json, id.to_string()],
        )?;
        Ok(())
    }

    pub async fn mark_process_exited(&self, id: Uuid, exit_code: i32, health: &str) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE supervised_processes
             SET exit_code = ?1, health_status = ?2, last_heartbeat = ?3
             WHERE id = ?4",
            params![exit_code, health, Utc::now().to_rfc3339(), id.to_string()],
        )?;
        Ok(())
    }

    pub async fn list_supervised_processes(&self) -> StorageResult<Vec<SupervisedProcess>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, process_id, process_type, workspace_path, task_id, start_time,
                    last_heartbeat, exit_code, health_status, restart_count, resource_usage_json
             FROM supervised_processes ORDER BY start_time DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let start_str: String = row.get(5)?;
            let hb_str: String = row.get(6)?;
            let task_str: Option<String> = row.get(4)?;

            Ok(SupervisedProcess {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                process_id: row.get(1)?,
                process_type: row.get(2)?,
                workspace_path: row.get(3)?,
                task_id: task_str.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
                start_time: parse_utc(&start_str).unwrap_or_else(Utc::now),
                last_heartbeat: parse_utc(&hb_str).unwrap_or_else(Utc::now),
                exit_code: row.get(7)?,
                health_status: row.get(8)?,
                restart_count: row.get(9)?,
                resource_usage_json: row.get(10)?,
            })
        })?;

        let mut procs = Vec::new();
        for r in rows {
            procs.push(r?);
        }
        Ok(procs)
    }

    // -------------------------------------------------------------------------
    // Budget Reservations
    // -------------------------------------------------------------------------

    pub async fn create_budget_reservation(&self, res: &BudgetReservation) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO budget_reservations (
                id, entity_type, entity_id, reserved_tokens, reserved_cost, status, created_at, expires_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                res.id.to_string(),
                res.entity_type,
                res.entity_id,
                res.reserved_tokens as i64,
                res.reserved_cost,
                res.status,
                res.created_at.to_rfc3339(),
                res.expires_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub async fn resolve_budget_reservation(&self, id: Uuid, new_status: &str) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE budget_reservations SET status = ?1 WHERE id = ?2",
            params![new_status, id.to_string()],
        )?;
        Ok(())
    }

    pub async fn get_active_reserved_tokens(&self, entity_type: &str, entity_id: &str) -> StorageResult<u64> {
        let conn = self.conn.lock().await;
        let now_str = Utc::now().to_rfc3339();
        let mut stmt = conn.prepare(
            "SELECT COALESCE(SUM(reserved_tokens), 0)
             FROM budget_reservations
             WHERE entity_type = ?1 AND entity_id = ?2 AND status = 'Pending' AND expires_at > ?3",
        )?;
        let sum: i64 = stmt.query_row(params![entity_type, entity_id, now_str], |r| r.get(0))?;
        Ok(sum.max(0) as u64)
    }

    // -------------------------------------------------------------------------
    // Runtime Event Journal
    // -------------------------------------------------------------------------

    pub async fn record_runtime_event(&self, event: &RuntimeEventRecord) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO runtime_events (
                id, event_type, correlation_id, workflow_id, task_id, worker_id, payload_json, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event.id.to_string(),
                event.event_type,
                event.correlation_id,
                event.workflow_id.map(|w| w.to_string()),
                event.task_id.map(|t| t.to_string()),
                event.worker_id,
                event.payload_json,
                event.timestamp.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub async fn list_runtime_events(&self, limit: usize) -> StorageResult<Vec<RuntimeEventRecord>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, event_type, correlation_id, workflow_id, task_id, worker_id, payload_json, timestamp
             FROM runtime_events ORDER BY timestamp DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let task_str: Option<String> = row.get(4)?;
            let wf_str: Option<String> = row.get(3)?;
            let time_str: String = row.get(7)?;

            Ok(RuntimeEventRecord {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                event_type: row.get(1)?,
                correlation_id: row.get(2)?,
                workflow_id: wf_str.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
                task_id: task_str.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
                worker_id: row.get(5)?,
                payload_json: row.get(6)?,
                timestamp: parse_utc(&time_str).unwrap_or_else(Utc::now),
            })
        })?;

        let mut events = Vec::new();
        for r in rows {
            events.push(r?);
        }
        Ok(events)
    }

    // -------------------------------------------------------------------------
    // Online SQLite Backup API
    // -------------------------------------------------------------------------

    pub async fn backup_to(&self, destination_path: &Path) -> StorageResult<()> {
        let src_conn = self.conn.lock().await;
        let mut dest_conn = rusqlite::Connection::open(destination_path)
            .map_err(StorageError::Sqlite)?;

        let backup = rusqlite::backup::Backup::new(&src_conn, &mut dest_conn)
            .map_err(StorageError::Sqlite)?;

        // Run entire backup step (-1 copies all pages)
        backup
            .run_to_completion(100, std::time::Duration::from_millis(50), None)
            .map_err(StorageError::Sqlite)?;

        Ok(())
    }
}
