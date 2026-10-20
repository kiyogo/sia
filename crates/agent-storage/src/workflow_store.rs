use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Database, StorageError, StorageResult};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowStatus {
    Pending,
    Planning,
    Running,
    WaitingApproval,
    Pausing,
    Paused,
    Completed,
    PartiallyCompleted,
    Failed,
    Cancelled,
}

impl std::fmt::Display for WorkflowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowStatus::Pending => write!(f, "Pending"),
            WorkflowStatus::Planning => write!(f, "Planning"),
            WorkflowStatus::Running => write!(f, "Running"),
            WorkflowStatus::WaitingApproval => write!(f, "WaitingApproval"),
            WorkflowStatus::Pausing => write!(f, "Pausing"),
            WorkflowStatus::Paused => write!(f, "Paused"),
            WorkflowStatus::Completed => write!(f, "Completed"),
            WorkflowStatus::PartiallyCompleted => write!(f, "PartiallyCompleted"),
            WorkflowStatus::Failed => write!(f, "Failed"),
            WorkflowStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl std::str::FromStr for WorkflowStatus {
    type Err = StorageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(WorkflowStatus::Pending),
            "Planning" => Ok(WorkflowStatus::Planning),
            "Running" => Ok(WorkflowStatus::Running),
            "WaitingApproval" => Ok(WorkflowStatus::WaitingApproval),
            "Pausing" => Ok(WorkflowStatus::Pausing),
            "Paused" => Ok(WorkflowStatus::Paused),
            "Completed" => Ok(WorkflowStatus::Completed),
            "PartiallyCompleted" => Ok(WorkflowStatus::PartiallyCompleted),
            "Failed" => Ok(WorkflowStatus::Failed),
            "Cancelled" => Ok(WorkflowStatus::Cancelled),
            _ => Ok(WorkflowStatus::Pending),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowTaskStatus {
    Pending,
    Ready,
    Assigned,
    Running,
    WaitingApproval,
    WaitingResource,
    Verifying,
    Completed,
    Failed,
    Cancelled,
    Blocked,
}

impl std::fmt::Display for WorkflowTaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowTaskStatus::Pending => write!(f, "Pending"),
            WorkflowTaskStatus::Ready => write!(f, "Ready"),
            WorkflowTaskStatus::Assigned => write!(f, "Assigned"),
            WorkflowTaskStatus::Running => write!(f, "Running"),
            WorkflowTaskStatus::WaitingApproval => write!(f, "WaitingApproval"),
            WorkflowTaskStatus::WaitingResource => write!(f, "WaitingResource"),
            WorkflowTaskStatus::Verifying => write!(f, "Verifying"),
            WorkflowTaskStatus::Completed => write!(f, "Completed"),
            WorkflowTaskStatus::Failed => write!(f, "Failed"),
            WorkflowTaskStatus::Cancelled => write!(f, "Cancelled"),
            WorkflowTaskStatus::Blocked => write!(f, "Blocked"),
        }
    }
}

