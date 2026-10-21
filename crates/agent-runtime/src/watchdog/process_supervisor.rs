use agent_storage::{Database, SupervisedProcess};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProcessMetadata {
    pub id: Uuid,
    pub pid: u32,
    pub process_type: String,
    pub workspace_path: String,
    pub task_id: Option<Uuid>,
    pub restart_count: u32,
    pub max_restarts: u32,
    pub last_heartbeat: chrono::DateTime<Utc>,
    pub is_quarantined: bool,
}

pub struct ProcessSupervisor {
    db: Arc<Database>,
    processes: RwLock<HashMap<Uuid, ProcessMetadata>>,
    heartbeat_timeout_secs: i64,
}

impl ProcessSupervisor {
    pub fn new(db: Arc<Database>, heartbeat_timeout_secs: i64) -> Self {
        Self {
            db,
            processes: RwLock::new(HashMap::new()),
            heartbeat_timeout_secs,
        }
    }

    /// Registers a newly launched child process for continuous supervision
    pub async fn register_process(
        &self,
        pid: u32,
        process_type: &str,
        workspace_path: &str,
        task_id: Option<Uuid>,
        max_restarts: u32,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let meta = ProcessMetadata {
            id,
            pid,
            process_type: process_type.to_string(),
            workspace_path: workspace_path.to_string(),
            task_id,
            restart_count: 0,
            max_restarts,
            last_heartbeat: now,
            is_quarantined: false,
        };

        // Persist to SQLite
        let db_record = SupervisedProcess {
            id,
            process_id: pid,
            process_type: process_type.to_string(),
            workspace_path: workspace_path.to_string(),
            task_id,
            start_time: now,
            last_heartbeat: now,
            exit_code: None,
            health_status: "Healthy".to_string(),
            restart_count: 0,
            resource_usage_json: None,
        };
        let _ = self.db.register_supervised_process(&db_record).await;

        let mut procs = self.processes.write().await;
        procs.insert(id, meta);

        id
    }

    /// Records heartbeat from a supervised worker process
    pub async fn record_heartbeat(&self, id: Uuid, resource_json: Option<&str>) {
        let now = Utc::now();
        let mut procs = self.processes.write().await;
        if let Some(meta) = procs.get_mut(&id) {
            meta.last_heartbeat = now;
            let _ = self.db.update_process_heartbeat(id, "Healthy", resource_json).await;
        }
    }

    /// Evaluates all supervised processes for heartbeats and crash conditions
    pub async fn check_health_and_reap(&self) -> Vec<(Uuid, String)> {
        let now = Utc::now();
        let mut events = Vec::new();

        let mut procs = self.processes.write().await;
        for (id, meta) in procs.iter_mut() {
            if meta.is_quarantined {
                continue;
            }

            let elapsed = (now - meta.last_heartbeat).num_seconds();
            if elapsed >= self.heartbeat_timeout_secs {
                // Heartbeat expired -> Check if process is still running
                let is_alive = is_pid_alive(meta.pid);
                if !is_alive {
                    // Process crashed
                    meta.restart_count += 1;
                    if meta.restart_count >= meta.max_restarts {
                        // Trip circuit breaker -> Quarantine process!
                        meta.is_quarantined = true;
                        let _ = self.db.mark_process_exited(*id, -1, "Quarantined").await;
                        events.push((*id, format!("Process {} entered CrashLoop -> Quarantined (no further restart)", meta.pid)));
                    } else {
                        let _ = self.db.mark_process_exited(*id, -1, "Crashed").await;
                        events.push((*id, format!("Process {} crashed (attempt {}/{}) -> Will restart", meta.pid, meta.restart_count, meta.max_restarts)));
                    }
                } else {
                    let _ = self.db.update_process_heartbeat(*id, "Unresponsive", None).await;
                    events.push((*id, format!("Process {} unresponsive (heartbeat timeout: {}s)", meta.pid, elapsed)));
                }
            }
        }

        events
    }

    /// Cleanly terminates a supervised process and its entire child process tree
    pub async fn kill_process_tree(&self, id: Uuid) -> bool {
        let mut procs = self.processes.write().await;
        if let Some(meta) = procs.remove(&id) {
            let _ = kill_pid_tree(meta.pid);
            let _ = self.db.mark_process_exited(id, 0, "Terminated").await;
            true
        } else {
            false
        }
    }

    pub async fn get_process_count(&self) -> usize {
        self.processes.read().await.len()
    }
}

fn is_pid_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new("powershell.exe");
        cmd.creation_flags(0x08000000);
        let out = cmd
            .args(["-NoProfile", "-NonInteractive", "-Command", &format!("if (Get-Process -Id {} -ErrorAction SilentlyContinue) {{ 'ALIVE' }}", pid)])
            .output();
        if let Ok(res) = out {
            String::from_utf8_lossy(&res.stdout).contains("ALIVE")
        } else {
            false
        }
    }
    #[cfg(not(windows))]
    {
        let res = unsafe { libc::kill(pid as i32, 0) };
        res == 0
    }
}

pub fn kill_pid_tree(pid: u32) -> bool {
    #[cfg(windows)]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new("taskkill.exe");
        cmd.creation_flags(0x08000000);
        let _ = cmd
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .output();
        true
    }
    #[cfg(not(windows))]
    {
        let _ = unsafe { libc::kill(-(pid as i32), libc::SIGKILL) };
        true
    }
}
