pub mod error;
pub mod language;
pub mod stub;
pub mod traits;

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
}
