use crate::error::{PlatformError, PlatformResult};
use crate::traits::{
    FileSystemProvider, OperatingSystemAdapter, PrerequisiteStatus, ProcessOutput, ShellExecutor,
    ShellType, StreamCallback,
};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub struct GenericOperatingSystemAdapter;

#[async_trait]
impl OperatingSystemAdapter for GenericOperatingSystemAdapter {
    fn os_name(&self) -> &'static str {
        if cfg!(target_os = "linux") {
            "Linux"
        } else if cfg!(target_os = "macos") {
            "macOS"
        } else {
            "Unknown"
        }
    }

    fn is_interactive(&self) -> bool {
        std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok()
    }

    fn home_dir(&self) -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }

    fn temp_dir(&self) -> PathBuf {
        std::env::temp_dir()
    }

    async fn check_prerequisites(&self) -> Vec<PrerequisiteStatus> {
        Vec::new()
    }
}

pub struct GenericShellExecutor;

#[async_trait]
impl ShellExecutor for GenericShellExecutor {
    async fn execute(
        &self,
        shell: ShellType,
        command: &str,
        cwd: &Path,
        envs: &[(String, String)],
        timeout_ms: Option<u64>,
    ) -> PlatformResult<ProcessOutput> {
        self.execute_with_stream(shell, command, cwd, envs, timeout_ms, None)
            .await
    }

    async fn execute_with_stream(
        &self,
        shell: ShellType,
        _command: &str,
        _cwd: &Path,
        _envs: &[(String, String)],
        _timeout_ms: Option<u64>,
        _on_stdout: Option<StreamCallback>,
    ) -> PlatformResult<ProcessOutput> {
        Err(PlatformError::NotSupported(format!(
            "Shell {shell:?} is not yet implemented for non-Windows platform in Phase 1"
        )))
    }
}

pub struct GenericFileSystemProvider;

impl FileSystemProvider for GenericFileSystemProvider {
    fn canonicalize(&self, path: &Path) -> PlatformResult<PathBuf> {
        std::fs::canonicalize(path).map_err(|e| PlatformError::FileSystem(e.to_string()))
    }

    fn is_within_boundary(&self, boundary: &Path, path: &Path) -> bool {
        let b = self.normalize_path(boundary);
        let p = self.normalize_path(path);
        p.starts_with(b)
    }

    fn normalize_path(&self, path: &Path) -> PathBuf {
        use std::path::Component;
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    if !components.is_empty() {
                        components.pop();
                    }
                }
                Component::CurDir => {}
                c => components.push(c),
            }
        }
        components.iter().collect()
    }
}

pub struct GenericComputerControlAdapter;

#[async_trait]
impl crate::traits::ComputerControlAdapter for GenericComputerControlAdapter {
    fn name(&self) -> &'static str {
        "GenericComputerControl"
    }

    fn is_available(&self) -> bool {
        false
    }

    async fn get_lock_status(&self) -> PlatformResult<crate::traits::DesktopLockStatus> {
        Ok(crate::traits::DesktopLockStatus::Unknown)
    }

    async fn list_windows(&self) -> PlatformResult<Vec<crate::traits::WindowInfo>> {
        Ok(Vec::new())
    }

    async fn get_active_window(&self) -> PlatformResult<Option<crate::traits::WindowInfo>> {
        Ok(None)
    }

    async fn switch_window(&self, _title: &str) -> PlatformResult<bool> {
        Ok(false)
    }

    async fn get_desktop_state(&self) -> PlatformResult<serde_json::Value> {
        Ok(serde_json::json!({ "status": "not_supported" }))
    }

    async fn take_screenshot(&self) -> PlatformResult<Vec<u8>> {
        Err(PlatformError::NotSupported("Screenshots not supported on this platform".to_string()))
    }

    async fn take_focused_screenshot(&self, _window_title: Option<&str>) -> PlatformResult<Vec<u8>> {
        Err(PlatformError::NotSupported("Screenshots not supported on this platform".to_string()))
    }
}
