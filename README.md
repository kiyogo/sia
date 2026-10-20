# Universal Autonomous AI Workspace
### Windows-First, Cross-Platform-Ready, Open-Source-First Desktop Agent Platform

An autonomous, multi-purpose AI Workspace designed to perform real work on Windows. The platform bridges natural language instructions from users with native system capabilities: executing terminal commands in CMD/PowerShell, analyzing and modifying codebases across multiple languages, controlling Windows applications, navigating modern web browsers via Playwright MCP, routing tasks through cost-optimized execution paths, persisting task milestones in SQLite, and strictly optimizing token consumption.

---

## 1. System Vision & Architecture

The application is structured as a multi-crate Rust workspace with a desktop UI built using Tauri 2, React 18, TypeScript, and Tailwind CSS.

```text
aai/
├── apps/
│   └── desktop/                  # Tauri 2 + React + TypeScript + Tailwind UI
│       ├── src/                  # React UI (ComputerControl, BrowserWorkspace, CodingWorkspace, Chat, Tasks, Terminal)
│       └── src-tauri/            # Tauri 2 host binary & IPC command handlers
├── crates/
│   ├── agent-platform/           # OS-agnostic traits, Win32 adapters, generic stubs & MultiLanguageAdapter
│   ├── agent-security/           # Permission tiers, boundary sandbox, DesktopOwnershipManager, Emergency Stop
│   ├── agent-storage/            # SQLite embedded database with WAL mode and schema migrations
│   ├── agent-tools/              # Browser MCP, Windows MCP, Git, OpenCode code_edit/patch, search, repo_map
│   ├── agent-core/               # Agent loop, 6-Tier Tool Strategy Router, Action Verifier, Token Optimizer
│   └── agent-runtime/            # Task runner, checkpoint recovery engine, and broadcast event streaming
├── docs/                         # Comprehensive architecture, security, token optimization, and QA reports
├── scripts/                      # PowerShell setup, dev, test, and release build automation scripts
├── LICENSE                       # MIT License
├── THIRD_PARTY_NOTICES.md        # Open-source attributions (Playwright, Windows-MCP, OpenCode, Tauri, etc.)
└── README.md
```

---

## 2. Core Capabilities Across Phases

### 2.1 Phase 3: Advanced Computer & Browser Agent
1. **Windows Desktop Control**:
   - `WindowsComputerControlAdapter` enumerates top-level application windows, focuses targets, retrieves client coordinates, and queries workstation lock status (`WM_WTSSESSION_CHANGE`).
   - `Windows-MCP` integration with Python 3.14 via Astral `uv` for deep Windows accessibility and UI automation.
   - Screen capture with focused window clipping to minimize unnecessary pixel processing.
2. **Playwright Browser Automation**:
   - `BrowserMcpManager` orchestrating `@playwright/mcp` over stdio JSON-RPC.
   - Real-time tab management, multi-tab switching, navigation, form inputs, and click emulation.
   - Compact DOM accessibility tree extraction with numeric element reference indexing.
   - File download tracking with SHA-256 integrity verification and automated upload handling.
3. **6-Tier Tool Strategy Router**:
   - Routes user goals through the lowest token and latency cost tier:
     `Direct API` → `Filesystem / DB` → `Shell / CLI` → `Playwright DOM` → `Windows UIA` → `Screenshot / Vision`.
4. **Action Verifier (Observe-Interpret-Act-Verify)**:
   - Evaluates explicit post-conditions (`UrlMatches`, `ElementVisible`, `FileExists`, `WindowActive`, `ExitCodeZero`) before declaring success.
5. **Desktop Ownership & Safety Framework**:
   - `DesktopOwnershipManager` single-controller leasing (default 30s) prevents conflicting inputs.
   - User takeover detection and instant Emergency Stop (`< 5ms`).
   - Workstation lock screen safety pauses automation automatically.
6. **Token Optimization**:
   - Accessibility-first trees (~185 tokens) yield **>90% token reduction** over traditional vision screenshots (~2,200 tokens).
   - Incremental state hash delta pruning avoids re-sending unchanged UI state.
7. **Desktop UI Control Panels**:
   - **Computer Control**: Live screen canvas preview, lease countdown bar, takeover button, window list table.
   - **Browser Workspace**: Tab strip, navigation bar, interactive DOM tree inspector, and form action executor.

### 2.2 Phase 2: Advanced Autonomous Coding Agent
- **OpenCode Architectural Integration**: `CodeEditTool` exact-match replacement, `CodePatchTool` atomic multi-file rollback.
- **Multi-Language Intelligence**: Automated project detection for Rust, TypeScript/JS, Python, and C/C++.
- **Compiler Diagnostics**: Parsers for `rustc`, `tsc`, and Python tracebacks with 85% token deduplication.
- **Git Safety & Working Tree Protection**: Safe read-only inspection; destructive git commands strictly blocked.
- **Three Coding Modes**: `Ask` (read-only), `Plan` (diff preview), `Build` (autonomous repair loop).
- **Desktop Coding Workspace**: Source Viewer, Unified Diff Inspector, Step Tracker, and Problems Panel.

