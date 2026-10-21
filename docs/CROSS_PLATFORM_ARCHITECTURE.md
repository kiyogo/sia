# Cross-Platform Architecture & Evolution
**Universal Autonomous AI Workspace**
**Current Platform:** Windows 10/11 | **Target Platforms:** Linux (X11 / Wayland), macOS (Cocoa / Accessibility)

---

## 1. Platform Abstraction Strategy
Although Phase 1 through Phase 5 are implemented Windows-first, the architecture isolates all operating system specifics behind Rust traits in `crates/agent-platform/src/traits.rs`:

```rust
pub trait OperatingSystemAdapter: Send + Sync { ... }
pub trait ShellExecutor: Send + Sync { ... }
pub trait FileSystemProvider: Send + Sync { ... }
pub trait ComputerControlAdapter: Send + Sync { ... }
pub trait BrowserAutomationAdapter: Send + Sync { ... }
```

No platform-specific API calls (Win32, UIA, COM) are leaked into `agent-core`, `agent-runtime`, or `agent-security`.

---

## 2. Multi-Agent Orchestration Portability (Phase 4 & Phase 5)
The Multi-Agent Orchestration Subsystem and Business Automation Engine are **100% OS-agnostic**:
1. **Task DAG & Topological Sort**: Implemented in pure Rust using Kahn's topological sorting algorithm and standard Tokio channels. Completely portable across all platforms.
2. **Worker Scheduling & Concurrency**: The Tokio async thread pool operates with identical bounded concurrency (1-4 workers) and cancellation semantics on Windows, Linux, and macOS.
3. **Shared Artifact Store**: Uses content-addressed storage (SHA-256) backed by the portable filesystem and SQLite WAL mode. Paths are stored normalized using canonical `PathBuf`.
4. **Pure Rust Document Processing (Phase 5)**:
   - **OpenXML DOCX Generation & Ingestion**: Implemented purely in Rust using the `zip` crate. Zero reliance on Microsoft Office COM automation, LibreOffice, or external C dynamic libraries.
   - **Headless PDF Generation**: Emits standard PDF 1.4 binary streams directly in memory.
   - **Spreadsheet & Data Analysis**: CSV parsing and XLSX OpenXML extraction operate identically on Windows, Linux, and macOS.
   - **SVG Chart Generation**: Native vector graphics string generation with zero graphics library dependencies.
5. **Business Connectors & Security**:
   - SQLite `business_store` tables operate with WAL mode across all target filesystems.
   - Rate limiting, circuit breaking, and secret redaction are pure Rust async constructs.

---

## 3. Platform Comparison Matrix

| Capability | Windows (Phases 1–5) | Linux (Target) | macOS (Target) |
|---|---|---|---|
| **Multi-Agent Runtime** | Tokio + SQLite WAL | Tokio + SQLite WAL | Tokio + SQLite WAL |
| **Task DAG Engine** | Pure Rust Kahn Sort | Pure Rust Kahn Sort | Pure Rust Kahn Sort |
| **Business & Research Engine** | Pure Rust (`agent-core`) | Pure Rust (`agent-core`) | Pure Rust (`agent-core`) |
| **Document (DOCX/PDF) Engine** | Pure Rust (`zip` crate) | Pure Rust (`zip` crate) | Pure Rust (`zip` crate) |
| **Data & SVG Chart Engine** | Pure Rust | Pure Rust | Pure Rust |
| **Shared Artifact Store** | SHA-256 + FS + SQLite | SHA-256 + FS + SQLite | SHA-256 + FS + SQLite |
| **Shell Executor** | `cmd.exe`, `powershell.exe` | `bash`, `sh`, `zsh` | `zsh`, `bash` |
| **Desktop Automation MCP** | `Windows-MCP` (UIA / Win32) | `Linux-MCP` (AT-SPI / libxdo / ydotool) | `macOS-MCP` (AXUIElement / AppleScript) |
| **Browser Automation MCP** | `Playwright MCP` (Chromium/Edge) | `Playwright MCP` (Chromium/Firefox) | `Playwright MCP` (Chromium/WebKit) |
| **Screen Capture** | DXGI / MSS / GDI | X11 XShm / PipeWire Wayland Portal | CoreGraphics ScreenCaptureKit |
| **Workstation Lock Status** | `logonui` process / Winlogon desktop | `loginctl` / ScreenSaver D-Bus | `CGSessionCopyCurrentDictionary` |
| **GUI Framework** | Tauri 2 (WebView2) | Tauri 2 (WebKitGTK) | Tauri 2 (WKWebView) |

---

## 4. Portability Principles
1. **MCP as Universal Driver Boundary**: Because both `Windows-MCP` and `Playwright MCP` speak standard JSON-RPC 2.0 over stdio, swapping OS backends requires changing only the spawned subprocess command without altering the agent orchestrator.
2. **File Paths**: All file operations use `std::path::PathBuf` with canonicalization and normalization to forward slashes across JSON interfaces.
3. **Line Endings**: Parsing logic handles both `\r\n` (CRLF) and `\n` (LF) transparently.
4. **Binary Releases**: Built via Cargo workspace on each target runner with zero non-standard C dynamic dependencies.
