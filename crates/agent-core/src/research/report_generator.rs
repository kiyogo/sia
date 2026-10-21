use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use agent_storage::{EvidenceItem, ResearchSource};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    pub title: String,
    pub executive_summary: String,
    pub key_findings: Vec<FindingSection>,
    pub contradictions_noted: Vec<String>,
    pub sources_cited: Vec<CitedSource>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingSection {
    pub heading: String,
    pub content: String,
    pub evidence_refs: Vec<usize>, // 1-based index into sources_cited
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitedSource {
    pub citation_index: usize,
    pub source_id: Uuid,
    pub title: String,
    pub url_or_path: String,
    pub published_date: Option<String>,
    pub verification_status: String,
}

pub struct ResearchReportGenerator;

impl ResearchReportGenerator {
    pub fn build_report(
        title: &str,
        executive_summary: &str,
        sections: Vec<(&str, &str, Vec<&EvidenceItem>)>,
        sources_map: &HashMap<Uuid, &ResearchSource>,
        contradictions: &[String],
    ) -> ResearchReport {
        let mut cited_sources: Vec<CitedSource> = Vec::new();
        let mut source_id_to_citation_idx: HashMap<Uuid, usize> = HashMap::new();

        let mut finding_sections = Vec::new();

        for (heading, body, evidence_items) in sections {
            let mut citation_indices = Vec::new();

            for item in evidence_items {
                let citation_idx = *source_id_to_citation_idx.entry(item.source_id).or_insert_with(|| {
                    let next_idx = cited_sources.len() + 1;
                    if let Some(src) = sources_map.get(&item.source_id) {
                        cited_sources.push(CitedSource {
                            citation_index: next_idx,
                            source_id: src.id,
                            title: src.title.clone(),
                            url_or_path: src.url_or_path.clone(),
                            published_date: src.published_date.clone(),
                            verification_status: src.verification_status.to_string(),
                        });
                    } else {
                        cited_sources.push(CitedSource {
                            citation_index: next_idx,
                            source_id: item.source_id,
                            title: format!("Source {}", item.source_id),
                            url_or_path: "unknown".to_string(),
                            published_date: None,
                            verification_status: "Unverified".to_string(),
                        });
                    }
                    next_idx
                });

                if !citation_indices.contains(&citation_idx) {
                    citation_indices.push(citation_idx);
                }
            }

            finding_sections.push(FindingSection {
                heading: heading.to_string(),
                content: body.to_string(),
                evidence_refs: citation_indices,
            });
        }

        ResearchReport {
            title: title.to_string(),
            executive_summary: executive_summary.to_string(),
            key_findings: finding_sections,
            contradictions_noted: contradictions.to_vec(),
            sources_cited: cited_sources,
            generated_at: chrono::Utc::now(),
        }
    }

    pub fn render_markdown(report: &ResearchReport) -> String {
        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", report.title));
        md.push_str(&format!("*Generated: {}*\n\n", report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        md.push_str("## Executive Summary\n\n");
        md.push_str(&report.executive_summary);
        md.push_str("\n\n");

        md.push_str("## Key Findings\n\n");
        for section in &report.key_findings {
            md.push_str(&format!("### {}\n\n", section.heading));
            md.push_str(&section.content);

            if !section.evidence_refs.is_empty() {
                let refs_str: Vec<String> = section.evidence_refs.iter().map(|idx| format!("[^{}]", idx)).collect();
                md.push_str(&format!(" {}\n\n", refs_str.join(" ")));
            } else {
                md.push_str("\n\n");
            }
        }

        if !report.contradictions_noted.is_empty() {
            md.push_str("## Source Contradictions & Discrepancies\n\n");
            for c in &report.contradictions_noted {
                md.push_str(&format!("- ⚠️ {}\n", c));
            }
            md.push_str("\n");
        }

        md.push_str("## References & Sources Cited\n\n");
        for src in &report.sources_cited {
            let pub_date = src.published_date.as_deref().unwrap_or("n.d.");
            md.push_str(&format!(
                "[^{}]: **{}** ({}) - [{}]({}). Status: `{}`\n",
                src.citation_index, src.title, pub_date, src.url_or_path, src.url_or_path, src.verification_status
            ));
        }

        md
    }
}
