# Cross-Platform Architecture & Evolution
**Universal Autonomous AI Workspace**
**Current Platform:** Windows 10/11 | **Target Platforms:** Linux (X11 / Wayland), macOS (Cocoa / Accessibility)

---

## 1. Platform Abstraction Strategy
Although Phase 3 is implemented Windows-first, the architecture isolates all operating system specifics behind Rust traits in `crates/agent-platform/src/traits.rs`:

```rust
pub trait OperatingSystemAdapter: Send + Sync { ... }
pub trait ShellExecutor: Send + Sync { ... }
pub trait FileSystemProvider: Send + Sync { ... }
pub trait ComputerControlAdapter: Send + Sync { ... }
pub trait BrowserAutomationAdapter: Send + Sync { ... }
```

No platform-specific API calls (Win32, UIA, COM) are leaked into `agent-core`, `agent-runtime`, or `agent-security`.

---

## 2. Platform Comparison Matrix

| Capability | Windows (Phase 3) | Linux (Target Phase 4+) | macOS (Target Phase 4+) |
|---|---|---|---|
| **Shell Executor** | `cmd.exe`, `powershell.exe` | `bash`, `sh`, `zsh` | `zsh`, `bash` |
| **Desktop Automation MCP** | `Windows-MCP` (UIA / Win32) | `Linux-MCP` (AT-SPI / libxdo / ydotool) | `macOS-MCP` (AXUIElement / AppleScript) |
| **Browser Automation MCP** | `Playwright MCP` (Chromium/Edge) | `Playwright MCP` (Chromium/Firefox) | `Playwright MCP` (Chromium/WebKit) |
| **Screen Capture** | DXGI / MSS / GDI | X11 XShm / PipeWire Wayland Portal | CoreGraphics ScreenCaptureKit |
| **Workstation Lock Status** | `logonui` process / Winlogon desktop | `loginctl` / ScreenSaver D-Bus | `CGSessionCopyCurrentDictionary` |
| **GUI Framework** | Tauri 2 (WebView2) | Tauri 2 (WebKitGTK) | Tauri 2 (WKWebView) |

---

## 3. Portability Principles
1. **MCP as Universal Driver Boundary**: Because both `Windows-MCP` and `Playwright MCP` speak standard JSON-RPC 2.0 over stdio, swapping OS backends requires changing only the spawned subprocess command without altering the agent orchestrator.
2. **File Paths**: All file operations use `std::path::PathBuf` with canonicalization and normalization to forward slashes across JSON interfaces.
3. **Line Endings**: Parsing logic handles both `\r\n` (CRLF) and `\n` (LF) transparently.
4. **Binary Releases**: Built via Cargo workspace on each target runner with no non-standard C dynamic dependencies.
