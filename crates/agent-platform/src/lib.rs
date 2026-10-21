pub mod error;
pub mod language;
pub mod resources;
pub mod service;
pub mod session;
pub mod stub;
pub mod traits;

pub use resources::*;
pub use service::*;
pub use session::*;

#[cfg(target_os = "windows")]
pub mod windows;

pub use error::{PlatformError, PlatformResult};
pub use language::MultiLanguageAdapter;
pub use traits::*;

#[cfg(target_os = "windows")]
pub use windows::{WindowsComputerControlAdapter, WindowsFileSystemProvider, WindowsOperatingSystemAdapter, WindowsShellExecutor};

#[cfg(not(target_os = "windows"))]
pub use stub::{GenericComputerControlAdapter as WindowsComputerControlAdapter, GenericFileSystemProvider as WindowsFileSystemProvider, GenericOperatingSystemAdapter as WindowsOperatingSystemAdapter, GenericShellExecutor as WindowsShellExecutor};

/// Factory function to return the default platform adapter for the current host OS
pub fn get_platform_adapters() -> (
    Box<dyn OperatingSystemAdapter>,
    Box<dyn ShellExecutor>,
    Box<dyn FileSystemProvider>,
) {
    #[cfg(target_os = "windows")]
    {
        (
            Box::new(WindowsOperatingSystemAdapter::new()),
            Box::new(WindowsShellExecutor::new()),
            Box::new(WindowsFileSystemProvider::new()),
        )
    }

    #[cfg(not(target_os = "windows"))]
    {
        (
            Box::new(stub::GenericOperatingSystemAdapter),
            Box::new(stub::GenericShellExecutor),
            Box::new(stub::GenericFileSystemProvider),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_os_adapter() {
        let (os, _, _) = get_platform_adapters();
        assert_eq!(os.os_name(), "Windows");
        assert!(os.home_dir().is_some());
    }

    #[tokio::test]
    async fn test_shell_executor_cmd() {
        let (_, shell, _) = get_platform_adapters();
        let temp = std::env::temp_dir();
        let output = shell
            .execute(ShellType::Cmd, "echo HelloPlatform", &temp, &[], Some(5000))
            .await
            .expect("CMD echo should work");
        assert_eq!(output.exit_code, Some(0));
        assert!(output.stdout.contains("HelloPlatform"));
    }

    #[tokio::test]
    async fn test_shell_executor_powershell() {
        let (_, shell, _) = get_platform_adapters();
        let temp = std::env::temp_dir();
        let output = shell
            .execute(
                ShellType::PowerShell,
                "Write-Output 'PowerShellActive'",
                &temp,
                &[],
                Some(5000),
            )
            .await
            .expect("PowerShell write-output should work");
        assert_eq!(output.exit_code, Some(0));
        assert!(output.stdout.contains("PowerShellActive"));
    }

    #[test]
    fn test_filesystem_boundary() {
        let (_, _, fs) = get_platform_adapters();
        let base = std::path::PathBuf::from(r"C:\workspace\project");
        let safe = std::path::PathBuf::from(r"C:\workspace\project\src\main.rs");
        let escape = std::path::PathBuf::from(r"C:\Windows\System32");

        assert!(fs.is_within_boundary(&base, &safe));
        assert!(!fs.is_within_boundary(&base, &escape));
    }

    #[tokio::test]
    async fn test_service_adapter_mock_lifecycle() {
        let adapter = MockServiceAdapter::new(ServiceStatus::NotInstalled);
        assert_eq!(adapter.status("TestAiService").await.unwrap(), ServiceStatus::NotInstalled);

        adapter.install("TestAiService", "Test AI Service", std::path::Path::new("test.exe"), ServiceStartupType::AutoStart)
            .await
            .expect("install");
        assert_eq!(adapter.status("TestAiService").await.unwrap(), ServiceStatus::Stopped);

        adapter.start("TestAiService").await.expect("start");
        assert_eq!(adapter.status("TestAiService").await.unwrap(), ServiceStatus::Running);

        adapter.stop("TestAiService").await.expect("stop");
        assert_eq!(adapter.status("TestAiService").await.unwrap(), ServiceStatus::Stopped);

        adapter.uninstall("TestAiService").await.expect("uninstall");
        assert_eq!(adapter.status("TestAiService").await.unwrap(), ServiceStatus::NotInstalled);
    }

    #[test]
    fn test_session_provider_mock() {
        let session = MockSessionProvider::new(1, true, false);
        assert!(session.is_interactive_desktop_available());

        session.set_locked(true);
        assert!(!session.is_interactive_desktop_available());

        session.set_locked(false);
        session.set_session_zero(true);
        assert!(!session.is_interactive_desktop_available());
    }

    #[test]
    fn test_system_resource_monitor() {
        let monitor = SystemResourceMonitor::new();
        let snap = monitor.get_snapshot();
        assert!(snap.memory_total_mb > 0);
        assert!(snap.process_count > 0);
        assert!(snap.disk_free_gb >= 0.0);
        // Backpressure check with impossible 100% threshold returns false
        assert!(!monitor.is_under_backpressure(101.0, 101.0));
    }
}
