use crate::error::{PlatformError, PlatformResult};
use crate::traits::{
    FileSystemProvider, OperatingSystemAdapter, PrerequisiteStatus, ProcessOutput, ShellExecutor,
    ShellType, StreamCallback,
};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub struct WindowsOperatingSystemAdapter;

impl WindowsOperatingSystemAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsOperatingSystemAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OperatingSystemAdapter for WindowsOperatingSystemAdapter {
    fn os_name(&self) -> &'static str {
        "Windows"
    }

    fn is_interactive(&self) -> bool {
        // Check if interactive user session is active (SESSIONNAME or USERNAME)
        std::env::var("SESSIONNAME").is_ok() || std::env::var("USERNAME").is_ok()
    }

    fn home_dir(&self) -> Option<PathBuf> {
        std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
    }

    fn temp_dir(&self) -> PathBuf {
        std::env::temp_dir()
    }

    async fn check_prerequisites(&self) -> Vec<PrerequisiteStatus> {
        let mut results = Vec::new();

        // 1. Check Python >= 3.14 (required by Windows-MCP)
        let python_check = check_python_314().await;
        results.push(python_check);

        // 2. Check uv
        let uv_check = check_command(
            "uv",
            &["--version"],
            "uv",
            ">=0.1",
            "Install uv via powershell: 'irm https://astral.sh/uv/install.ps1 | iex' or 'pip install uv'",
        )
        .await;
        results.push(uv_check);

        // 3. Check git
        let git_check = check_command(
            "git",
            &["--version"],
            "Git",
            ">=2.0",
            "Download and install Git for Windows from https://git-scm.com/download/win",
        )
        .await;
        results.push(git_check);

        results
    }
}

async fn check_python_314() -> PrerequisiteStatus {
    // Check python commands: python, py -3.14, or uv python find
    let commands_to_try = [
        ("python", vec!["--version"]),
        ("py", vec!["-3.14", "--version"]),
        ("uv", vec!["python", "find", "3.14"]),
    ];

    for (cmd, args) in commands_to_try {
        if let Ok(output) = Command::new(cmd).args(&args).output().await {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).to_string()
                    + &String::from_utf8_lossy(&output.stderr);
                if text.contains("3.14") || text.contains("3.15") || text.contains("3.16") {
                    return PrerequisiteStatus {
                        name: "Python 3.14+".to_string(),
                        installed: true,
                        version: Some(text.trim().to_string()),
                        required_version: ">=3.14 (for Windows-MCP)".to_string(),
                        install_hint: "Python 3.14+ detected successfully.".to_string(),
                    };
                }
            }
        }
    }

    PrerequisiteStatus {
        name: "Python 3.14+".to_string(),
        installed: false,
        version: None,
        required_version: ">=3.14 (for Windows-MCP)".to_string(),
        install_hint: "Windows-MCP requires Python 3.14+. Run 'uv python install 3.14' or download from python.org.".to_string(),
    }
}

async fn check_command(
    cmd: &str,
    args: &[&str],
    name: &str,
    required_version: &str,
    hint: &str,
) -> PrerequisiteStatus {
    match Command::new(cmd).args(args).output().await {
        Ok(out) if out.status.success() => {
            let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            PrerequisiteStatus {
                name: name.to_string(),
                installed: true,
                version: Some(ver),
                required_version: required_version.to_string(),
                install_hint: format!("{name} is installed and available in PATH."),
            }
        }
        _ => PrerequisiteStatus {
            name: name.to_string(),
            installed: false,
            version: None,
            required_version: required_version.to_string(),
            install_hint: hint.to_string(),
        },
    }
}

pub struct WindowsShellExecutor;

impl WindowsShellExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ShellExecutor for WindowsShellExecutor {
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
        command: &str,
        cwd: &Path,
        envs: &[(String, String)],
        timeout_ms: Option<u64>,
        on_stdout: Option<StreamCallback>,
    ) -> PlatformResult<ProcessOutput> {
        let start = Instant::now();
        let timeout_duration = std::time::Duration::from_millis(timeout_ms.unwrap_or(60_000));

        let mut cmd = match shell {
            ShellType::Cmd => {
                let mut c = Command::new("cmd.exe");
                c.args(["/d", "/c", command]);
                c
            }
            ShellType::PowerShell => {
                let mut c = Command::new("powershell.exe");
                c.args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    command,
                ]);
                c
            }
            ShellType::Bash | ShellType::Sh => {
                // If git bash or wsl bash is on path, execute bash -c
                let mut c = Command::new("bash.exe");
                c.args(["-c", command]);
                c
            }
        };

        cmd.current_dir(cwd);
        for (k, v) in envs {
            cmd.env(k, v);
        }

        // Configure piped stdout/stderr
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            PlatformError::ExecutionFailed(format!(
                "Failed to spawn process {shell:?} for '{command}': {e}"
            ))
        })?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let stdout_handle = tokio::spawn(async move {
            let mut out = String::new();
            if let Some(stdout) = stdout {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    if let Some(cb) = &on_stdout {
                        cb(line.clone());
                    }
                    out.push_str(&line);
                    out.push('\n');
                }
            }
            out
        });

        let stderr_handle = tokio::spawn(async move {
            let mut err = String::new();
            if let Some(stderr) = stderr {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    err.push_str(&line);
                    err.push('\n');
                }
            }
            err
        });

        let wait_result = tokio::time::timeout(timeout_duration, child.wait()).await;

        match wait_result {
            Ok(status_res) => {
                let status = status_res.map_err(|e| {
                    PlatformError::ExecutionFailed(format!("Error waiting for process: {e}"))
                })?;

                let captured_stdout = stdout_handle.await.unwrap_or_default();
                let captured_stderr = stderr_handle.await.unwrap_or_default();

                let duration_ms = start.elapsed().as_millis() as u64;

                Ok(ProcessOutput {
                    stdout: captured_stdout,
                    stderr: captured_stderr,
                    exit_code: status.code(),
                    duration_ms,
                    truncated: false,
                })
            }
            Err(_) => {
                // Timeout occurred, kill process
                let _ = child.kill().await;
                Err(PlatformError::Timeout(timeout_duration.as_millis() as u64))
            }
        }
    }
}

pub struct WindowsFileSystemProvider;

impl WindowsFileSystemProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsFileSystemProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemProvider for WindowsFileSystemProvider {
    fn canonicalize(&self, path: &Path) -> PlatformResult<PathBuf> {
        let canonical = std::fs::canonicalize(path).map_err(|e| {
            PlatformError::FileSystem(format!("Failed to canonicalize path {path:?}: {e}"))
        })?;
        Ok(self.normalize_path(&canonical))
    }

    fn normalize_path(&self, path: &Path) -> PathBuf {
        let s = path.to_string_lossy();
        // Remove Windows \\?\ UNC prefix if present
        let clean = if let Some(stripped) = s.strip_prefix(r"\\?\") {
            stripped
        } else {
            &s
        };
        PathBuf::from(clean)
    }

    fn is_within_boundary(&self, boundary: &Path, path: &Path) -> bool {
        let b = self.normalize_path(boundary);
        let p = self.normalize_path(path);

        let b_str = b.to_string_lossy().to_lowercase();
        let p_str = p.to_string_lossy().to_lowercase();

        p_str.starts_with(&b_str)
    }
}
