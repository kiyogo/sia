# Universal Autonomous AI Workspace
### Windows-First, Cross-Platform-Ready, Open-Source-First Desktop Agent Platform

An autonomous, multi-purpose AI Workspace designed to perform real work on Windows. The platform bridges natural language instructions from users with native system capabilities: executing terminal commands in CMD/PowerShell, reading and modifying source code, integrating Windows-MCP desktop automation, persisting task milestones in SQLite, and optimizing token consumption.

---

## 1. System Vision & Architecture

The application is structured as a multi-crate Rust workspace with a desktop UI built using Tauri 2, React 18, TypeScript, and Tailwind CSS.

```text
aai/
├── apps/
│   └── desktop/                  # Tauri 2 + React + TypeScript + Tailwind UI
│       ├── src/                  # React UI components (Dashboard, Chat, Tasks, Workspace, Terminal, Settings)
│       └── src-tauri/            # Tauri 2 host binary & IPC command handlers
├── crates/
│   ├── agent-platform/           # OS-agnostic traits (OS, Shell, FS, Desktop, CodingEngine) & Windows implementations
│   ├── agent-security/           # Permission tiers, boundary sandbox, dangerous command filters, Emergency Stop
│   ├── agent-storage/            # SQLite embedded database with WAL mode and schema migrations
│   ├── agent-tools/              # Real tools: CMD, PowerShell, file CRUD, patch diffing, and standard MCP client
│   ├── agent-core/               # Agent loop, model providers (OpenAI-compatible / mock), and Token Optimizer
│   └── agent-runtime/            # Task runner, checkpoint recovery engine, and broadcast event streaming
├── docs/                         # Architecture, Security, Roadmap, QA, Token Optimization, Open-Source Inventory
├── scripts/                      # PowerShell setup, dev, test, and build automation scripts
├── LICENSE                       # MIT License
├── THIRD_PARTY_NOTICES.md        # Open-source attributions (Windows-MCP, OpenCode, Tauri, etc.)
└── README.md
```

---

## 2. Core Features (Phase 1)

1. **AI Chat & Real-Time Event Streaming**:
   - Conversational interface with multi-session management.
   - Live visual cards for tool executions (tool name, parameters, exit codes, execution durations, stdout).
   - Cancellation control allowing the operator to stop generation at any time.

2. **Native Terminal Execution**:
   - Supports Windows CMD and PowerShell natively with output streaming and exit code capture.
   - Process tree termination and timeout controls to prevent hanging commands.

3. **Workspace Filesystem Tools**:
   - Targeted file reading with line offset/limit pagination.
   - Atomic file writes and directory scaffolding.
   - Recursive directory tree inspection and full-text content searching.
   - Search-and-replace patching with unified diff generation.

