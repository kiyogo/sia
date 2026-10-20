use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Model provider error: {0}")]
    Model(String),

    #[error("Storage error: {0}")]
    Storage(#[from] agent_storage::StorageError),

    #[error("Tool error: {0}")]
    Tool(#[from] agent_tools::ToolError),

    #[error("Security error: {0}")]
    Security(#[from] agent_security::SecurityError),

    #[error("Maximum agent loops reached ({0}) without completion")]
    MaxLoopsExceeded(u32),

    #[error("Context budget exceeded ({current} tokens > {max_budget} budget)")]
    ContextBudgetExceeded { current: u32, max_budget: u32 },

    #[error("Agent operation cancelled")]
    Cancelled,

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type CoreResult<T> = Result<T, CoreError>;
