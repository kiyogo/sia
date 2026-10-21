use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataClassificationLevel {
    Public = 1,
    Internal = 2,
    Confidential = 3,
    Restricted = 4,
}

impl std::fmt::Display for DataClassificationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataClassificationLevel::Public => write!(f, "Public"),
            DataClassificationLevel::Internal => write!(f, "Internal"),
            DataClassificationLevel::Confidential => write!(f, "Confidential"),
            DataClassificationLevel::Restricted => write!(f, "Restricted"),
        }
    }
}

pub struct SecretRedactor {
    patterns: Vec<Regex>,
}

impl Default for SecretRedactor {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretRedactor {
    pub fn new() -> Self {
        let regexes = vec![
            // OpenAI / Anthropic / Generic API keys
            Regex::new(r"sk-[a-zA-Z0-9_-]{20,}").unwrap(),
            // Bearer Tokens
            Regex::new(r"(?i)bearer\s+[a-zA-Z0-9_\-\.]{20,}").unwrap(),
            // GitHub personal access tokens
            Regex::new(r"gh[pousr]_[a-zA-Z0-9]{36,}").unwrap(),
            // AWS Access Key ID
            Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
            // Generic password fields: password = "..." or password: "..."
            Regex::new(r#"(?i)(password|secret|api_key|token)\s*[:=]\s*["']([^"']{6,})["']"#).unwrap(),
        ];

        Self { patterns: regexes }
    }

    /// Redacts known secrets and tokens from text, replacing them with [REDACTED_SECRET]
    pub fn redact(&self, input: &str) -> String {
        let mut result = input.to_string();
        for pattern in &self.patterns {
            result = pattern.replace_all(&result, "[REDACTED_SECRET]").to_string();
        }
        result
    }

    /// Checks if a string contains unmasked secret patterns
    pub fn contains_secrets(&self, input: &str) -> bool {
        self.patterns.iter().any(|p| p.is_match(input))
    }
}

pub struct DataClassificationService {
    redactor: SecretRedactor,
}

impl Default for DataClassificationService {
    fn default() -> Self {
        Self::new()
    }
}

impl DataClassificationService {
    pub fn new() -> Self {
        Self {
            redactor: SecretRedactor::new(),
        }
    }

    /// Determines the classification level of text content based on keywords and patterns
    pub fn classify(&self, content: &str) -> DataClassificationLevel {
        let lower = content.to_lowercase();

        // Restricted: Passwords, credentials, private keys, financial credit cards, SSN
        if self.redactor.contains_secrets(content)
            || lower.contains("private key")
            || lower.contains("confidential - do not distribute")
            || lower.contains("salary")
            || lower.contains("classified")
        {
            return DataClassificationLevel::Restricted;
        }

        // Confidential: Internal financial statements, customer PII, trade secrets
        if lower.contains("confidential")
            || lower.contains("internal only")
            || lower.contains("proprietary")
            || lower.contains("nda")
        {
            return DataClassificationLevel::Confidential;
        }

        // Internal: Standard workspace documentation, internal project codes
        if lower.contains("company internal") || lower.contains("draft") {
            return DataClassificationLevel::Internal;
        }

        DataClassificationLevel::Public
    }

    pub fn sanitize_for_prompt(&self, content: &str) -> String {
        self.redactor.redact(content)
    }

    pub fn is_allowed_for_external_transfer(
        &self,
        level: DataClassificationLevel,
        approved_by_user: bool,
    ) -> bool {
        match level {
            DataClassificationLevel::Public => true,
            DataClassificationLevel::Internal => approved_by_user,
            DataClassificationLevel::Confidential => approved_by_user,
            DataClassificationLevel::Restricted => false, // Never automatically transfer restricted data
        }
    }
}
