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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DesktopLockStatus {
    Unlocked,
    Locked,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub process_name: String,
    pub is_active: bool,
    pub bounds: Option<WindowBounds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiElementNode {
    pub id: String,
    pub role: String,
    pub name: String,
    pub value: Option<String>,
    pub bounds: Option<WindowBounds>,
    pub is_interactive: bool,
    pub children: Vec<UiElementNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowserTabInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomSnapshotNode {
    pub ref_id: String,
    pub tag: String,
    pub role: Option<String>,
    pub text: Option<String>,
    pub value: Option<String>,
    pub is_interactive: bool,
    pub attributes: std::collections::HashMap<String, String>,
    pub children: Vec<DomSnapshotNode>,
}

#[async_trait]
pub trait ComputerControlAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    async fn get_lock_status(&self) -> PlatformResult<DesktopLockStatus>;
    async fn list_windows(&self) -> PlatformResult<Vec<WindowInfo>>;
    async fn get_active_window(&self) -> PlatformResult<Option<WindowInfo>>;
    async fn switch_window(&self, title: &str) -> PlatformResult<bool>;
    async fn get_desktop_state(&self) -> PlatformResult<serde_json::Value>;
    async fn take_screenshot(&self) -> PlatformResult<Vec<u8>>;
    async fn take_focused_screenshot(&self, window_title: Option<&str>) -> PlatformResult<Vec<u8>>;
}

#[async_trait]
pub trait BrowserAutomationAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    async fn navigate(&self, url: &str) -> PlatformResult<String>;
    async fn get_snapshot(&self) -> PlatformResult<DomSnapshotNode>;
    async fn click_element(&self, selector_or_ref: &str) -> PlatformResult<bool>;
    async fn fill_element(&self, selector_or_ref: &str, value: &str) -> PlatformResult<bool>;
    async fn list_tabs(&self) -> PlatformResult<Vec<BrowserTabInfo>>;
    async fn close(&self) -> PlatformResult<()>;
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
