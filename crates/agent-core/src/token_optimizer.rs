use crate::model::{ChatMessage, ModelUsage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBudgetConfig {
    pub max_context_tokens: u32,
    pub system_budget: u32,
    pub history_budget: u32,
    pub tool_result_budget: u32,
    pub completion_reserve: u32,
    pub max_tool_output_lines: usize,
    pub max_tool_output_bytes: usize,
}

impl Default for ContextBudgetConfig {
    fn default() -> Self {
        Self {
            max_context_tokens: 32_000,
            system_budget: 2_500,
            history_budget: 18_000,
            tool_result_budget: 7_500,
            completion_reserve: 4_000,
            max_tool_output_lines: 150,
            max_tool_output_bytes: 16_384,
        }
    }
}

pub struct TokenOptimizer {
    config: ContextBudgetConfig,
}

impl TokenOptimizer {
    pub fn new(config: ContextBudgetConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &ContextBudgetConfig {
        &self.config
    }

    /// Fast token estimator: average 4 chars per token for English/code, with 1.3 multiplier for Unicode/Vietnamese
    pub fn estimate_tokens(text: &str) -> u32 {
        let char_count = text.chars().count();
        let ascii_count = text.chars().filter(|c| c.is_ascii()).count();
        let non_ascii_count = char_count - ascii_count;

        let estimated = (ascii_count as f64 / 4.0) + (non_ascii_count as f64 / 1.5);
        (estimated.ceil() as u32).max(1)
    }

    /// Truncate large tool output to preserve token budget
    pub fn truncate_tool_output(&self, raw: &str) -> (String, bool) {
        let lines: Vec<&str> = raw.lines().collect();
        let bytes = raw.as_bytes();

        let line_limit = self.config.max_tool_output_lines;
        let byte_limit = self.config.max_tool_output_bytes;

        if lines.len() <= line_limit && bytes.len() <= byte_limit {
            return (raw.to_string(), false);
        }

        let mut truncated_text = String::new();
        let mut current_bytes = 0;
        let mut included_lines = 0;

        // Keep head lines
        let head_count = line_limit.min(lines.len());
        for line in &lines[..head_count] {
            if current_bytes + line.len() + 1 > byte_limit {
                break;
            }
            truncated_text.push_str(line);
            truncated_text.push('\n');
            current_bytes += line.len() + 1;
            included_lines += 1;
        }

        let omitted_lines = lines.len().saturating_sub(included_lines);
        let omitted_bytes = bytes.len().saturating_sub(current_bytes);

        truncated_text.push_str(&format!(
            "\n[... Output truncated: {} lines ({} bytes) omitted to preserve token budget. Use offset/limit or specific queries to inspect further ...]",
            omitted_lines, omitted_bytes
        ));

        (truncated_text, true)
    }

    /// Compact conversation messages if they exceed history budget
    pub fn compact_history(&self, messages: &[ChatMessage]) -> Vec<ChatMessage> {
        let total_tokens: u32 = messages
            .iter()
            .map(|m| Self::estimate_tokens(m.content.as_deref().unwrap_or("")))
            .sum();

        if total_tokens <= self.config.history_budget || messages.len() <= 4 {
            return messages.to_vec();
        }

        // Keep the first message (usually system/initial user prompt) and the most recent 4 messages
        let mut compacted = Vec::new();
        compacted.push(messages[0].clone());

        let middle_end = messages.len().saturating_sub(4);
        let middle_messages = &messages[1..middle_end];

        let mut summary = String::from("[Compacted Historical Progress Summary]:\n");
        for m in middle_messages {
            let role = &m.role;
            let preview = m
                .content
                .as_deref()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("");
            if !preview.is_empty() {
                summary.push_str(&format!("• [{role}]: {}\n", preview));
            }
            if let Some(tcs) = &m.tool_calls {
                for tc in tcs {
                    summary.push_str(&format!("  -> Called tool {}\n", tc.function.name));
                }
            }
        }

        compacted.push(ChatMessage {
            role: "system".to_string(),
            content: Some(summary),
            tool_calls: None,
            tool_call_id: None,
        });

        // Add latest turns
        for m in &messages[middle_end..] {
            compacted.push(m.clone());
        }

        compacted
    }

    /// Calculate estimated cost in USD based on standard rates
    pub fn estimate_cost_usd(usage: &ModelUsage, price_per_m_in: f64, price_per_m_out: f64) -> f64 {
        let in_cost = (usage.prompt_tokens as f64 / 1_000_000.0) * price_per_m_in;
        let out_cost = (usage.completion_tokens as f64 / 1_000_000.0) * price_per_m_out;
        in_cost + out_cost
    }

    /// Prune DOM snapshot tree: strip non-interactive, empty container nodes and prune past max_depth
    pub fn prune_dom_tree(node: &agent_platform::DomSnapshotNode, max_depth: usize) -> Option<agent_platform::DomSnapshotNode> {
        if max_depth == 0 {
            return None;
        }

        let mut pruned_children = Vec::new();
        for child in &node.children {
            if let Some(pruned_child) = Self::prune_dom_tree(child, max_depth - 1) {
                pruned_children.push(pruned_child);
            }
        }

        let has_text = node.text.as_ref().map(|t| !t.trim().is_empty()).unwrap_or(false);
        let has_value = node.value.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false);

        if node.is_interactive || has_text || has_value || !pruned_children.is_empty() {
            Some(agent_platform::DomSnapshotNode {
                ref_id: node.ref_id.clone(),
                tag: node.tag.clone(),
                role: node.role.clone(),
                text: node.text.clone(),
                value: node.value.clone(),
                is_interactive: node.is_interactive,
                attributes: node.attributes.clone(),
                children: pruned_children,
            })
        } else {
            None
        }
    }

    /// Convert structured DOM snapshot into compact, line-based textual representation
    pub fn dom_to_compact_text(node: &agent_platform::DomSnapshotNode, indent: usize) -> String {
        let mut out = String::new();
        let indent_str = "  ".repeat(indent);

        let role_str = node.role.as_deref().unwrap_or(&node.tag);
        let mut line = format!("{}[{}] {}", indent_str, node.ref_id, role_str);

        if let Some(ref text) = node.text {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                let snippet = if trimmed.len() > 60 {
                    format!("{}...", &trimmed[..60])
                } else {
                    trimmed.to_string()
                };
                line.push_str(&format!(" \"{}\"", snippet));
            }
        }

        if let Some(ref val) = node.value {
            if !val.is_empty() {
                line.push_str(&format!(" value=\"{}\"", val));
            }
        }

        if node.is_interactive {
            line.push_str(" [interactive]");
        }

        out.push_str(&line);
        out.push('\n');

        for child in &node.children {
            out.push_str(&Self::dom_to_compact_text(child, indent + 1));
        }

        out
    }

    /// Calculate delta between previous DOM snapshot and current DOM snapshot
    pub fn compute_dom_delta(
        prev: &agent_platform::DomSnapshotNode,
        curr: &agent_platform::DomSnapshotNode,
    ) -> String {
        let prev_text = Self::dom_to_compact_text(prev, 0);
        let curr_text = Self::dom_to_compact_text(curr, 0);

        if prev_text == curr_text {
            return "[Observation Delta: No changes detected in DOM tree]".to_string();
        }

        let prev_lines: std::collections::HashSet<&str> = prev_text.lines().collect();
        let curr_lines: Vec<&str> = curr_text.lines().collect();

        let mut added = Vec::new();
        for line in curr_lines {
            if !prev_lines.contains(line) {
                added.push(line);
            }
        }

        if added.is_empty() {
            "[Observation Delta: Structural node modifications detected]".to_string()
        } else {
            format!("[Observation Delta: {} changed/added nodes]:\n{}", added.len(), added.join("\n"))
        }
    }

    /// Observation state hash for caching
    pub fn hash_state(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimation() {
        let text = "Hello world from Rust token estimator!";
        let tokens = TokenOptimizer::estimate_tokens(text);
        assert!(tokens > 5 && tokens < 20);

        let vn_text = "Xin chào thế giới từ trợ lý AI!";
        let vn_tokens = TokenOptimizer::estimate_tokens(vn_text);
        assert!(vn_tokens > 5);
    }

    #[test]
    fn test_tool_output_truncation() {
        let optimizer = TokenOptimizer::new(ContextBudgetConfig {
            max_tool_output_lines: 5,
            max_tool_output_bytes: 200,
            ..Default::default()
        });

        let big_output = (1..=20)
            .map(|i| format!("Log entry line number {}", i))
            .collect::<Vec<_>>()
            .join("\n");

        let (truncated, was_truncated) = optimizer.truncate_tool_output(&big_output);
        assert!(was_truncated);
        assert!(truncated.contains("Output truncated"));
    }

    #[test]
    fn test_history_compaction() {
        let optimizer = TokenOptimizer::new(ContextBudgetConfig {
            history_budget: 30, // very small budget
            ..Default::default()
        });

        let mut messages = Vec::new();
        for i in 0..10 {
            messages.push(ChatMessage {
                role: if i % 2 == 0 { "user".to_string() } else { "assistant".to_string() },
                content: Some(format!("Step {} involves writing tests and building code with various explanations", i)),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        let compacted = optimizer.compact_history(&messages);
        assert!(compacted.len() < messages.len());
        assert!(compacted.iter().any(|m| m.content.as_deref().unwrap_or("").contains("Compacted Historical Progress Summary")));
    }
}