4. **Windows-MCP Native Desktop Control**:
   - Integrates [Windows-MCP](https://github.com/CursorTouch/Windows-MCP) via an out-of-process Model Context Protocol (MCP) client over stdio JSON-RPC 2.0.
   - Pre-flight diagnostic checking Python 3.14+ and `uv` prerequisites.
   - Dynamic tool discovery (`tools/list`) and execution (`tools/call`).

5. **Token Optimization Engine**:
   - Context budget partitioning across system prompts, history, tool outputs, and generation reserves.
   - Automatic tool output truncation (configurable line/byte limits) with pagination tips.
   - Conversation compaction that condenses older turns into structured milestone summaries.
   - Accurate token usage tracking and cost estimation.

6. **Security & Human-in-the-Loop Approval**:
   - Scoped workspace sandbox preventing out-of-boundary file tampering.
   - Destructive command filters (`rmdir /s`, `del /f /s`, `format`, `shutdown`).
   - Approval dialog requiring manual confirmation for dangerous operations.
   - Instant global Emergency Stop kill switch.

7. **Durability & Crash Recovery**:
   - All conversations, messages, tasks, checkpoints, and tool audit logs are stored in SQLite.
   - Safe restart scan transitions interrupted tasks to `Paused` for inspection rather than blindly re-running hazardous side-effects.

---

## 3. System Prerequisites

- **Operating System**: Windows 10 or Windows 11 (64-bit)
- **Rust Toolchain**: Rust 1.80+ with MSVC build tools (`cargo`, `rustc`)
- **Node.js**: Node 18+ with `pnpm` (`pnpm install -g pnpm`)
- **Python**: Python 3.14+ (Required for Windows-MCP; install via `uv python install 3.14` or python.org)
- **uv**: Astral `uv` package manager (`irm https://astral.sh/uv/install.ps1 | iex`)
- **WebView2**: Microsoft Edge WebView2 Runtime (pre-installed on modern Windows 10/11)

---

## 4. Quickstart Guide

### Step 1 — Automated Environment Setup
Run the setup script from PowerShell to verify prerequisites and install dependencies:
```powershell
.\scripts\setup.ps1
```

### Step 2 — Run the Test Suite
Execute the full unit test and acceptance test suite:
```powershell
.\scripts\test.ps1
```

### Step 3 — Launch in Development Mode
Launch the application with live reload:
```powershell
.\scripts\dev.ps1
```

### Step 4 — Build the Production Release Binary
Compile the standalone Windows executable:
```powershell
.\scripts\build.ps1
```
The output binary will be created at: `target\release\desktop.exe`.

---

## 5. Acceptance Test Verification

All 10 required acceptance test scenarios have been implemented and verified with automated test suites:

| Scenario | Objective | Status | Evidence |
| :--- | :--- | :--- | :--- |
| **Test 1 — Chat** | Live streaming & SQLite message persistence | **PASS** | `test_acceptance_1_chat_and_persistence` |
| **Test 2 — Terminal** | Real CMD and PowerShell execution & exit codes | **PASS** | `test_acceptance_2_terminal_execution` |
| **Test 3 — Coding** | Real project creation, compile check, and file patching | **PASS** | `test_acceptance_3_coding_project` |
| **Test 4 — Windows-MCP** | Python 3.14+ & uv diagnostic, dynamic tool discovery | **PASS** | `test_acceptance_4_windows_mcp_prerequisites` |
| **Test 5 — Permission** | Boundary sandboxing & approval requirement | **PASS** | `test_acceptance_5_permission_boundary` |
| **Test 6 — Persistence** | SQLite state retention across process restart | **PASS** | `test_acceptance_6_persistence_across_restart` |
| **Test 7 — Recovery** | Safe checkpoint recovery without hazardous replay | **PASS** | `test_acceptance_7_safe_checkpoint_recovery` |
| **Test 8 — Token Opt** | Output truncation and conversation compaction | **PASS** | `test_acceptance_8_token_optimization` |
| **Test 9 — Desktop Build** | Compiles Windows PE binary (`desktop.exe`) | **PASS** | `cargo build -p desktop` & `pnpm run build` |
| **Test 10 — Compliance**| Complete inventory and attribution of third-party code | **PASS** | `THIRD_PARTY_NOTICES.md` & `docs/OPEN_SOURCE_INVENTORY.md` |

---

## 6. Open Source Compliance & Attribution

This project strictly adheres to open-source licensing principles:
- **Windows-MCP**: Copyright (c) 2024-2026 Jeomon George (CursorTouch) under the MIT License. Connected via an out-of-process stdio client.
- **OpenCode**: Copyright (c) 2025 opencode under the MIT License. Researched as architectural reference for the `CodingEngineAdapter` trait.
- **Tauri 2**: The Tauri Programme within The Commons Conservancy under MIT / Apache-2.0.
- All third-party notices, license texts, and dependency inventories are maintained in [THIRD_PARTY_NOTICES.md](file:///THIRD_PARTY_NOTICES.md) and [docs/OPEN_SOURCE_INVENTORY.md](file:///docs/OPEN_SOURCE_INVENTORY.md).

---

## 7. Next Steps for Phase 2

1. Implement `CodingEngineAdapter` utilizing OpenCode CLI/SDK for automated pull request workflows.
2. Integrate local quantized LLMs (GGUF via `candle` / DirectML).
3. Connect Playwright MCP for web navigation and data extraction.
4. Expand `OperatingSystemAdapter` and `ShellExecutor` to Linux and macOS.
