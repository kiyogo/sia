use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProvenanceRecord {
    pub id: Uuid,
    pub dataset_id: String,
    pub source_path: String,
    pub sheet_name: Option<String>,
    pub columns_used: Vec<String>,
    pub transformations_applied: Vec<String>,
    pub calculation_method: String,
    pub execution_timestamp: DateTime<Utc>,
    pub result_summary: String,
    pub verification_hash: String,
}

impl DataProvenanceRecord {
    pub fn new(
        dataset_id: &str,
        source_path: &str,
        sheet_name: Option<&str>,
        columns_used: Vec<String>,
        transformations: Vec<String>,
        calculation_method: &str,
        result_summary: &str,
    ) -> Self {
        let hash_input = format!(
            "{}:{}:{}:{}",
            dataset_id,
            source_path,
            calculation_method,
            columns_used.join(",")
        );
        let verification_hash = format!("{:x}", simple_hash(&hash_input));

        Self {
            id: Uuid::new_v4(),
            dataset_id: dataset_id.to_string(),
            source_path: source_path.to_string(),
            sheet_name: sheet_name.map(|s| s.to_string()),
            columns_used,
            transformations_applied: transformations,
            calculation_method: calculation_method.to_string(),
            execution_timestamp: Utc::now(),
            result_summary: result_summary.to_string(),
            verification_hash,
        }
    }
}

fn simple_hash(data: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}
