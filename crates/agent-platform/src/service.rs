use crate::error::{PlatformError, PlatformResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatus {
    NotInstalled,
    Stopped,
    StartPending,
    Running,
    StopPending,
    Failed,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::NotInstalled => write!(f, "NotInstalled"),
            ServiceStatus::Stopped => write!(f, "Stopped"),
            ServiceStatus::StartPending => write!(f, "StartPending"),
            ServiceStatus::Running => write!(f, "Running"),
            ServiceStatus::StopPending => write!(f, "StopPending"),
            ServiceStatus::Failed => write!(f, "Failed"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStartupType {
    AutoStart,
    Manual,
    Disabled,
}

#[async_trait]
pub trait RuntimeServiceAdapter: Send + Sync {
    async fn status(&self, service_name: &str) -> PlatformResult<ServiceStatus>;
    async fn install(
        &self,
        service_name: &str,
        display_name: &str,
        exe_path: &Path,
        startup_type: ServiceStartupType,
    ) -> PlatformResult<()>;
    async fn uninstall(&self, service_name: &str) -> PlatformResult<()>;
    async fn start(&self, service_name: &str) -> PlatformResult<()>;
    async fn stop(&self, service_name: &str) -> PlatformResult<()>;
    async fn restart(&self, service_name: &str) -> PlatformResult<()>;
}

#[cfg(windows)]
pub struct WindowsServiceAdapter;

#[cfg(windows)]
impl WindowsServiceAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(windows)]
#[async_trait]
impl RuntimeServiceAdapter for WindowsServiceAdapter {
    async fn status(&self, service_name: &str) -> PlatformResult<ServiceStatus> {
        use std::ffi::OsStr;
        use windows_service::service::ServiceAccess;
        use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

        let manager = match ServiceManager::local_computer(None::<&OsStr>, ServiceManagerAccess::CONNECT) {
            Ok(m) => m,
            Err(e) => return Err(PlatformError::Os(format!("SCM connect error: {e}"))),
        };

        match manager.open_service(service_name, ServiceAccess::QUERY_STATUS) {
            Ok(service) => match service.query_status() {
                Ok(status) => match status.current_state {
                    windows_service::service::ServiceState::Stopped => Ok(ServiceStatus::Stopped),
                    windows_service::service::ServiceState::StartPending => Ok(ServiceStatus::StartPending),
                    windows_service::service::ServiceState::StopPending => Ok(ServiceStatus::StopPending),
                    windows_service::service::ServiceState::Running => Ok(ServiceStatus::Running),
                    _ => Ok(ServiceStatus::Failed),
                },
                Err(e) => Err(PlatformError::Os(format!("Failed to query status: {e}"))),
            },
            Err(windows_service::Error::Winapi(raw)) if raw.raw_os_error() == Some(1060) => {
                // ERROR_SERVICE_DOES_NOT_EXIST
                Ok(ServiceStatus::NotInstalled)
            }
            Err(e) => Err(PlatformError::Os(format!("Failed to open service: {e}"))),
        }
    }

    async fn install(
        &self,
        service_name: &str,
        display_name: &str,
        exe_path: &Path,
        startup_type: ServiceStartupType,
    ) -> PlatformResult<()> {
        use std::ffi::OsStr;
        use windows_service::service::{
            ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType,
        };
        use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

        let manager = ServiceManager::local_computer(
            None::<&OsStr>,
            ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE,
        )
        .map_err(|e| PlatformError::Os(format!("Requires Administrator privilege to install service: {e}")))?;

        let start_type = match startup_type {
            ServiceStartupType::AutoStart => ServiceStartType::AutoStart,
            ServiceStartupType::Manual => ServiceStartType::OnDemand,
            ServiceStartupType::Disabled => ServiceStartType::Disabled,
        };

        let service_info = ServiceInfo {
            name: std::ffi::OsString::from(service_name),
            display_name: std::ffi::OsString::from(display_name),
            service_type: ServiceType::OWN_PROCESS,
            start_type,
            error_control: ServiceErrorControl::Normal,
            executable_path: exe_path.to_path_buf(),
            launch_arguments: Vec::new(),
            dependencies: Vec::new(),
            account_name: None,
            account_password: None,
        };

        manager
            .create_service(&service_info, ServiceAccess::empty())
            .map_err(|e| PlatformError::Os(format!("Failed to create service: {e}")))?;

        Ok(())
    }

    async fn uninstall(&self, service_name: &str) -> PlatformResult<()> {
        use std::ffi::OsStr;
        use windows_service::service::ServiceAccess;
        use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

        let manager = ServiceManager::local_computer(None::<&OsStr>, ServiceManagerAccess::CONNECT)
            .map_err(|e| PlatformError::Os(format!("SCM connect error: {e}")))?;

        let service = manager
            .open_service(service_name, ServiceAccess::DELETE | ServiceAccess::STOP)
            .map_err(|e| PlatformError::Os(format!("Failed to open service for deletion: {e}")))?;

        let _ = service.stop();
        service
            .delete()
            .map_err(|e| PlatformError::Os(format!("Failed to delete service: {e}")))?;

        Ok(())
    }

    async fn start(&self, service_name: &str) -> PlatformResult<()> {
        use std::ffi::OsStr;
        use windows_service::service::ServiceAccess;
        use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

        let manager = ServiceManager::local_computer(None::<&OsStr>, ServiceManagerAccess::CONNECT)
            .map_err(|e| PlatformError::Os(format!("SCM connect error: {e}")))?;

        let service = manager
            .open_service(service_name, ServiceAccess::START)
            .map_err(|e| PlatformError::Os(format!("Failed to open service for start: {e}")))?;

        service
            .start::<&OsStr>(&[])
            .map_err(|e| PlatformError::Os(format!("Failed to start service: {e}")))?;

        Ok(())
    }

    async fn stop(&self, service_name: &str) -> PlatformResult<()> {
        use std::ffi::OsStr;
        use windows_service::service::ServiceAccess;
        use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

        let manager = ServiceManager::local_computer(None::<&OsStr>, ServiceManagerAccess::CONNECT)
            .map_err(|e| PlatformError::Os(format!("SCM connect error: {e}")))?;

        let service = manager
            .open_service(service_name, ServiceAccess::STOP)
            .map_err(|e| PlatformError::Os(format!("Failed to open service for stop: {e}")))?;

        service
            .stop()
            .map_err(|e| PlatformError::Os(format!("Failed to stop service: {e}")))?;

        Ok(())
    }

    async fn restart(&self, service_name: &str) -> PlatformResult<()> {
        let _ = self.stop(service_name).await;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        self.start(service_name).await
    }
}

/// Mock / In-Memory Service Adapter for testing and non-Windows environments
pub struct MockServiceAdapter {
    status: tokio::sync::RwLock<ServiceStatus>,
}

impl MockServiceAdapter {
    pub fn new(initial_status: ServiceStatus) -> Self {
        Self {
            status: tokio::sync::RwLock::new(initial_status),
        }
    }
}

#[async_trait]
impl RuntimeServiceAdapter for MockServiceAdapter {
    async fn status(&self, _service_name: &str) -> PlatformResult<ServiceStatus> {
        Ok(*self.status.read().await)
    }

    async fn install(
        &self,
        _service_name: &str,
        _display_name: &str,
        _exe_path: &Path,
        _startup_type: ServiceStartupType,
    ) -> PlatformResult<()> {
        let mut s = self.status.write().await;
        *s = ServiceStatus::Stopped;
        Ok(())
    }

    async fn uninstall(&self, _service_name: &str) -> PlatformResult<()> {
        let mut s = self.status.write().await;
        *s = ServiceStatus::NotInstalled;
        Ok(())
    }

    async fn start(&self, _service_name: &str) -> PlatformResult<()> {
        let mut s = self.status.write().await;
        if *s == ServiceStatus::NotInstalled {
            return Err(PlatformError::Os("Service not installed".to_string()));
        }
        *s = ServiceStatus::Running;
        Ok(())
    }

    async fn stop(&self, _service_name: &str) -> PlatformResult<()> {
        let mut s = self.status.write().await;
        if *s == ServiceStatus::NotInstalled {
            return Err(PlatformError::Os("Service not installed".to_string()));
        }
        *s = ServiceStatus::Stopped;
        Ok(())
    }

    async fn restart(&self, service_name: &str) -> PlatformResult<()> {
        self.stop(service_name).await?;
        self.start(service_name).await
    }
}
