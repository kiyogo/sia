use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub url_or_path: String,
    pub title: String,
    pub snippet: String,
    pub published_date: Option<String>,
    pub score: f64,
}

#[async_trait]
pub trait SearchProviderAdapter: Send + Sync {
    fn provider_name(&self) -> &str;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String>;
}

/// In-memory/mock web search provider with deterministic responses for testing
pub struct MockWebSearchProvider {
    canned_results: Vec<SearchResult>,
}

impl MockWebSearchProvider {
    pub fn new() -> Self {
        Self {
            canned_results: vec![
                SearchResult {
                    url_or_path: "https://gartner.example.com/ai-market-2026".to_string(),
                    title: "Gartner Predicts Enterprise AI Market to Double by 2028".to_string(),
                    snippet: "Enterprise AI market adoption is expanding at 28% CAGR across Fortune 500 companies.".to_string(),
                    published_date: Some("2026-03-15".to_string()),
                    score: 0.96,
                },
                SearchResult {
                    url_or_path: "https://idc.example.com/market-spend".to_string(),
                    title: "IDC Worldwide AI Spending Guide".to_string(),
                    snippet: "Global spending on artificial intelligence software and infrastructure will exceed $300B.".to_string(),
                    published_date: Some("2026-02-20".to_string()),
                    score: 0.92,
                },
                SearchResult {
                    url_or_path: "https://techcrunch.example.com/autonomous-agents".to_string(),
                    title: "The Rise of Autonomous AI Workspaces".to_string(),
                    snippet: "Multi-agent systems with specialized workers and local privacy are outperforming single monolithic LLMs.".to_string(),
                    published_date: Some("2026-05-10".to_string()),
                    score: 0.88,
                },
            ],
        }
    }

    pub fn with_custom_results(results: Vec<SearchResult>) -> Self {
        Self { canned_results: results }
    }
}

#[async_trait]
impl SearchProviderAdapter for MockWebSearchProvider {
    fn provider_name(&self) -> &str {
        "MockWebSearchProvider"
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        let q_lower = query.to_lowercase();
        let terms: Vec<&str> = q_lower.split_whitespace().collect();

        let mut matched: Vec<SearchResult> = self
            .canned_results
            .iter()
            .filter(|r| {
                if terms.is_empty() {
                    return true;
                }
                let text = format!("{} {}", r.title, r.snippet).to_lowercase();
                terms.iter().any(|&term| text.contains(term))
            })
            .cloned()
            .collect();

        // If no term matched, fall back to returning top results
        if matched.is_empty() {
            matched = self.canned_results.clone();
        }

        matched.truncate(limit);
        Ok(matched)
    }
}

/// Local workspace file search provider scanning documents in a directory
pub struct LocalWorkspaceSearchProvider {
    workspace_root: PathBuf,
}

impl LocalWorkspaceSearchProvider {
    pub fn new(root: PathBuf) -> Self {
        Self { workspace_root: root }
    }
}

#[async_trait]
impl SearchProviderAdapter for LocalWorkspaceSearchProvider {
    fn provider_name(&self) -> &str {
        "LocalWorkspaceSearchProvider"
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        if !self.workspace_root.exists() {
            return Ok(Vec::new());
        }

        let q_lower = query.to_lowercase();
        let mut results = Vec::new();

        for entry in walkdir::WalkDir::new(&self.workspace_root)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ["md", "txt", "json", "csv"].contains(&ext_lower.as_str()) {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            if content.to_lowercase().contains(&q_lower) {
                                let snippet = content
                                    .lines()
                                    .find(|l| l.to_lowercase().contains(&q_lower))
                                    .unwrap_or_else(|| content.lines().next().unwrap_or(""))
                                    .to_string();

                                results.push(SearchResult {
                                    url_or_path: path.to_string_lossy().to_string(),
                                    title: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                                    snippet,
                                    published_date: None,
                                    score: 0.85,
                                });

                                if results.len() >= limit {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}
