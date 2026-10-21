use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use agent_storage::{EvidenceItem, EvidenceType, VerificationStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionReport {
    pub entity_or_metric: String,
    pub evidence_a: EvidenceItem,
    pub evidence_b: EvidenceItem,
    pub difference_summary: String,
}

#[derive(Default)]
pub struct EvidenceStore {
    items: HashMap<Uuid, EvidenceItem>,
    source_to_evidence: HashMap<Uuid, Vec<Uuid>>,
}

impl EvidenceStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_evidence(
        &mut self,
        source_id: Uuid,
        task_id: Option<Uuid>,
        claim_text: String,
        evidence_type: EvidenceType,
        page_or_section: Option<String>,
        confidence_score: f64,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let item = EvidenceItem {
            id,
            source_id,
            task_id,
            claim_text,
            evidence_type,
            page_or_section,
            confidence_score,
            verification_status: VerificationStatus::Verified,
            created_at: Utc::now(),
        };

        self.items.insert(id, item);
        self.source_to_evidence.entry(source_id).or_default().push(id);
        id
    }

    pub fn get_evidence(&self, id: Uuid) -> Option<&EvidenceItem> {
        self.items.get(&id)
    }

    pub fn get_by_source(&self, source_id: Uuid) -> Vec<&EvidenceItem> {
        if let Some(ids) = self.source_to_evidence.get(&source_id) {
            ids.iter().filter_map(|id| self.items.get(id)).collect()
        } else {
            Vec::new()
        }
    }

    pub fn list_all(&self) -> Vec<&EvidenceItem> {
        self.items.values().collect()
    }

    /// Detect potential contradictions between evidence claims.
    /// E.g., multiple sources reporting divergent figures or conflicting assertions.
    pub fn detect_contradictions(&self) -> Vec<ContradictionReport> {
        let all_items: Vec<&EvidenceItem> = self.items.values().collect();
        let mut reports = Vec::new();

        for i in 0..all_items.len() {
            for j in (i + 1)..all_items.len() {
                let a = all_items[i];
                let b = all_items[j];

                // If from different sources, check for semantic contradiction or numerical variance
                if a.source_id != b.source_id {
                    if let Some(diff) = check_claim_conflict(&a.claim_text, &b.claim_text) {
                        reports.push(ContradictionReport {
                            entity_or_metric: diff.0,
                            evidence_a: a.clone(),
                            evidence_b: b.clone(),
                            difference_summary: diff.1,
                        });
                    }
                }
            }
        }

        reports
    }
}

/// Heuristic conflict detector for numerical claims or direct contradictory keywords
fn check_claim_conflict(claim_a: &str, claim_b: &str) -> Option<(String, String)> {
    let a_lower = claim_a.to_lowercase();
    let b_lower = claim_b.to_lowercase();

    // Check shared key nouns (e.g., "market size", "revenue", "growth", "growth rate", "sales")
    let key_metrics = ["market", "revenue", "growth", "sales", "price", "valuation", "share"];
    for metric in &key_metrics {
        if a_lower.contains(metric) && b_lower.contains(metric) {
            // Extract numbers from both
            let nums_a = extract_numbers(&a_lower);
            let nums_b = extract_numbers(&b_lower);

            if !nums_a.is_empty() && !nums_b.is_empty() {
                // If the primary numerical values differ significantly (> 15% variance)
                let val_a = nums_a[0];
                let val_b = nums_b[0];
                let diff = (val_a - val_b).abs();
                let max_val = val_a.max(val_b);
                if max_val > 0.0 && (diff / max_val) > 0.15 {
                    return Some((
                        metric.to_string(),
                        format!(
                            "Numerical discrepancy in {} metrics: Claim A states {}, but Claim B states {}",
                            metric, val_a, val_b
                        ),
                    ));
                }
            } else if (a_lower.contains("increased") && b_lower.contains("decreased"))
                || (a_lower.contains("positive") && b_lower.contains("negative"))
                || (a_lower.contains("profitable") && b_lower.contains("loss"))
            {
                return Some((
                    metric.to_string(),
                    format!("Direct semantic conflict regarding {}: opposite directional claims", metric),
                ));
            }
        }
    }

    None
}

fn extract_numbers(s: &str) -> Vec<f64> {
    let mut numbers = Vec::new();
    let mut current = String::new();
    let mut in_num = false;

    for c in s.chars() {
        if c.is_ascii_digit() || (c == '.' && in_num && !current.contains('.')) {
            current.push(c);
            in_num = true;
        } else {
            if in_num {
                if let Ok(num) = current.parse::<f64>() {
                    numbers.push(num);
                }
                current.clear();
                in_num = false;
            }
        }
    }
    if in_num {
        if let Ok(num) = current.parse::<f64>() {
            numbers.push(num);
        }
    }

    numbers
}
