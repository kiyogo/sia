use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Platform operation not supported on this operating system: {0}")]
    NotSupported(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Process timed out after {0} ms")]
    Timeout(u64),

    #[error("Process was cancelled")]
    Cancelled,

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Interactive session required but not available: {0}")]
    NoInteractiveSession(String),

    #[error("Missing system prerequisite: {0}")]
    MissingPrerequisite(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type PlatformResult<T> = Result<T, PlatformError>;