impl std::str::FromStr for WorkflowTaskStatus {
    type Err = StorageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(WorkflowTaskStatus::Pending),
            "Ready" => Ok(WorkflowTaskStatus::Ready),
            "Assigned" => Ok(WorkflowTaskStatus::Assigned),
            "Running" => Ok(WorkflowTaskStatus::Running),
            "WaitingApproval" => Ok(WorkflowTaskStatus::WaitingApproval),
            "WaitingResource" => Ok(WorkflowTaskStatus::WaitingResource),
            "Verifying" => Ok(WorkflowTaskStatus::Verifying),
            "Completed" => Ok(WorkflowTaskStatus::Completed),
            "Failed" => Ok(WorkflowTaskStatus::Failed),
            "Cancelled" => Ok(WorkflowTaskStatus::Cancelled),
            "Blocked" => Ok(WorkflowTaskStatus::Blocked),
            _ => Ok(WorkflowTaskStatus::Pending),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub title: String,
    pub goal: String,
    pub status: WorkflowStatus,
    pub budget_json: String,
    pub result_summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTask {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub parent_task_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub assigned_agent: String,
    pub status: WorkflowTaskStatus,
    pub priority: i32,
    pub inputs_json: String,
    pub outputs_json: String,
    pub acceptance_criteria: String,
    pub retry_count: u32,
    pub max_retries: u32,
    pub timeout_secs: u64,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    pub workflow_id: Uuid,
    pub task_id: Uuid,
    pub depends_on_task_id: Uuid,
    pub condition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAttempt {
    pub id: Uuid,
    pub task_id: Uuid,
    pub attempt_number: u32,
    pub agent_id: String,
    pub status: String,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub token_in: u64,
    pub token_out: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedArtifact {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub task_id: Option<Uuid>,
    pub artifact_type: String,
    pub name: String,
    pub path_or_uri: String,
    pub content_preview: String,
    pub hash: String,
    pub metadata_json: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLease {
    pub resource_id: String,
    pub resource_type: String,
    pub owner_agent_id: String,
    pub workflow_id: Uuid,
    pub task_id: Uuid,
    pub lease_until: DateTime<Utc>,
    pub fencing_token: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessageRecord {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub task_id: Option<Uuid>,
    pub source_agent: String,
    pub target_agent: String,
    pub message_type: String,
    pub payload_json: String,
    pub artifact_refs_json: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTokenRecord {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub task_id: Option<Uuid>,
    pub agent_type: String,
    pub token_in: u64,
    pub token_out: u64,
    pub estimated_cost: f64,
    pub created_at: DateTime<Utc>,
}

impl Database {
    // --- Workflow methods ---
    pub async fn create_workflow(&self, wf: &Workflow) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"INSERT INTO workflows (id, title, goal, status, budget_json, result_summary, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            params![
                wf.id.to_string(),
                wf.title,
                wf.goal,
                wf.status.to_string(),
                wf.budget_json,
                wf.result_summary,
                wf.created_at.to_rfc3339(),
                wf.updated_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub async fn get_workflow(&self, id: Uuid) -> StorageResult<Option<Workflow>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT id, title, goal, status, budget_json, result_summary, created_at, updated_at
               FROM workflows WHERE id = ?1"#,
        )?;
        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let title: String = row.get(1)?;
            let goal: String = row.get(2)?;
            let status_str: String = row.get(3)?;
            let budget_json: String = row.get(4)?;
            let result_summary: Option<String> = row.get(5)?;
            let created_at_str: String = row.get(6)?;
            let updated_at_str: String = row.get(7)?;

            Ok(Some(Workflow {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                title,
                goal,
                status: status_str.parse().unwrap_or(WorkflowStatus::Pending),
                budget_json,
                result_summary,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_workflows(&self) -> StorageResult<Vec<Workflow>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT id, title, goal, status, budget_json, result_summary, created_at, updated_at
               FROM workflows ORDER BY created_at DESC"#,
        )?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let title: String = row.get(1)?;
            let goal: String = row.get(2)?;
            let status_str: String = row.get(3)?;
            let budget_json: String = row.get(4)?;
            let result_summary: Option<String> = row.get(5)?;
            let created_at_str: String = row.get(6)?;
            let updated_at_str: String = row.get(7)?;

            Ok(Workflow {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                title,
                goal,
                status: status_str.parse().unwrap_or(WorkflowStatus::Pending),
                budget_json,
                result_summary,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub async fn update_workflow_status(
        &self,
        id: Uuid,
        status: WorkflowStatus,
        summary: Option<&str>,
    ) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"UPDATE workflows SET status = ?1, result_summary = COALESCE(?2, result_summary), updated_at = ?3
               WHERE id = ?4"#,
            params![
                status.to_string(),
                summary,
                Utc::now().to_rfc3339(),
                id.to_string()
            ],
        )?;
        Ok(())
    }

    // --- WorkflowTask methods ---
    pub async fn create_workflow_task(&self, task: &WorkflowTask) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"INSERT INTO workflow_tasks (
                id, workflow_id, parent_task_id, title, description, assigned_agent,
                status, priority, inputs_json, outputs_json, acceptance_criteria,
                retry_count, max_retries, timeout_secs, started_at, completed_at, error
               ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)"#,
            params![
                task.id.to_string(),
                task.workflow_id.to_string(),
                task.parent_task_id.map(|u| u.to_string()),
                task.title,
                task.description,
                task.assigned_agent,
                task.status.to_string(),
                task.priority,
                task.inputs_json,
                task.outputs_json,
                task.acceptance_criteria,
                task.retry_count,
                task.max_retries,
                task.timeout_secs as i64,
                task.started_at.map(|d| d.to_rfc3339()),
                task.completed_at.map(|d| d.to_rfc3339()),
                task.error,
            ],
        )?;
        Ok(())
    }

    pub async fn get_workflow_task(&self, id: Uuid) -> StorageResult<Option<WorkflowTask>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT id, workflow_id, parent_task_id, title, description, assigned_agent,
                      status, priority, inputs_json, outputs_json, acceptance_criteria,
                      retry_count, max_retries, timeout_secs, started_at, completed_at, error
               FROM workflow_tasks WHERE id = ?1"#,
        )?;
        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let wf_id_str: String = row.get(1)?;
            let parent_id_str: Option<String> = row.get(2)?;
            let title: String = row.get(3)?;
            let description: String = row.get(4)?;
            let assigned_agent: String = row.get(5)?;
            let status_str: String = row.get(6)?;
            let priority: i32 = row.get(7)?;
            let inputs_json: String = row.get(8)?;
            let outputs_json: String = row.get(9)?;
            let acceptance_criteria: String = row.get(10)?;
            let retry_count: u32 = row.get(11)?;
            let max_retries: u32 = row.get(12)?;
            let timeout_secs: i64 = row.get(13)?;
            let started_at_str: Option<String> = row.get(14)?;
            let completed_at_str: Option<String> = row.get(15)?;
            let error: Option<String> = row.get(16)?;

            Ok(Some(WorkflowTask {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                workflow_id: Uuid::parse_str(&wf_id_str).unwrap_or_default(),
                parent_task_id: parent_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
                title,
                description,
                assigned_agent,
                status: status_str.parse().unwrap_or(WorkflowTaskStatus::Pending),
                priority,
                inputs_json,
                outputs_json,
                acceptance_criteria,
                retry_count,
                max_retries,
                timeout_secs: timeout_secs as u64,
                started_at: started_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                completed_at: completed_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                error,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_workflow_tasks(&self, workflow_id: Uuid) -> StorageResult<Vec<WorkflowTask>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT id, workflow_id, parent_task_id, title, description, assigned_agent,
                      status, priority, inputs_json, outputs_json, acceptance_criteria,
                      retry_count, max_retries, timeout_secs, started_at, completed_at, error
               FROM workflow_tasks WHERE workflow_id = ?1 ORDER BY priority DESC, started_at ASC"#,
        )?;
        let rows = stmt.query_map(params![workflow_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let wf_id_str: String = row.get(1)?;
            let parent_id_str: Option<String> = row.get(2)?;
            let title: String = row.get(3)?;
            let description: String = row.get(4)?;
            let assigned_agent: String = row.get(5)?;
            let status_str: String = row.get(6)?;
            let priority: i32 = row.get(7)?;
            let inputs_json: String = row.get(8)?;
            let outputs_json: String = row.get(9)?;
            let acceptance_criteria: String = row.get(10)?;
            let retry_count: u32 = row.get(11)?;
            let max_retries: u32 = row.get(12)?;
            let timeout_secs: i64 = row.get(13)?;
            let started_at_str: Option<String> = row.get(14)?;
            let completed_at_str: Option<String> = row.get(15)?;
            let error: Option<String> = row.get(16)?;

            Ok(WorkflowTask {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                workflow_id: Uuid::parse_str(&wf_id_str).unwrap_or_default(),
                parent_task_id: parent_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
                title,
                description,
                assigned_agent,
                status: status_str.parse().unwrap_or(WorkflowTaskStatus::Pending),
                priority,
                inputs_json,
                outputs_json,
                acceptance_criteria,
                retry_count,
                max_retries,
                timeout_secs: timeout_secs as u64,
                started_at: started_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                completed_at: completed_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                error,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub async fn update_workflow_task_status(
        &self,
        id: Uuid,
        status: WorkflowTaskStatus,
        error: Option<&str>,
        outputs_json: Option<&str>,
    ) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        let now = Utc::now().to_rfc3339();
        let is_terminal = matches!(status, WorkflowTaskStatus::Completed | WorkflowTaskStatus::Failed | WorkflowTaskStatus::Cancelled);
        let completed_at = if is_terminal { Some(now.clone()) } else { None };
        let started_at = if status == WorkflowTaskStatus::Running { Some(now.clone()) } else { None };

        conn.execute(
            r#"UPDATE workflow_tasks SET
                status = ?1,
                error = ?2,
                outputs_json = COALESCE(?3, outputs_json),
                started_at = COALESCE(started_at, ?4),
                completed_at = COALESCE(?5, completed_at)
               WHERE id = ?6"#,
            params![
                status.to_string(),
                error,
                outputs_json,
                started_at,
                completed_at,
                id.to_string()
            ],
        )?;
        Ok(())
    }

    // --- Task Dependencies ---
    pub async fn add_task_dependency(&self, dep: &TaskDependency) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"INSERT OR REPLACE INTO task_dependencies (workflow_id, task_id, depends_on_task_id, condition)
               VALUES (?1, ?2, ?3, ?4)"#,
            params![
                dep.workflow_id.to_string(),
                dep.task_id.to_string(),
                dep.depends_on_task_id.to_string(),
                dep.condition
            ],
        )?;
        Ok(())
    }

    pub async fn get_task_dependencies(&self, workflow_id: Uuid) -> StorageResult<Vec<TaskDependency>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT workflow_id, task_id, depends_on_task_id, condition
               FROM task_dependencies WHERE workflow_id = ?1"#,
        )?;
        let rows = stmt.query_map(params![workflow_id.to_string()], |row| {
            let wf_str: String = row.get(0)?;
            let task_str: String = row.get(1)?;
            let dep_str: String = row.get(2)?;
            let condition: String = row.get(3)?;
            Ok(TaskDependency {
                workflow_id: Uuid::parse_str(&wf_str).unwrap_or_default(),
                task_id: Uuid::parse_str(&task_str).unwrap_or_default(),
                depends_on_task_id: Uuid::parse_str(&dep_str).unwrap_or_default(),
                condition,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // --- Task Attempts ---
    pub async fn create_task_attempt(&self, attempt: &TaskAttempt) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"INSERT INTO task_attempts (id, task_id, attempt_number, agent_id, status, error, started_at, completed_at, token_in, token_out)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                attempt.id.to_string(),
                attempt.task_id.to_string(),
                attempt.attempt_number,
                attempt.agent_id,
                attempt.status,
                attempt.error,
                attempt.started_at.to_rfc3339(),
                attempt.completed_at.map(|d| d.to_rfc3339()),
                attempt.token_in as i64,
                attempt.token_out as i64,
            ],
        )?;
        Ok(())
    }

    // --- Shared Artifacts ---
    pub async fn save_shared_artifact(&self, artifact: &SharedArtifact) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"INSERT OR REPLACE INTO shared_artifacts (id, workflow_id, task_id, artifact_type, name, path_or_uri, content_preview, hash, metadata_json, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                artifact.id.to_string(),
                artifact.workflow_id.to_string(),
                artifact.task_id.map(|u| u.to_string()),
                artifact.artifact_type,
                artifact.name,
                artifact.path_or_uri,
                artifact.content_preview,
                artifact.hash,
                artifact.metadata_json,
                artifact.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub async fn list_shared_artifacts(&self, workflow_id: Uuid) -> StorageResult<Vec<SharedArtifact>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT id, workflow_id, task_id, artifact_type, name, path_or_uri, content_preview, hash, metadata_json, created_at
               FROM shared_artifacts WHERE workflow_id = ?1 ORDER BY created_at ASC"#,
        )?;
        let rows = stmt.query_map(params![workflow_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let wf_str: String = row.get(1)?;
            let task_str: Option<String> = row.get(2)?;
            let artifact_type: String = row.get(3)?;
            let name: String = row.get(4)?;
            let path_or_uri: String = row.get(5)?;
            let content_preview: String = row.get(6)?;
            let hash: String = row.get(7)?;
            let metadata_json: String = row.get(8)?;
            let created_at_str: String = row.get(9)?;

            Ok(SharedArtifact {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                workflow_id: Uuid::parse_str(&wf_str).unwrap_or_default(),
                task_id: task_str.and_then(|s| Uuid::parse_str(&s).ok()),
                artifact_type,
                name,
                path_or_uri,
                content_preview,
                hash,
                metadata_json,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub async fn get_shared_artifact(&self, id: Uuid) -> StorageResult<Option<SharedArtifact>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT id, workflow_id, task_id, artifact_type, name, path_or_uri, content_preview, hash, metadata_json, created_at
               FROM shared_artifacts WHERE id = ?1"#,
        )?;
        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let wf_str: String = row.get(1)?;
            let task_str: Option<String> = row.get(2)?;
            let artifact_type: String = row.get(3)?;
            let name: String = row.get(4)?;
            let path_or_uri: String = row.get(5)?;
            let content_preview: String = row.get(6)?;
            let hash: String = row.get(7)?;
            let metadata_json: String = row.get(8)?;
            let created_at_str: String = row.get(9)?;

            Ok(Some(SharedArtifact {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                workflow_id: Uuid::parse_str(&wf_str).unwrap_or_default(),
                task_id: task_str.and_then(|s| Uuid::parse_str(&s).ok()),
                artifact_type,
                name,
                path_or_uri,
                content_preview,
                hash,
                metadata_json,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    // --- Resource Leases ---
    pub async fn acquire_resource_lease(&self, lease: &ResourceLease) -> StorageResult<bool> {
        let conn = self.conn.lock().await;

        // Check if existing lease is still valid and owned by someone else
        let mut stmt = conn.prepare("SELECT owner_agent_id, lease_until FROM resource_leases WHERE resource_id = ?1")?;
        let mut rows = stmt.query(params![lease.resource_id])?;
        if let Some(row) = rows.next()? {
            let owner: String = row.get(0)?;
            let until_str: String = row.get(1)?;
            let until = DateTime::parse_from_rfc3339(&until_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            if until > Utc::now() && owner != lease.owner_agent_id {
                // Resource is currently actively locked by another agent
                return Ok(false);
            }
        }

        conn.execute(
            r#"INSERT OR REPLACE INTO resource_leases (resource_id, resource_type, owner_agent_id, workflow_id, task_id, lease_until, fencing_token)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                lease.resource_id,
                lease.resource_type,
                lease.owner_agent_id,
                lease.workflow_id.to_string(),
                lease.task_id.to_string(),
                lease.lease_until.to_rfc3339(),
                lease.fencing_token as i64
            ],
        )?;
        Ok(true)
    }

    pub async fn release_resource_lease(&self, resource_id: &str, fencing_token: u64) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "DELETE FROM resource_leases WHERE resource_id = ?1 AND fencing_token = ?2",
            params![resource_id, fencing_token as i64],
        )?;
        Ok(())
    }

    pub async fn get_resource_lease(&self, resource_id: &str) -> StorageResult<Option<ResourceLease>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT resource_id, resource_type, owner_agent_id, workflow_id, task_id, lease_until, fencing_token
               FROM resource_leases WHERE resource_id = ?1"#,
        )?;
        let mut rows = stmt.query(params![resource_id])?;
        if let Some(row) = rows.next()? {
            let res_id: String = row.get(0)?;
            let res_type: String = row.get(1)?;
            let owner: String = row.get(2)?;
            let wf_str: String = row.get(3)?;
            let task_str: String = row.get(4)?;
            let until_str: String = row.get(5)?;
            let token: i64 = row.get(6)?;

            Ok(Some(ResourceLease {
                resource_id: res_id,
                resource_type: res_type,
                owner_agent_id: owner,
                workflow_id: Uuid::parse_str(&wf_str).unwrap_or_default(),
                task_id: Uuid::parse_str(&task_str).unwrap_or_default(),
                lease_until: DateTime::parse_from_rfc3339(&until_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                fencing_token: token as u64,
            }))
        } else {
            Ok(None)
        }
    }

    // --- Agent Messages ---
    pub async fn record_agent_message(&self, msg: &AgentMessageRecord) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"INSERT INTO agent_messages (id, workflow_id, task_id, source_agent, target_agent, message_type, payload_json, artifact_refs_json, timestamp)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![
                msg.id.to_string(),
                msg.workflow_id.to_string(),
                msg.task_id.map(|u| u.to_string()),
                msg.source_agent,
                msg.target_agent,
                msg.message_type,
                msg.payload_json,
                msg.artifact_refs_json,
                msg.timestamp.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub async fn list_agent_messages(&self, workflow_id: Uuid) -> StorageResult<Vec<AgentMessageRecord>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT id, workflow_id, task_id, source_agent, target_agent, message_type, payload_json, artifact_refs_json, timestamp
               FROM agent_messages WHERE workflow_id = ?1 ORDER BY timestamp ASC"#,
        )?;
        let rows = stmt.query_map(params![workflow_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let wf_str: String = row.get(1)?;
            let task_str: Option<String> = row.get(2)?;
            let source_agent: String = row.get(3)?;
            let target_agent: String = row.get(4)?;
            let message_type: String = row.get(5)?;
            let payload_json: String = row.get(6)?;
            let artifact_refs_json: String = row.get(7)?;
            let ts_str: String = row.get(8)?;

            Ok(AgentMessageRecord {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                workflow_id: Uuid::parse_str(&wf_str).unwrap_or_default(),
                task_id: task_str.and_then(|s| Uuid::parse_str(&s).ok()),
                source_agent,
                target_agent,
                message_type,
                payload_json,
                artifact_refs_json,
                timestamp: DateTime::parse_from_rfc3339(&ts_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // --- Workflow Token Usage ---
    pub async fn record_workflow_tokens(&self, record: &WorkflowTokenRecord) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"INSERT INTO workflow_token_usage (id, workflow_id, task_id, agent_type, token_in, token_out, estimated_cost, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            params![
                record.id.to_string(),
                record.workflow_id.to_string(),
                record.task_id.map(|u| u.to_string()),
                record.agent_type,
                record.token_in as i64,
                record.token_out as i64,
                record.estimated_cost,
                record.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub async fn get_workflow_token_usage(&self, workflow_id: Uuid) -> StorageResult<(u64, u64, f64)> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT COALESCE(SUM(token_in), 0), COALESCE(SUM(token_out), 0), COALESCE(SUM(estimated_cost), 0.0)
               FROM workflow_token_usage WHERE workflow_id = ?1"#,
        )?;
        let mut rows = stmt.query(params![workflow_id.to_string()])?;
        if let Some(row) = rows.next()? {
            let tin: i64 = row.get(0)?;
            let tout: i64 = row.get(1)?;
            let cost: f64 = row.get(2)?;
            Ok((tin as u64, tout as u64, cost))
        } else {
            Ok((0, 0, 0.0))
        }
    }
}
