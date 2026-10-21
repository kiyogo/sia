use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use agent_storage::{ResearchSource, SourceType, VerificationStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFilter {
    pub source_type: Option<SourceType>,
    pub verification_status: Option<VerificationStatus>,
    pub query: Option<String>,
}

#[derive(Default)]
pub struct ResearchSourceRegistry {
    sources: HashMap<Uuid, ResearchSource>,
    url_to_id: HashMap<String, Uuid>,
    hash_to_id: HashMap<String, Uuid>,
}

impl ResearchSourceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_source(
        &mut self,
        url_or_path: String,
        title: String,
        source_type: SourceType,
        published_date: Option<String>,
        raw_content: &str,
    ) -> (Uuid, bool) {
        let content_hash = format!("{:x}", md5_or_simple_hash(raw_content));
        let canonical_url = url_or_path.trim().to_lowercase();

        // Deduplication by URL
        if let Some(&existing_id) = self.url_to_id.get(&canonical_url) {
            return (existing_id, false);
        }

        // Deduplication by content hash
        if let Some(&existing_id) = self.hash_to_id.get(&content_hash) {
            return (existing_id, false);
        }

        let id = Uuid::new_v4();
        let snippet = if raw_content.len() > 300 {
            Some(raw_content[..300].to_string())
        } else if !raw_content.is_empty() {
            Some(raw_content.to_string())
        } else {
            None
        };

        let source = ResearchSource {
            id,
            url_or_path: url_or_path.clone(),
            title,
            source_type,
            retrieved_at: Utc::now(),
            published_date,
            content_hash: content_hash.clone(),
            verification_status: VerificationStatus::Verified,
            raw_snippet: snippet,
        };

        self.sources.insert(id, source);
        self.url_to_id.insert(canonical_url, id);
        self.hash_to_id.insert(content_hash, id);

        (id, true)
    }

    pub fn get_source(&self, id: Uuid) -> Option<&ResearchSource> {
        self.sources.get(&id)
    }

    pub fn get_by_url(&self, url: &str) -> Option<&ResearchSource> {
        let canonical = url.trim().to_lowercase();
        self.url_to_id.get(&canonical).and_then(|id| self.sources.get(id))
    }

    pub fn mark_status(&mut self, id: Uuid, status: VerificationStatus) -> bool {
        if let Some(source) = self.sources.get_mut(&id) {
            source.verification_status = status;
            true
        } else {
            false
        }
    }

    pub fn list_sources(&self, filter: Option<&SourceFilter>) -> Vec<&ResearchSource> {
        self.sources
            .values()
            .filter(|s| {
                if let Some(f) = filter {
                    if let Some(ref st) = f.source_type {
                        if &s.source_type != st {
                            return false;
                        }
                    }
                    if let Some(ref vs) = f.verification_status {
                        if &s.verification_status != vs {
                            return false;
                        }
                    }
                    if let Some(ref q) = f.query {
                        let query_lower = q.to_lowercase();
                        if !s.title.to_lowercase().contains(&query_lower)
                            && !s.url_or_path.to_lowercase().contains(&query_lower)
                        {
                            return false;
                        }
                    }
                }
                true
            })
            .collect()
    }

    pub fn total_count(&self) -> usize {
        self.sources.len()
    }
}

fn md5_or_simple_hash(data: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}
