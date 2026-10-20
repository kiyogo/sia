use crate::error::PlatformResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShellType {
    Cmd,
    PowerShell,
    Bash,
    Sh,
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellType::Cmd => write!(f, "cmd"),
            ShellType::PowerShell => write!(f, "powershell"),
            ShellType::Bash => write!(f, "bash"),
            ShellType::Sh => write!(f, "sh"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerequisiteStatus {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub required_version: String,
    pub install_hint: String,
}

#[async_trait]
pub trait OperatingSystemAdapter: Send + Sync {
    fn os_name(&self) -> &'static str;
    fn is_interactive(&self) -> bool;
    fn home_dir(&self) -> Option<PathBuf>;
    fn temp_dir(&self) -> PathBuf;
    async fn check_prerequisites(&self) -> Vec<PrerequisiteStatus>;
}

pub type StreamCallback = Box<dyn Fn(String) + Send + Sync + 'static>;

#[async_trait]
pub trait ShellExecutor: Send + Sync {
    async fn execute(
        &self,
        shell: ShellType,
        command: &str,
        cwd: &Path,
        envs: &[(String, String)],
        timeout_ms: Option<u64>,
    ) -> PlatformResult<ProcessOutput>;

    async fn execute_with_stream(
        &self,
        shell: ShellType,
        command: &str,
        cwd: &Path,
        envs: &[(String, String)],
        timeout_ms: Option<u64>,
        on_stdout: Option<StreamCallback>,
    ) -> PlatformResult<ProcessOutput>;
}

pub trait FileSystemProvider: Send + Sync {
    fn canonicalize(&self, path: &Path) -> PlatformResult<PathBuf>;
    fn is_within_boundary(&self, boundary: &Path, path: &Path) -> bool;
    fn normalize_path(&self, path: &Path) -> PathBuf;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticMessage {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub code: Option<String>,
    pub context_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub diagnostics: Vec<DiagnosticMessage>,
    pub root_cause: Option<DiagnosticMessage>,
    pub total_errors: usize,
    pub total_warnings: usize,
    pub raw_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub root_path: PathBuf,
    pub languages: Vec<String>,
    pub primary_language: Option<String>,
    pub build_systems: Vec<String>,
    pub test_runners: Vec<String>,
    pub entry_points: Vec<PathBuf>,
    pub is_git: bool,
    pub git_branch: Option<String>,
}

#[async_trait]
pub trait ComputerControlAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    async fn get_desktop_state(&self) -> PlatformResult<serde_json::Value>;
    async fn switch_window(&self, title: &str) -> PlatformResult<bool>;
    async fn take_screenshot(&self) -> PlatformResult<Vec<u8>>;
}

#[async_trait]
pub trait LanguageToolingAdapter: Send + Sync {
    fn language_name(&self) -> &'static str;
    fn detect_project(&self, root: &Path) -> Option<ProjectInfo>;
    fn build_command(&self, root: &Path) -> Option<(String, Vec<String>)>;
    fn test_command(&self, root: &Path, filter: Option<&str>) -> Option<(String, Vec<String>)>;
    fn parse_diagnostics(&self, stdout: &str, stderr: &str) -> DiagnosticReport;
}

#[async_trait]
pub trait CodingEngineAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    async fn detect_project(&self, workspace: &Path) -> PlatformResult<ProjectInfo>;
    async fn analyze_code(&self, workspace: &Path, query: &str) -> PlatformResult<String>;
}
