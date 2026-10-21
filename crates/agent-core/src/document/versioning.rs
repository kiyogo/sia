use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub original_path: PathBuf,
    pub backup_path: PathBuf,
    pub timestamp: chrono::DateTime<Utc>,
    pub file_size_bytes: u64,
}

pub struct DocumentVersionManager;

impl DocumentVersionManager {
    /// Creates a non-destructive backup of the file before any modification
    pub fn backup_before_edit<P: AsRef<Path>>(file_path: P) -> Result<DocumentVersion, String> {
        let path = file_path.as_ref();
        if !path.exists() {
            return Err("Cannot backup non-existent file".to_string());
        }

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let versions_dir = parent.join(".doc_versions");
        std::fs::create_dir_all(&versions_dir)
            .map_err(|e| format!("Failed to create versions directory: {}", e))?;

        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("doc");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("bin");
        let timestamp_str = Utc::now().format("%Y%m%d_%H%M%S_%3f").to_string();
        let backup_name = format!("{}_{}.{}", file_stem, timestamp_str, ext);
        let backup_path = versions_dir.join(backup_name);

        std::fs::copy(path, &backup_path)
            .map_err(|e| format!("Failed to copy file for backup: {}", e))?;

        let metadata = std::fs::metadata(&backup_path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;

        Ok(DocumentVersion {
            original_path: path.to_path_buf(),
            backup_path,
            timestamp: Utc::now(),
            file_size_bytes: metadata.len(),
        })
    }
}
