# Phase 6 Security Architecture & Least Privilege

## 1. Windows Service Account Security

Background services running with elevated permissions represent an attractive target for privilege escalation. Phase 6 enforces the following security boundaries:

- **Service Account Isolation:** Recommended execution under `NT AUTHORITY\LocalService` or a dedicated virtual service account (`NT SERVICE\AAIRuntime`), avoiding full `NT AUTHORITY\SYSTEM` whenever interactive desktop control is not required.
- **Restricted Token & Privileges:** Unnecessary privileges (e.g. `SeDebugPrivilege`, `SeTcbPrivilege`) are stripped from the service process token.
- **Directory Permissions:** The workspace and database directories (`%APPDATA%\aai`) are ACL-protected to allow read/write only to the service account and the active administrative group.

## 2. IPC & Tauri Command Boundary

Communication between the Tauri Desktop UI and the background runtime service is mediated by authenticated local IPC:
- Local loopback only; no network-exposed HTTP listeners.
- All 16 Phase 6 Tauri IPC commands (`install_windows_service`, `kill_supervised_process`, `reconcile_unknown_task`, etc.) validate input boundaries and parameter lengths.
- Emergency stop triggers an instant, un-bypassable atomic hardware lock that stops all scheduled jobs, terminates supervised child processes, and drops database write locks.
