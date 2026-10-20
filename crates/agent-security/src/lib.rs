use agent_platform::FileSystemProvider;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Action blocked by security policy: {0}")]
    PolicyBlocked(String),

    #[error("Action outside allowed workspace boundary: {0:?}")]
    OutsideWorkspace(PathBuf),

    #[error("Action requires human approval (Approval ID: {0})")]
    ApprovalRequired(Uuid),

    #[error("Action was rejected by user")]
    ApprovalRejected,

    #[error("Emergency stop is active")]
    EmergencyStopActive,
}

pub type SecurityResult<T> = Result<T, SecurityError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionTier {
    ReadOnly,
    WorkspaceWrite,
    ShellExecution,
    DesktopControl,
    DestructiveAction,
    ExternalSideEffect,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionMode {
    Strict,        // Needs approval for everything except ReadOnly
    WorkspaceAuto, // Auto allows ReadOnly and WorkspaceWrite inside workspace
    FullAuto,      // Auto allows ReadOnly, WorkspaceWrite, safe ShellExecution
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub tier: PermissionTier,
    pub description: String,
    pub details: serde_json::Value,
    pub status: ApprovalStatus,
    pub requested_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

pub struct SecurityEngine {
    workspace_root: RwLock<PathBuf>,
    mode: RwLock<PermissionMode>,
    emergency_stop: AtomicBool,
    pending_approvals: RwLock<HashMap<Uuid, ApprovalRequest>>,
    fs_provider: Box<dyn FileSystemProvider>,
}

impl SecurityEngine {
    pub fn new(workspace_root: PathBuf, mode: PermissionMode, fs_provider: Box<dyn FileSystemProvider>) -> Self {
        Self {
            workspace_root: RwLock::new(workspace_root),
            mode: RwLock::new(mode),
            emergency_stop: AtomicBool::new(false),
            pending_approvals: RwLock::new(HashMap::new()),
            fs_provider,
        }
    }

    pub fn trigger_emergency_stop(&self) {
        self.emergency_stop.store(true, Ordering::SeqCst);
    }

    pub fn reset_emergency_stop(&self) {
        self.emergency_stop.store(false, Ordering::SeqCst);
    }

    pub fn is_emergency_stop(&self) -> bool {
        self.emergency_stop.load(Ordering::SeqCst)
    }

    pub async fn get_mode(&self) -> PermissionMode {
        *self.mode.read().await
    }

    pub async fn set_mode(&self, mode: PermissionMode) {
        let mut m = self.mode.write().await;
        *m = mode;
    }

    pub async fn get_workspace_root(&self) -> PathBuf {
        self.workspace_root.read().await.clone()
    }

    pub async fn set_workspace_root(&self, root: PathBuf) {
        let mut w = self.workspace_root.write().await;
        *w = root;
    }

    pub async fn check_permission(
        &self,
        tier: PermissionTier,
        description: &str,
        path: Option<&Path>,
        details: serde_json::Value,
    ) -> SecurityResult<()> {
        if self.is_emergency_stop() {
            return Err(SecurityError::EmergencyStopActive);
        }

        // Validate boundary if path is provided
        if let Some(p) = path {
            let root = self.workspace_root.read().await;
            if !self.fs_provider.is_within_boundary(&root, p) {
                // Modifying outside workspace is considered DestructiveAction or requires Approval
                return self
                    .request_approval(
                        PermissionTier::DestructiveAction,
                        &format!("Access outside workspace: {}", p.display()),
                        details,
                    )
                    .await;
            }
        }

        let mode = *self.mode.read().await;

        match (mode, tier) {
            (_, PermissionTier::ReadOnly) => Ok(()),
            (PermissionMode::FullAuto, PermissionTier::WorkspaceWrite | PermissionTier::ShellExecution) => {
                // If shell execution is destructive, enforce approval
                if tier == PermissionTier::ShellExecution && Self::is_dangerous_command(&description) {
                    return self.request_approval(tier, description, details).await;
                }
                Ok(())
            }
            (PermissionMode::WorkspaceAuto, PermissionTier::WorkspaceWrite) => Ok(()),
            _ => self.request_approval(tier, description, details).await,
        }
    }

    fn is_dangerous_command(cmd: &str) -> bool {
        let lower = cmd.to_lowercase();
        let dangerous_keywords = [
            "rmdir /s",
            "del /f /s",
            "format ",
            "shutdown",
            "remove-item -recurse -force",
            "reg delete",
            "drop database",
            "mkfs",
            "dd if=",
            ":(){ :|:& };:",
        ];
        dangerous_keywords.iter().any(|k| lower.contains(k))
    }

    async fn request_approval(
        &self,
        tier: PermissionTier,
        description: &str,
        details: serde_json::Value,
    ) -> SecurityResult<()> {
        let id = Uuid::new_v4();
        let req = ApprovalRequest {
            id,
            tier,
            description: description.to_string(),
            details,
            status: ApprovalStatus::Pending,
            requested_at: Utc::now(),
            resolved_at: None,
        };

        let mut map = self.pending_approvals.write().await;
        map.insert(id, req);

        Err(SecurityError::ApprovalRequired(id))
    }

    pub async fn resolve_approval(&self, id: Uuid, approved: bool) -> Option<ApprovalRequest> {
        let mut map = self.pending_approvals.write().await;
        if let Some(req) = map.get_mut(&id) {
            req.status = if approved {
                ApprovalStatus::Approved
            } else {
                ApprovalStatus::Rejected
            };
            req.resolved_at = Some(Utc::now());
            Some(req.clone())
        } else {
            None
        }
    }

    pub async fn list_pending_approvals(&self) -> Vec<ApprovalRequest> {
        let map = self.pending_approvals.read().await;
        map.values()
            .filter(|r| r.status == ApprovalStatus::Pending)
            .cloned()
            .collect()
    }
}

pub type SharedSecurityEngine = Arc<SecurityEngine>;

#[cfg(test)]
mod tests {
    use super::*;
    use agent_platform::WindowsFileSystemProvider;

    #[tokio::test]
    async fn test_security_tiers_and_approval() {
        let root = PathBuf::from(r"C:\workspace\myproject");
        let fs = Box::new(WindowsFileSystemProvider::new());
        let sec = SecurityEngine::new(root.clone(), PermissionMode::Strict, fs);

        // ReadOnly passes in Strict
        assert!(sec
            .check_permission(PermissionTier::ReadOnly, "read file", None, serde_json::json!({}))
            .await
            .is_ok());

        // WorkspaceWrite requires approval in Strict
        let err = sec
            .check_permission(
                PermissionTier::WorkspaceWrite,
                "write file",
                Some(&root.join("src/main.rs")),
                serde_json::json!({}),
            )
            .await;
        assert!(matches!(err, Err(SecurityError::ApprovalRequired(_))));

        let pending = sec.list_pending_approvals().await;
        assert_eq!(pending.len(), 1);
        let id = pending[0].id;

        // Resolve approval
        sec.resolve_approval(id, true).await;
        let pending_after = sec.list_pending_approvals().await;
        assert_eq!(pending_after.len(), 0);
    }

    #[tokio::test]
    async fn test_emergency_stop() {
        let root = PathBuf::from(r"C:\workspace\myproject");
        let fs = Box::new(WindowsFileSystemProvider::new());
        let sec = SecurityEngine::new(root, PermissionMode::FullAuto, fs);

        assert!(!sec.is_emergency_stop());
        sec.trigger_emergency_stop();
        assert!(sec.is_emergency_stop());

        let err = sec
            .check_permission(PermissionTier::ReadOnly, "read", None, serde_json::json!({}))
            .await;
        assert!(matches!(err, Err(SecurityError::EmergencyStopActive)));
    }
}
