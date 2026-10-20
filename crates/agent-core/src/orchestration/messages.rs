use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::agent_registry::AgentType;
use super::artifact_store::ArtifactRef;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    TaskAssigned,
    TaskStarted,
    ProgressUpdated,
    ArtifactProduced,
    TaskCompleted,
    TaskFailed,
    ApprovalRequired,
    ResourceBlocked,
    TaskCancelled,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::TaskAssigned => write!(f, "TaskAssigned"),
            MessageType::TaskStarted => write!(f, "TaskStarted"),
            MessageType::ProgressUpdated => write!(f, "ProgressUpdated"),
            MessageType::ArtifactProduced => write!(f, "ArtifactProduced"),
            MessageType::TaskCompleted => write!(f, "TaskCompleted"),
            MessageType::TaskFailed => write!(f, "TaskFailed"),
            MessageType::ApprovalRequired => write!(f, "ApprovalRequired"),
            MessageType::ResourceBlocked => write!(f, "ResourceBlocked"),
            MessageType::TaskCancelled => write!(f, "TaskCancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub task_id: Option<Uuid>,
    pub source_agent: AgentType,
    pub target_agent: AgentType,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
    pub artifact_refs: Vec<ArtifactRef>,
    pub timestamp: DateTime<Utc>,
}

impl AgentMessage {
    pub fn new(
        workflow_id: Uuid,
        task_id: Option<Uuid>,
        source_agent: AgentType,
        target_agent: AgentType,
        message_type: MessageType,
        payload: serde_json::Value,
        artifact_refs: Vec<ArtifactRef>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            workflow_id,
            task_id,
            source_agent,
            target_agent,
            message_type,
            payload,
            artifact_refs,
            timestamp: Utc::now(),
        }
    }
}
