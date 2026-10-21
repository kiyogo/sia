use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorClass {
    Transient,
    Permanent,
    Resource,
    UserActionRequired,
    UnknownOutcome,
}

impl std::fmt::Display for ErrorClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorClass::Transient => write!(f, "Transient"),
            ErrorClass::Permanent => write!(f, "Permanent"),
            ErrorClass::Resource => write!(f, "Resource"),
            ErrorClass::UserActionRequired => write!(f, "UserActionRequired"),
            ErrorClass::UnknownOutcome => write!(f, "UnknownOutcome"),
        }
    }
}

pub struct ErrorClassifier;

impl ErrorClassifier {
    pub fn classify(status_code: Option<u16>, error_msg: &str, is_side_effect_sent: bool) -> ErrorClass {
        let msg_lower = error_msg.to_lowercase();

        // 1. If an external mutation was sent and the connection dropped before response, it is UnknownOutcome
        if is_side_effect_sent && (msg_lower.contains("timeout") || msg_lower.contains("connection dropped") || msg_lower.contains("reset by peer")) {
            return ErrorClass::UnknownOutcome;
        }

        // 2. HTTP Status Code Checks
        if let Some(code) = status_code {
            match code {
                429 | 502 | 503 | 504 => return ErrorClass::Transient,
                401 | 403 => return ErrorClass::Permanent, // Invalid credentials
                400 | 404 | 405 | 422 => return ErrorClass::Permanent,
                _ => {}
            }
        }

        // 3. User Action Required
        if msg_lower.contains("approval") || msg_lower.contains("permission denied") || msg_lower.contains("locked") || msg_lower.contains("interactive session") || msg_lower.contains("mfa") || msg_lower.contains("login") {
            return ErrorClass::UserActionRequired;
        }

        // 4. Resource Errors
        if msg_lower.contains("out of memory") || msg_lower.contains("disk full") || msg_lower.contains("no space left") || msg_lower.contains("resource limit") {
            return ErrorClass::Resource;
        }

        // 5. Permanent Errors
        if msg_lower.contains("invalid api key") || msg_lower.contains("unauthorized") || msg_lower.contains("cannot find") || msg_lower.contains("no such file") || msg_lower.contains("syntax error") {
            return ErrorClass::Permanent;
        }

        // 6. Transient Network Glitches
        if msg_lower.contains("timeout") || msg_lower.contains("connection refused") || msg_lower.contains("temporarily unavailable") || msg_lower.contains("network error") || msg_lower.contains("rate limit") {
            return ErrorClass::Transient;
        }

        // Default to Permanent to prevent infinite restart loops on unrecognized bugs
        ErrorClass::Permanent
    }
}