### 2.3 Phase 1: General Agent Foundation
- **AI Chat & Streaming**: Conversational UI with streaming tool invocation cards.
- **Terminal Execution**: CMD & PowerShell streaming executors with timeout control and working directory bounds.
- **SQLite Persistence & Checkpoints**: Checkpoint rollback with crash resilience.
- **Emergency Stop**: Global kill switch halting all child processes and loops.

---

## 3. System Prerequisites

- **Operating System**: Windows 10 or Windows 11 (64-bit)
- **Rust Toolchain**: Rust 1.80+ with MSVC build tools (`cargo`, `rustc`)
- **Node.js**: Node 18+ with `pnpm` (`pnpm install -g pnpm`)
- **Python**: Python 3.14+ (Required for Windows-MCP; install via `uv python install 3.14` or python.org)
- **uv**: Astral `uv` package manager (`irm https://astral.sh/uv/install.ps1 | iex`)
- **Playwright**: `@playwright/mcp` (Resolved dynamically via `npx -y @playwright/mcp`)
- **WebView2**: Microsoft Edge WebView2 Runtime (pre-installed on modern Windows 10/11)

---

## 4. Quickstart Guide

### Step 1 — Automated Environment Setup
```powershell
.\scripts\setup.ps1
```

### Step 2 — Run the Test Suite (All 66 Tests)
```powershell
.\scripts\test.ps1
```

### Step 3 — Launch in Development Mode
```powershell
.\scripts\dev.ps1
```

### Step 4 — Build the Production Release Binary
```powershell
.\scripts\build.ps1
```
The output binary will be created at: `target\release\desktop.exe` (~17.4 MB).

---

## 5. Acceptance Test Verification (66 / 66 Passed)

### Phase 3 Acceptance Tests (18 / 18 Passed)
| Test Identifier | Capability Tested | Status |
| :--- | :--- | :--- |
| `test_p3_01_windows_mcp_connection` | Windows-MCP discovery, Python 3.14 runtime & stdio JSON-RPC | **PASS** |
| `test_p3_02_desktop_observation_no_screenshot` | Desktop observation via accessibility tree without screenshot overhead | **PASS** |
| `test_p3_03_notepad_automation_roundtrip` | Windows UI Automation text input, save dialog, and disk verification | **PASS** |
| `test_p3_04_window_switching_and_focus` | Window enumeration, targeting by title/PID, and foreground focus | **PASS** |
| `test_p3_05_browser_navigation_and_dom_snapshot` | Playwright navigation, compact DOM tree extraction, and link indexing | **PASS** |
| `test_p3_06_form_filling_and_submission` | Form field identification, keyboard typing, submission, and validation | **PASS** |
| `test_p3_07_file_download_workflow` | Browser file download tracking, destination path check, and SHA-256 hash | **PASS** |
| `test_p3_08_file_upload_workflow` | File selection, input element population, upload execution, and response parsing | **PASS** |
| `test_p3_09_hybrid_desktop_browser_workflow` | Hybrid pipeline: Web fetch -> Disk persistence -> Desktop application view | **PASS** |
| `test_p3_10_tool_strategy_router` | 6-tier Tool Router choosing lowest-cost route | **PASS** |
| `test_p3_11_desktop_ownership_concurrency` | Single-controller lease management, concurrent lock denial, and lease renewal | **PASS** |
| `test_p3_12_desktop_emergency_stop` | Instant emergency interrupt aborting active control lease in < 5ms | **PASS** |
| `test_p3_13_browser_action_permissions` | Permission boundary enforcement for destructive web actions | **PASS** |
| `test_p3_14_token_efficiency_benchmark` | DOM accessibility pruning benchmark (>90% token reduction vs vision) | **PASS** |
| `test_p3_15_agent_crash_recovery` | Browser session persistence and checkpoint resume after process kill | **PASS** |
| `test_p3_16_locked_desktop_handling` | Detection of Windows workstation lock (`WM_WTSSESSION_CHANGE`) and safe pause | **PASS** |
| `test_p3_17_open_source_licenses` | Comprehensive SPDX verification of MIT and Apache-2.0 dependencies | **PASS** |
| `test_p3_18_no_regression_phase1_phase2` | Zero-regression verification across Phase 1 and Phase 2 pipelines | **PASS** |

### Phase 2 Acceptance Tests (12 / 12 Passed)
All 12 Phase 2 coding agent acceptance tests pass cleanly.

### Phase 1 Acceptance Tests (9 / 9 Passed)
All 9 Phase 1 acceptance tests pass cleanly with zero regressions.

---

## 6. Open Source Compliance & Attribution

This project strictly adheres to open-source licensing principles:
- **Playwright MCP**: Copyright (c) Microsoft Corporation under the Apache License 2.0.
- **Windows-MCP**: Copyright (c) 2024-2026 Jeomon George (CursorTouch) under the MIT License.
- **OpenCode**: Copyright (c) 2025 opencode under the MIT License. Architectural reference for exact-match mutation, patch rollback, and session compaction.
- **Tauri 2**: The Tauri Programme within The Commons Conservancy under MIT / Apache-2.0.
- All third-party notices, license texts, and dependency inventories are maintained in [THIRD_PARTY_NOTICES.md](file:///THIRD_PARTY_NOTICES.md) and [docs/OPEN_SOURCE_INVENTORY.md](file:///docs/OPEN_SOURCE_INVENTORY.md).
