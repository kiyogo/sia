# Cross-Platform 24/7 Runtime Architecture

## 1. Abstraction Strategy

While Phase 6 prioritizes Windows 10/11 through native Windows Service Control Manager (SCM) and Win32 Session APIs, all runtime systems are decoupled behind platform-agnostic Rust traits:

| Subsystem | Windows 10/11 | Linux (Ubuntu, Debian, RHEL) | macOS (Darwin) |
| :--- | :--- | :--- | :--- |
| **Service Host** | Windows SCM (`windows-service`) | `systemd` unit (`.service`) | `launchd` daemon (`.plist`) |
| **Process Tree Kill** | `taskkill.exe /PID <pid> /T /F` | `kill(-pid, SIGKILL)` (process groups) | `kill(-pid, SIGKILL)` |
| **Hardware Metrics** | `sysinfo` (Win32 PDH / WMI) | `sysinfo` (`/proc`, `/sys`) | `sysinfo` (`sysctl`, `host_info`) |
| **Interactive Bridge** | `ProcessIdToSessionId` + `logonui` | Wayland / X11 `$DISPLAY` inspection | WindowServer / Quartz Core detection |
| **Persistent Storage** | SQLite WAL (Win32 file locks) | SQLite WAL (`fcntl` advisory locks) | SQLite WAL (`flock` locks) |

## 2. Linux & macOS Readiness

The `RuntimeServiceAdapter` trait allows plug-and-play implementations:
- `SystemdServiceAdapter`: Writes unit definitions to `/etc/systemd/system/aai-runtime.service` and executes `systemctl enable --now aai-runtime`.
- `LaunchdServiceAdapter`: Writes XML property lists to `/Library/LaunchDaemons/com.aai.runtime.plist` and calls `launchctl bootstrap`.
