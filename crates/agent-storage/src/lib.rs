use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

pub mod workflow_store;
pub use workflow_store::*;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Record not found: {0}")]
    NotFound(String),
}

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub token_in: u64,
    pub token_out: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub tool_calls: Option<serde_json::Value>,
    pub tool_call_id: Option<String>,
    pub token_count: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    WaitingApproval,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::Running => write!(f, "Running"),
            TaskStatus::WaitingApproval => write!(f, "WaitingApproval"),
            TaskStatus::Paused => write!(f, "Paused"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Failed => write!(f, "Failed"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = StorageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(TaskStatus::Pending),
            "Running" => Ok(TaskStatus::Running),
            "WaitingApproval" => Ok(TaskStatus::WaitingApproval),
            "Paused" => Ok(TaskStatus::Paused),
            "Completed" => Ok(TaskStatus::Completed),
            "Failed" => Ok(TaskStatus::Failed),
            "Cancelled" => Ok(TaskStatus::Cancelled),
            _ => Ok(TaskStatus::Pending),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub title: String,
    pub status: TaskStatus,
    pub goal: String,
    pub current_step: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: Uuid,
    pub task_id: Uuid,
    pub step_index: u32,
    pub state_json: String,
    pub summary: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAuditLog {
    pub id: Uuid,
    pub task_id: Option<Uuid>,
    pub tool_name: String,
    pub parameters_json: String,
    pub output_preview: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub approved_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let conn = Connection::open(path)?;
        Self::run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn open_in_memory() -> StorageResult<Self> {
        let conn = Connection::open_in_memory()?;
        Self::run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn run_migrations(conn: &Connection) -> StorageResult<()> {
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                token_in INTEGER NOT NULL DEFAULT 0,
                token_out INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                tool_calls TEXT,
                tool_call_id TEXT,
                token_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                conversation_id TEXT,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                goal TEXT NOT NULL,
                current_step TEXT NOT NULL,
                result TEXT,
                error TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                step_index INTEGER NOT NULL,
                state_json TEXT NOT NULL,
                summary TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS tool_audit_logs (
                id TEXT PRIMARY KEY,
                task_id TEXT,
                tool_name TEXT NOT NULL,
                parameters_json TEXT NOT NULL,
                output_preview TEXT NOT NULL,
                exit_code INTEGER,
                duration_ms INTEGER NOT NULL,
                approved_by TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS workflows (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                goal TEXT NOT NULL,
                status TEXT NOT NULL,
                budget_json TEXT NOT NULL,
                result_summary TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS workflow_tasks (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                parent_task_id TEXT,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                assigned_agent TEXT NOT NULL,
                status TEXT NOT NULL,
                priority INTEGER NOT NULL DEFAULT 0,
                inputs_json TEXT NOT NULL DEFAULT '{}',
                outputs_json TEXT NOT NULL DEFAULT '{}',
                acceptance_criteria TEXT NOT NULL DEFAULT '',
                retry_count INTEGER NOT NULL DEFAULT 0,
                max_retries INTEGER NOT NULL DEFAULT 3,
                timeout_secs INTEGER NOT NULL DEFAULT 300,
                started_at TEXT,
                completed_at TEXT,
                error TEXT,
                FOREIGN KEY(workflow_id) REFERENCES workflows(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS task_dependencies (
                workflow_id TEXT NOT NULL,
                task_id TEXT NOT NULL,
                depends_on_task_id TEXT NOT NULL,
                condition TEXT NOT NULL DEFAULT 'Success',
                PRIMARY KEY(workflow_id, task_id, depends_on_task_id),
                FOREIGN KEY(workflow_id) REFERENCES workflows(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS task_attempts (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                attempt_number INTEGER NOT NULL,
                agent_id TEXT NOT NULL,
                status TEXT NOT NULL,
                error TEXT,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                token_in INTEGER NOT NULL DEFAULT 0,
                token_out INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY(task_id) REFERENCES workflow_tasks(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS shared_artifacts (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                task_id TEXT,
                artifact_type TEXT NOT NULL,
                name TEXT NOT NULL,
                path_or_uri TEXT NOT NULL,
                content_preview TEXT NOT NULL,
                hash TEXT NOT NULL,
                metadata_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                FOREIGN KEY(workflow_id) REFERENCES workflows(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS resource_leases (
                resource_id TEXT PRIMARY KEY,
                resource_type TEXT NOT NULL,
                owner_agent_id TEXT NOT NULL,
                workflow_id TEXT NOT NULL,
                task_id TEXT NOT NULL,
                lease_until TEXT NOT NULL,
                fencing_token INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS agent_messages (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                task_id TEXT,
                source_agent TEXT NOT NULL,
                target_agent TEXT NOT NULL,
                message_type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                artifact_refs_json TEXT NOT NULL DEFAULT '[]',
                timestamp TEXT NOT NULL,
                FOREIGN KEY(workflow_id) REFERENCES workflows(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS workflow_token_usage (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                task_id TEXT,
                agent_type TEXT NOT NULL,
                token_in INTEGER NOT NULL,
                token_out INTEGER NOT NULL,
                estimated_cost REAL NOT NULL DEFAULT 0.0,
                created_at TEXT NOT NULL,
                FOREIGN KEY(workflow_id) REFERENCES workflows(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
            CREATE INDEX IF NOT EXISTS idx_checkpoints_task ON checkpoints(task_id);
            CREATE INDEX IF NOT EXISTS idx_workflow_tasks_workflow ON workflow_tasks(workflow_id);
            CREATE INDEX IF NOT EXISTS idx_workflow_tasks_status ON workflow_tasks(status);
            CREATE INDEX IF NOT EXISTS idx_task_deps_task ON task_dependencies(task_id);
            CREATE INDEX IF NOT EXISTS idx_artifacts_workflow ON shared_artifacts(workflow_id);
            CREATE INDEX IF NOT EXISTS idx_agent_msgs_workflow ON agent_messages(workflow_id);
            "#,
        )?;
        Ok(())
    }

    // --- Conversation methods ---
    pub async fn create_conversation(&self, title: &str) -> StorageResult<Conversation> {
        let conv = Conversation {
            id: Uuid::new_v4(),
            title: title.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            token_in: 0,
            token_out: 0,
        };
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO conversations (id, title, created_at, updated_at, token_in, token_out) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                conv.id.to_string(),
                conv.title,
                conv.created_at.to_rfc3339(),
                conv.updated_at.to_rfc3339(),
                conv.token_in,
                conv.token_out
            ],
        )?;
        Ok(conv)
    }

    pub async fn list_conversations(&self) -> StorageResult<Vec<Conversation>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT id, title, created_at, updated_at, token_in, token_out FROM conversations ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let title: String = row.get(1)?;
            let created_at_str: String = row.get(2)?;
            let updated_at_str: String = row.get(3)?;
            let token_in: u64 = row.get(4)?;
            let token_out: u64 = row.get(5)?;

            Ok(Conversation {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                title,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                token_in,
                token_out,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // --- Message methods ---
    pub async fn add_message(&self, msg: &Message) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        let tool_calls_json = msg.tool_calls.as_ref().map(|v| v.to_string());
        conn.execute(
            r#"INSERT INTO messages (id, conversation_id, role, content, tool_calls, tool_call_id, token_count, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            params![
                msg.id.to_string(),
                msg.conversation_id.to_string(),
                msg.role,
                msg.content,
                tool_calls_json,
                msg.tool_call_id,
                msg.token_count,
                msg.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub async fn get_messages(&self, conversation_id: Uuid) -> StorageResult<Vec<Message>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT id, conversation_id, role, content, tool_calls, tool_call_id, token_count, created_at
               FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC"#,
        )?;
        let rows = stmt.query_map(params![conversation_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let conv_id_str: String = row.get(1)?;
            let role: String = row.get(2)?;
            let content: String = row.get(3)?;
            let tool_calls_raw: Option<String> = row.get(4)?;
            let tool_call_id: Option<String> = row.get(5)?;
            let token_count: u32 = row.get(6)?;
            let created_at_str: String = row.get(7)?;

            let tool_calls = tool_calls_raw.and_then(|s| serde_json::from_str(&s).ok());

            Ok(Message {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                conversation_id: Uuid::parse_str(&conv_id_str).unwrap_or_default(),
                role,
                content,
                tool_calls,
                tool_call_id,
                token_count,
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

    // --- Task methods ---
    pub async fn create_task(&self, task: &Task) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"INSERT INTO tasks (id, conversation_id, title, status, goal, current_step, result, error, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                task.id.to_string(),
                task.conversation_id.map(|id| id.to_string()),
                task.title,
                task.status.to_string(),
                task.goal,
                task.current_step,
                task.result,
                task.error,
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub async fn update_task_status(
        &self,
        task_id: Uuid,
        status: TaskStatus,
        current_step: &str,
        result: Option<&str>,
        error: Option<&str>,
    ) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"UPDATE tasks SET status = ?1, current_step = ?2, result = ?3, error = ?4, updated_at = ?5 WHERE id = ?6"#,
            params![
                status.to_string(),
                current_step,
                result,
                error,
                Utc::now().to_rfc3339(),
                task_id.to_string()
            ],
        )?;
        Ok(())
    }

    pub async fn list_tasks(&self) -> StorageResult<Vec<Task>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT id, conversation_id, title, status, goal, current_step, result, error, created_at, updated_at
               FROM tasks ORDER BY updated_at DESC"#,
        )?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let conv_id_str: Option<String> = row.get(1)?;
            let title: String = row.get(2)?;
            let status_str: String = row.get(3)?;
            let goal: String = row.get(4)?;
            let current_step: String = row.get(5)?;
            let result: Option<String> = row.get(6)?;
            let error: Option<String> = row.get(7)?;
            let created_at_str: String = row.get(8)?;
            let updated_at_str: String = row.get(9)?;

            Ok(Task {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                conversation_id: conv_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
                title,
                status: status_str.parse().unwrap_or(TaskStatus::Pending),
                goal,
                current_step,
                result,
                error,
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

    // --- Checkpoint methods ---
    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"INSERT INTO checkpoints (id, task_id, step_index, state_json, summary, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
            params![
                checkpoint.id.to_string(),
                checkpoint.task_id.to_string(),
                checkpoint.step_index,
                checkpoint.state_json,
                checkpoint.summary,
                checkpoint.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub async fn get_latest_checkpoint(&self, task_id: Uuid) -> StorageResult<Option<Checkpoint>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"SELECT id, task_id, step_index, state_json, summary, created_at
               FROM checkpoints WHERE task_id = ?1 ORDER BY step_index DESC LIMIT 1"#,
        )?;
        let mut rows = stmt.query(params![task_id.to_string()])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let t_id_str: String = row.get(1)?;
            let step_index: u32 = row.get(2)?;
            let state_json: String = row.get(3)?;
            let summary: String = row.get(4)?;
            let created_at_str: String = row.get(5)?;

            Ok(Some(Checkpoint {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                task_id: Uuid::parse_str(&t_id_str).unwrap_or_default(),
                step_index,
                state_json,
                summary,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    // --- Audit log methods ---
    pub async fn log_tool_audit(&self, log: &ToolAuditLog) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"INSERT INTO tool_audit_logs (id, task_id, tool_name, parameters_json, output_preview, exit_code, duration_ms, approved_by, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![
                log.id.to_string(),
                log.task_id.map(|id| id.to_string()),
                log.tool_name,
                log.parameters_json,
                log.output_preview,
                log.exit_code,
                log.duration_ms,
                log.approved_by,
                log.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_conversations_and_messages() {
        let db = Database::open_in_memory().expect("open memory db");
        let conv = db
            .create_conversation("Test Workspace")
            .await
            .expect("create conv");

        let msg = Message {
            id: Uuid::new_v4(),
            conversation_id: conv.id,
            role: "user".to_string(),
            content: "Check cargo version".to_string(),
            tool_calls: None,
            tool_call_id: None,
            token_count: 5,
            created_at: Utc::now(),
        };

        db.add_message(&msg).await.expect("add message");
        let messages = db.get_messages(conv.id).await.expect("get messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Check cargo version");
    }

    #[tokio::test]
    async fn test_tasks_and_checkpoints() {
        let db = Database::open_in_memory().expect("open memory db");
        let task_id = Uuid::new_v4();
        let task = Task {
            id: task_id,
            conversation_id: None,
            title: "Run unit tests".to_string(),
            status: TaskStatus::Running,
            goal: "Run tests and verify".to_string(),
            current_step: "Executing cargo test".to_string(),
            result: None,
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        db.create_task(&task).await.expect("create task");

        let ck = Checkpoint {
            id: Uuid::new_v4(),
            task_id,
            step_index: 1,
            state_json: r#"{"step":"test"}"#.to_string(),
            summary: "Executed step 1".to_string(),
            created_at: Utc::now(),
        };

        db.save_checkpoint(&ck).await.expect("save ck");
        let latest = db
            .get_latest_checkpoint(task_id)
            .await
            .expect("get ck")
            .expect("must exist");
        assert_eq!(latest.step_index, 1);
    }

    #[tokio::test]
    async fn test_multi_agent_workflow_storage() {
        let db = Database::open_in_memory().expect("open memory db");
        let wf_id = Uuid::new_v4();
        let wf = Workflow {
            id: wf_id,
            title: "Build & Test Web App".to_string(),
            goal: "Full lifecycle build, test, and verification".to_string(),
            status: WorkflowStatus::Running,
            budget_json: r#"{"max_tokens":50000}"#.to_string(),
            result_summary: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        db.create_workflow(&wf).await.expect("create workflow");
        let fetched_wf = db.get_workflow(wf_id).await.expect("get wf").expect("must exist");
        assert_eq!(fetched_wf.title, "Build & Test Web App");

        // Create tasks
        let task1_id = Uuid::new_v4();
        let task1 = WorkflowTask {
            id: task1_id,
            workflow_id: wf_id,
            parent_task_id: None,
            title: "Generate React Component".to_string(),
            description: "Code a high performance component".to_string(),
            assigned_agent: "Coding".to_string(),
            status: WorkflowTaskStatus::Completed,
            priority: 10,
            inputs_json: "{}".to_string(),
            outputs_json: r#"{"files":["App.tsx"]}"#.to_string(),
            acceptance_criteria: "Compiles cleanly".to_string(),
            retry_count: 0,
            max_retries: 3,
            timeout_secs: 60,
            started_at: Some(Utc::now()),
            completed_at: Some(Utc::now()),
            error: None,
        };
        db.create_workflow_task(&task1).await.expect("create task1");

        let task2_id = Uuid::new_v4();
        let task2 = WorkflowTask {
            id: task2_id,
            workflow_id: wf_id,
            parent_task_id: None,
            title: "Verify in Headless Browser".to_string(),
            description: "Navigate and check DOM tree".to_string(),
            assigned_agent: "Browser".to_string(),
            status: WorkflowTaskStatus::Ready,
            priority: 5,
            inputs_json: "{}".to_string(),
            outputs_json: "{}".to_string(),
            acceptance_criteria: "Title matches".to_string(),
            retry_count: 0,
            max_retries: 3,
            timeout_secs: 60,
            started_at: None,
            completed_at: None,
            error: None,
        };
        db.create_workflow_task(&task2).await.expect("create task2");

        // Add dependency: task2 depends on task1
        let dep = TaskDependency {
            workflow_id: wf_id,
            task_id: task2_id,
            depends_on_task_id: task1_id,
            condition: "Success".to_string(),
        };
        db.add_task_dependency(&dep).await.expect("add dependency");

        let deps = db.get_task_dependencies(wf_id).await.expect("get deps");
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].task_id, task2_id);
        assert_eq!(deps[0].depends_on_task_id, task1_id);

        // Save artifact
        let art = SharedArtifact {
            id: Uuid::new_v4(),
            workflow_id: wf_id,
            task_id: Some(task1_id),
            artifact_type: "CodePatch".to_string(),
            name: "App.tsx".to_string(),
            path_or_uri: "src/App.tsx".to_string(),
            content_preview: "export function App() {}".to_string(),
            hash: "sha256:abc1234".to_string(),
            metadata_json: "{}".to_string(),
            created_at: Utc::now(),
        };
        db.save_shared_artifact(&art).await.expect("save artifact");
        let arts = db.list_shared_artifacts(wf_id).await.expect("list arts");
        assert_eq!(arts.len(), 1);
        assert_eq!(arts[0].name, "App.tsx");

        // Acquire and release resource lease
        let lease = ResourceLease {
            resource_id: "desktop:primary".to_string(),
            resource_type: "Desktop".to_string(),
            owner_agent_id: "ComputerAgent".to_string(),
            workflow_id: wf_id,
            task_id: task1_id,
            lease_until: Utc::now() + chrono::Duration::seconds(30),
            fencing_token: 1,
        };
        let acquired = db.acquire_resource_lease(&lease).await.expect("acquire lease");
        assert!(acquired);

        // Try to acquire same lease with different agent should be denied
        let conflicting_lease = ResourceLease {
            resource_id: "desktop:primary".to_string(),
            resource_type: "Desktop".to_string(),
            owner_agent_id: "AnotherAgent".to_string(),
            workflow_id: wf_id,
            task_id: task2_id,
            lease_until: Utc::now() + chrono::Duration::seconds(30),
            fencing_token: 2,
        };
        let conflict_acquired = db.acquire_resource_lease(&conflicting_lease).await.expect("check conflict");
        assert!(!conflict_acquired);

        // Release lease
        db.release_resource_lease("desktop:primary", 1).await.expect("release lease");
    }
}
