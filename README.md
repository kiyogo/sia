# Universal Autonomous AI Workspace
### Windows-First, Cross-Platform-Ready, Open-Source-First Desktop Agent Platform

An autonomous, multi-purpose AI Workspace designed to perform real work on Windows. The platform bridges natural language instructions from users with native system capabilities: executing terminal commands in CMD/PowerShell, analyzing and modifying codebases across multiple languages, integrating Windows-MCP desktop automation, persisting task milestones in SQLite, and optimizing token consumption.

---

## 1. System Vision & Architecture

The application is structured as a multi-crate Rust workspace with a desktop UI built using Tauri 2, React 18, TypeScript, and Tailwind CSS.

```text
aai/
├── apps/
│   └── desktop/                  # Tauri 2 + React + TypeScript + Tailwind UI
│       ├── src/                  # React UI components (CodingWorkspace, Dashboard, Chat, Tasks, Terminal, Settings)
│       └── src-tauri/            # Tauri 2 host binary & IPC command handlers
├── crates/
│   ├── agent-platform/           # OS-agnostic traits, Windows adapters & MultiLanguageAdapter
│   ├── agent-security/           # Permission tiers, boundary sandbox, dangerous command filters, Emergency Stop
│   ├── agent-storage/            # SQLite embedded database with WAL mode and schema migrations
│   ├── agent-tools/              # Git tools, OpenCode code_edit, code_patch, search, repo_map, build & test runners
│   ├── agent-core/               # Agent loop, CodingWorkflowEngine (Ask/Plan/Build), and Token Optimizer
│   └── agent-runtime/            # Task runner, checkpoint recovery engine, and broadcast event streaming
├── docs/                         # Architecture, Coding Agent, Code Intelligence, Git Safety, Security, QA Reports
├── scripts/                      # PowerShell setup, dev, test, and build automation scripts
├── LICENSE                       # MIT License
├── THIRD_PARTY_NOTICES.md        # Open-source attributions (OpenCode, Windows-MCP, Tauri, etc.)
└── README.md
```

---

## 2. Core Features (Phase 1 & Phase 2)

### 2.1 Phase 2: Advanced Autonomous Coding Agent
1. **Multi-Language Intelligence**:
   - Automated project discovery for **Rust** (`Cargo.toml`), **TypeScript / JavaScript** (`tsconfig.json`, `package.json`), **Python** (`pyproject.toml`, `requirements.txt`), and **C/C++** (`CMakeLists.txt`).
   - Identifies package managers (`cargo`, `npm`, `pnpm`, `yarn`, `bun`, `uv`, `pytest`) and project entry points.
2. **OpenCode Architectural Integration**:
   - `CodeEditTool`: Exact-match string replacement with single-match uniqueness enforcement, quote normalization, and unified diff preview.
   - `CodePatchTool`: Atomic multi-file editing with pre-mutation snapshotting and 100% clean rollback on failure.
   - `RepoMapTool`: Compact, AST-aware architectural mapping adhering to strict character/token ceilings.
   - `CodeSearchTool` & `FileOutlineTool`: Fast ripgrep-like walk and symbol outline extraction.
3. **Compiler Diagnostics & Self-Correction**:
   - Multi-language diagnostic parsers for `rustc` (JSON & human stderr), TypeScript (`tsc`), and Python tracebacks.
   - Diagnostic deduplicator isolates root cause errors and suppresses cascading noise, saving up to 85% of context tokens.
   - Closed-loop autonomous repair: Edit -> Compile -> Deduplicate -> Fix -> Re-test.
4. **Git Safety & Working Tree Protection**:
   - Read-only queries (`git_status`, `git_diff`, `git_log`).
   - Hard blocks on destructive operations (`git reset --hard`, `git clean -f`, force push) without explicit human confirmation.
5. **Three Coding Modes**:
   - `Ask`: Read-only comprehension, code tracing, and architecture explanation.
   - `Plan`: Phased implementation plan drafting and diff preview without disk changes.
   - `Build`: End-to-end autonomous execution with compiler verification and self-correction.
6. **Desktop Coding Workspace**:
   - Multi-tab developer UI: Source Viewer, Unified Diff Inspector, Step Progression, and Problems & Diagnostics panel.

### 2.2 Phase 1: General Agent Foundation
- **AI Chat & Streaming**: Conversational UI with tool invocation visual cards.
- **Terminal Execution**: CMD & PowerShell streaming executors with timeout control.
- **Windows-MCP Automation**: Model Context Protocol client for native desktop automation.
- **SQLite Persistence & Checkpoints**: Automatic milestone checkpointing with safe crash recovery.
- **Emergency Stop**: Instant global kill switch halting all child processes and loops.

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
```powershell
.\scripts\setup.ps1
```

### Step 2 — Run the Test Suite (All 44 Tests)
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
The output binary will be created at: `target\release\desktop.exe`.

---

## 5. Acceptance Test Verification

### Phase 2 Acceptance Tests (12 / 12 Passed)
| Test Identifier | Capability Tested | Status |
| :--- | :--- | :--- |
| `test_p2_01_project_discovery` | Multi-language project discovery (Rust, TS, Python, Git) | **PASS** |
| `test_p2_02_compact_repo_map` | Token-budgeted compact repo map excluding build caches | **PASS** |
| `test_p2_03_symbol_search_and_outline` | AST-aware symbol search and file outline extraction | **PASS** |
| `test_p2_04_git_safety_and_diff` | Git working tree query without destructive commands | **PASS** |
| `test_p2_05_opencode_exact_edit` | OpenCode exact-match editing & quote normalization | **PASS** |
| `test_p2_06_atomic_patch_rollback` | Multi-file atomic patch with clean rollback on error | **PASS** |
| `test_p2_07_rust_compiler_diagnostics` | Rustc JSON and human error diagnostic parser | **PASS** |
| `test_p2_08_ts_and_python_diagnostics` | TypeScript tsc and Python traceback diagnostic parsers | **PASS** |
| `test_p2_09_token_optimization_dedup` | Diagnostic deduplication and compact prompt formatting | **PASS** |
| `test_p2_10_coding_modes_isolation` | Tool isolation across Ask, Plan, and Build modes | **PASS** |
| `test_p2_11_scoped_workspace_sandbox` | Scoped path boundary enforcement | **PASS** |
| `test_p2_12_autonomous_self_correction` | Closed-loop autonomous error diagnosis and repair | **PASS** |

### Phase 1 Regression Tests (9 / 9 Passed)
All 9 Phase 1 acceptance tests continue to pass with 100% success rate. Total test suite: 44 automated tests passing.

---

## 6. Open Source Compliance & Attribution

This project strictly adheres to open-source licensing principles:
- **OpenCode**: Copyright (c) 2025 opencode under the MIT License. Architectural reference for exact-match mutation, patch rollback, and session compaction.
- **Windows-MCP**: Copyright (c) 2024-2026 Jeomon George (CursorTouch) under the MIT License.
- **Tauri 2**: The Tauri Programme within The Commons Conservancy under MIT / Apache-2.0.
- All third-party notices, license texts, and dependency inventories are maintained in [THIRD_PARTY_NOTICES.md](file:///THIRD_PARTY_NOTICES.md) and [docs/OPEN_SOURCE_INVENTORY.md](file:///docs/OPEN_SOURCE_INVENTORY.md).
