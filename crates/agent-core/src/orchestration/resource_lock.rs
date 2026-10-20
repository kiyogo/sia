use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ResourceLockError {
    #[error("Resource '{resource}' is currently locked by agent '{owner}' until {until}")]
    ResourceBusy {
        resource: String,
        owner: String,
        until: DateTime<Utc>,
    },

    #[error("File conflict: path '{0}' is actively locked by another agent")]
    FileConflict(PathBuf),

    #[error("Stale fencing token {token} for resource '{resource}'. Current active token is {current}")]
    StaleFencingToken {
        resource: String,
        token: u64,
        current: u64,
    },

    #[error("Lock not found for resource '{0}'")]
    LockNotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveLease {
    pub resource_id: String,
    pub owner_agent: String,
    pub workflow_id: Uuid,
    pub task_id: Uuid,
    pub acquired_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub fencing_token: u64,
}

#[derive(Debug)]
pub struct ResourceLockManager {
    leases: Arc<Mutex<HashMap<String, ActiveLease>>>,
    file_locks: Arc<Mutex<HashMap<PathBuf, (String, Uuid, u64)>>>, // Path -> (Owner, TaskId, FencingToken)
    fencing_seq: AtomicU64,
}

impl Default for ResourceLockManager {
    fn default() -> Self {
        Self {
            leases: Arc::new(Mutex::new(HashMap::new())),
            file_locks: Arc::new(Mutex::new(HashMap::new())),
            fencing_seq: AtomicU64::new(1),
        }
    }
}

impl ResourceLockManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Acquires a named resource lease (e.g. "desktop:primary", "browser:context:1") with fencing token
    pub async fn acquire_lease(
        &self,
        resource_id: &str,
        owner_agent: &str,
        workflow_id: Uuid,
        task_id: Uuid,
        duration_secs: u64,
    ) -> Result<ActiveLease, ResourceLockError> {
        let mut leases = self.leases.lock().await;
        let now = Utc::now();

        // Clean up expired lease if present
        if let Some(existing) = leases.get(resource_id) {
            if existing.expires_at > now && existing.owner_agent != owner_agent {
                return Err(ResourceLockError::ResourceBusy {
                    resource: resource_id.to_string(),
                    owner: existing.owner_agent.clone(),
                    until: existing.expires_at,
                });
            }
        }

        let fencing_token = self.fencing_seq.fetch_add(1, Ordering::SeqCst);
        let expires_at = now + Duration::seconds(duration_secs as i64);

        let lease = ActiveLease {
            resource_id: resource_id.to_string(),
            owner_agent: owner_agent.to_string(),
            workflow_id,
            task_id,
            acquired_at: now,
            expires_at,
            fencing_token,
        };

        leases.insert(resource_id.to_string(), lease.clone());
        Ok(lease)
    }

    /// Extends / renews an existing lease
    pub async fn renew_lease(
        &self,
        resource_id: &str,
        fencing_token: u64,
        additional_secs: u64,
    ) -> Result<ActiveLease, ResourceLockError> {
        let mut leases = self.leases.lock().await;
        if let Some(lease) = leases.get_mut(resource_id) {
            if lease.fencing_token != fencing_token {
                return Err(ResourceLockError::StaleFencingToken {
                    resource: resource_id.to_string(),
                    token: fencing_token,
                    current: lease.fencing_token,
                });
            }
            lease.expires_at = Utc::now() + Duration::seconds(additional_secs as i64);
            Ok(lease.clone())
        } else {
            Err(ResourceLockError::LockNotFound(resource_id.to_string()))
        }
    }

    /// Releases a resource lease if fencing token matches
    pub async fn release_lease(
        &self,
        resource_id: &str,
        fencing_token: u64,
    ) -> Result<(), ResourceLockError> {
        let mut leases = self.leases.lock().await;
        if let Some(existing) = leases.get(resource_id) {
            if existing.fencing_token != fencing_token {
                return Err(ResourceLockError::StaleFencingToken {
                    resource: resource_id.to_string(),
                    token: fencing_token,
                    current: existing.fencing_token,
                });
            }
            leases.remove(resource_id);
            Ok(())
        } else {
            // Already released or expired
            Ok(())
        }
    }

    /// Acquires a file mutation lock to prevent two coding agents editing the same file concurrently
    pub async fn lock_file<P: AsRef<Path>>(
        &self,
        path: P,
        owner_agent: &str,
        task_id: Uuid,
    ) -> Result<u64, ResourceLockError> {
        let mut file_locks = self.file_locks.lock().await;
        let p = path.as_ref().to_path_buf();

        if let Some((_existing_owner, existing_task, _)) = file_locks.get(&p) {
            if existing_task != &task_id {
                return Err(ResourceLockError::FileConflict(p));
            }
        }

        let fencing_token = self.fencing_seq.fetch_add(1, Ordering::SeqCst);
        file_locks.insert(p, (owner_agent.to_string(), task_id, fencing_token));
        Ok(fencing_token)
    }

    /// Releases a file mutation lock
    pub async fn unlock_file<P: AsRef<Path>>(&self, path: P) {
        let mut file_locks = self.file_locks.lock().await;
        file_locks.remove(path.as_ref());
    }

    /// Releases all locks belonging to a specific task (used on cancellation, completion, or crash)
    pub async fn release_all_for_task(&self, task_id: Uuid) {
        let mut leases = self.leases.lock().await;
        leases.retain(|_, v| v.task_id != task_id);

        let mut file_locks = self.file_locks.lock().await;
        file_locks.retain(|_, (_, tid, _)| *tid != task_id);
    }
}
