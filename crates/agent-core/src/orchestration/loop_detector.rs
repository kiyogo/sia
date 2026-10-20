use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct LoopDetector {
    call_counts: HashMap<String, usize>,
    max_identical_calls: usize,
    repeated_failures: HashMap<String, usize>,
    max_identical_failures: usize,
}

impl LoopDetector {
    pub fn new(max_identical_calls: usize, max_identical_failures: usize) -> Self {
        Self {
            call_counts: HashMap::new(),
            max_identical_calls,
            repeated_failures: HashMap::new(),
            max_identical_failures,
        }
    }

    /// Records a tool invocation fingerprint and checks if loop threshold exceeded
    pub fn record_tool_call(&mut self, tool_name: &str, params_json: &str) -> bool {
        let fingerprint = format!("{}:{}", tool_name, params_json);
        let count = self.call_counts.entry(fingerprint).or_insert(0);
        *count += 1;
        *count >= self.max_identical_calls
    }

    /// Records a failure on a specific task/plan signature
    pub fn record_failure(&mut self, signature: &str) -> bool {
        let count = self.repeated_failures.entry(signature.to_string()).or_insert(0);
        *count += 1;
        *count >= self.max_identical_failures
    }

    pub fn reset(&mut self) {
        self.call_counts.clear();
        self.repeated_failures.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_detection() {
        let mut detector = LoopDetector::new(3, 2);
        assert!(!detector.record_tool_call("code_edit", "{\"file\":\"test.rs\"}"));
        assert!(!detector.record_tool_call("code_edit", "{\"file\":\"test.rs\"}"));
        assert!(detector.record_tool_call("code_edit", "{\"file\":\"test.rs\"}"));
    }
}
