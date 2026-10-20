use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub task_id: Option<Uuid>,
    pub artifact_type: String,
    pub name: String,
    pub hash: String,
    pub path_or_uri: String,
    pub summary: String,
    pub byte_size: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredArtifact {
    pub reference: ArtifactRef,
    pub full_content: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct SharedArtifactStore {
    artifacts: Arc<RwLock<HashMap<Uuid, StoredArtifact>>>,
}

impl SharedArtifactStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compute_hash(content: &str) -> String {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("hash:{:016x}", hasher.finish())
    }

    pub async fn store_artifact(
        &self,
        workflow_id: Uuid,
        task_id: Option<Uuid>,
        artifact_type: &str,
        name: &str,
        path_or_uri: &str,
        full_content: &str,
        metadata: serde_json::Value,
    ) -> ArtifactRef {
        let id = Uuid::new_v4();
        let hash = Self::compute_hash(full_content);
        let byte_size = full_content.len();

        let summary = if full_content.lines().count() > 5 {
            let first_3 = full_content.lines().take(3).collect::<Vec<_>>().join("\n");
            format!("{}\n... ({} total bytes)", first_3, byte_size)
        } else {
            full_content.to_string()
        };

        let reference = ArtifactRef {
            id,
            workflow_id,
            task_id,
            artifact_type: artifact_type.to_string(),
            name: name.to_string(),
            hash,
            path_or_uri: path_or_uri.to_string(),
            summary,
            byte_size,
            created_at: Utc::now(),
        };

        let stored = StoredArtifact {
            reference: reference.clone(),
            full_content: full_content.to_string(),
            metadata,
        };

        let mut map = self.artifacts.write().await;
        map.insert(id, stored);

        reference
    }

    pub async fn get_artifact(&self, id: &Uuid) -> Option<StoredArtifact> {
        let map = self.artifacts.read().await;
        map.get(id).cloned()
    }

    pub async fn get_reference(&self, id: &Uuid) -> Option<ArtifactRef> {
        let map = self.artifacts.read().await;
        map.get(id).map(|s| s.reference.clone())
    }

    pub async fn list_for_workflow(&self, workflow_id: &Uuid) -> Vec<ArtifactRef> {
        let map = self.artifacts.read().await;
        let mut list: Vec<_> = map
            .values()
            .filter(|a| &a.reference.workflow_id == workflow_id)
            .map(|a| a.reference.clone())
            .collect();
        list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        list
    }

    pub async fn list_for_task(&self, task_id: &Uuid) -> Vec<ArtifactRef> {
        let map = self.artifacts.read().await;
        let mut list: Vec<_> = map
            .values()
            .filter(|a| a.reference.task_id.as_ref() == Some(task_id))
            .map(|a| a.reference.clone())
            .collect();
        list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        list
    }
}
