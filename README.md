# Universal Autonomous AI Workspace
### Windows-First, Cross-Platform-Ready, Open-Source-First Desktop Agent Platform

An autonomous, multi-purpose AI Workspace designed to perform real work on Windows. The platform bridges natural language instructions from users with native system capabilities: executing continuous 24/7 background automations as a Windows Service, recovering reliably from software crashes and power interruptions, scheduling recurring and cron-driven workflows, executing terminal commands in CMD/PowerShell, analyzing and modifying codebases across multiple languages, controlling Windows applications, navigating modern web browsers via Playwright MCP, orchestrating complex Multi-Agent Task DAGs, conducting systematic evidence-grounded research, processing multi-format documents, performing deterministic data analytics with responsive vector charts, and automating business communications through an approval-gated email and calendar engine.

---

## 1. System Vision & Architecture

The application is structured as a multi-crate Rust workspace with a desktop UI built using Tauri 2, React 18, TypeScript, and Tailwind CSS.

```text
aai/
├── apps/
│   └── desktop/                  # Tauri 2 + React + TypeScript + Tailwind UI
│       ├── src/                  # React UI (RuntimeControlCenter, ResearchWorkspace, BusinessDashboard, WorkflowOrchestrator, ComputerControl, BrowserWorkspace, CodingWorkspace, Chat, Tasks, Terminal)
│       └── src-tauri/            # Tauri 2 host binary & IPC command handlers
├── crates/
│   ├── agent-platform/           # SCM Service Adapter, InteractiveSessionProvider, SystemResourceMonitor, Win32 adapters & MultiLanguageAdapter
│   ├── agent-security/           # Permission tiers, DataClassification, SecretRedactor, PromptInjectionDefense, DesktopOwnershipManager
│   ├── agent-storage/            # SQLite embedded database with WAL mode, online backup API, RuntimeStore, WorkflowStore & BusinessStore
│   ├── agent-tools/              # Browser MCP, Windows MCP, Git, OpenCode code_edit/patch, search, repo_map
│   ├── agent-core/               # BudgetGovernanceEngine, ErrorClassifier, CircuitBreaker, Research Engine, Document Intelligence, Data Analysis
│   └── agent-runtime/            # AgentRuntimeService, AutonomousScheduler, ProcessSupervisor (Watchdog), CrashRecoveryEngine, InteractiveWorker
├── docs/                         # Comprehensive architecture, 24/7 runtime, multi-agent, business automation, research, security, and QA reports
├── scripts/                      # PowerShell setup, service management, backup/restore, test, and release build automation scripts
├── LICENSE                       # MIT License
├── THIRD_PARTY_NOTICES.md        # Open-source attributions (Playwright, Windows-MCP, OpenCode, Tauri, windows-service, sysinfo, Zip, etc.)
└── README.md
```

---

## 2. Core Capabilities Across Phases

### 2.1 Phase 6: Autonomous 24/7 Runtime & Self-Recovery System
1. **Three-Layer Decoupled Architecture**:
   - **Layer 1 (Background Service)**: `AgentRuntimeService` running independently of the desktop GUI, managing durable job scheduling, process supervision, and recovery.
   - **Layer 2 (Interactive Worker)**: `WindowsInteractiveWorker` bridging desktop GUI automation to Session 1+, with automatic deferral (`WaitingForInteractiveSession`) when running in Session 0 or when the workstation is locked (`logonui.exe`).
   - **Layer 3 (Desktop UI)**: `RuntimeControlCenter.tsx` providing real-time SCM service lifecycle management, scheduler inspection, watchdog tables, and crash recovery reconciliation.
2. **Windows Service Control Manager (SCM) Integration**:
   - Native Windows service lifecycle (Install, Start, Stop, Restart, Status, Uninstall) via `windows-service` (0.8).
   - Automated SCM recovery policies (`restart/60000/restart/60000/restart/60000`).
   - Automated PowerShell service control scripts in `scripts/`.
3. **Durable Execution Engine & Startup Crash Recovery**:
   - SQLite WAL persistence with monotonic fencing tokens (`retry_count`) eliminating stale worker races.
   - Deterministic startup reconciliation: pure idempotent tasks safely reset to `Ready`, while tasks with external side effects (e.g. emails, commits, payments) are held in `OutcomeUnknown` preventing duplicate executions.
   - Manual human operator reconciliation queue supporting `RetrySafeStep`, `MarkCompleted`, or `Cancel`.
4. **Autonomous Scheduler & Missed-Run Policies**:
   - Interval, daily, weekly, and cron-driven job execution.
   - Missed-run policies (`Skip` preventing backlog floods; `RunOnce` for reliable catch-up).
   - Overlapping job concurrency protection (`JobConcurrencyPolicy::Skip`).
5. **Process Supervision & Watchdog**:
   - Child process monitoring with heartbeat tracking.
   - Crash loop circuit breaker: automatically quarantines processes exceeding 3 crashes within 60 seconds (`ProcessStatus::Quarantined`).
   - Clean process tree termination via `taskkill /T /F`.
6. **Hardware Resource Monitoring & Memory Backpressure**:
   - Real-time CPU, RAM, and free disk sampling via `sysinfo` (0.33).
   - Automated memory backpressure pausing non-essential tasks when utilization exceeds 90%.
   - Disk space exhaustion warnings (< 5GB warning; < 1GB critical).
7. **Budget Governance & Online Backups**:
   - Pre-request token and monetary reservations eliminating concurrent multi-worker budget overruns.
   - Native online SQLite Backup API (`rusqlite::backup::Backup`) with non-blocking snapshots and verified restoration.

### 2.2 Phase 5: Advanced Business & Research Automation
1. **Research Agent & Provenance Engine**:
   - Systematic web and local workspace research querying `LocalWorkspaceSearchProvider` and external search adapters.
   - Source provenance tracking with computed SHA-256 integrity digests, HTTP etags, and credibility scoring.
   - Sentence/offset evidence extraction and automated quantitative contradiction detection (>15% divergence flagged).
   - Generates comprehensive markdown whitepapers with verifiable, bracketed citations (`[1]`, `[2]`).
2. **Document Intelligence Subsystem**:
   - Multi-format parser for Markdown, HTML, plain text, and OpenXML DOCX.
   - Native OpenXML (`.docx`) package generator with standard ECMA-376 structure (`word/document.xml`, `[Content_Types].xml`, `_rels`) using pure Rust `zip` crate.
   - Headless PDF 1.4 binary stream generator with standard xref tables and font dictionaries.
   - Structure-aware chunking preserving paragraph boundaries and heading hierarchy with sliding overlap.
   - Document revision versioning and unified diffing.
3. **Deterministic Data Analysis & Vector Charting**:
   - Auto-detecting CSV/TSV reader and OpenXML XLSX sheet reader.
   - Code-first arithmetic computation: Mean, Median, Sample Standard Deviation, Min, Max, and Tukey IQR Outlier Detection ($Q_1 - 1.5 \times IQR$, $Q_3 + 1.5 \times IQR$).
   - Multi-level categorical group-by aggregations (Sum, Count, Average).
   - Standalone responsive SVG vector chart generation (Bar and Line charts) with dark/light theme support.
4. **Business Connectors & Email Subsystem**:
   - Resilient connector registry with leaky-bucket rate limiting, exponential backoff with jitter, and 3-strike circuit breaking.
   - Thread summarization, tone-matched draft creation, and idempotent dispatch tracking with unique `operation_id` GUIDs.
   - **Strict Human Approval Gate**: Compulsory authorization queue in `business_approvals` preventing any unauthorized outbound emails or external mutations.
   - Calendar availability checking, UTC time normalization, and conflict resolution.
5. **Business Workflow Coordinator**:
   - 5 pre-packaged business templates (`market_research`, `data_reporting`, `email_assistance`, `research_to_document`, `hybrid_pipeline`) orchestrating tasks on Phase 4's durable Task DAG.
6. **Enterprise Security & Token Optimization**:
   - 4-tier data classification (`Public`, `Internal`, `Confidential`, `Restricted`).
   - Deterministic secret redaction replacing API keys, passwords, and tokens with `[REDACTED_SECRET]`.
   - Structural prompt injection containment wrapping external data in `<UNTRUSTED_EXTERNAL_DATA>` blocks.
   - Code-first analytics saving **>98% of tokens** compared to injecting raw datasets into LLMs.
7. **Desktop UI: Research Workspace & Business Dashboard**:
   - **Research Workspace**: Query input, evidence cross-examination table, contradiction badge indicators, and cited markdown report preview.
   - **Business Dashboard**: Template launcher, interactive SVG charts, data summary tables, and one-click Human Approval Queue.

### 2.2 Phase 4: Advanced Multi-Agent Orchestration System
- **Supervisor AI Engine**: Goal decomposition, dependency graphing, and dynamic replanning.
- **Task Directed Acyclic Graph (DAG) Engine**: Strict cycle detection using Kahn's topological sorting algorithm.
- **Agent Registry & Worker Specialization**: Supervisor, Coding (OpenCode), Browser (Playwright MCP), Computer (Windows-MCP / Win32), Research, Document, Data Analysis, and Email workers.
- **Tokio Task Scheduler**: Bounded concurrency (1 to 4 parallel workers) with pause, resume, and cancellation.
- **Resource Locking**: Single-controller desktop lock, file mutation locks, and monotonic fencing tokens.
- **Shared Artifact Store**: Content-addressed artifact storage (SHA-256) passing compact `ArtifactRef`s (**87.3% token savings**).
- **Durable Persistence**: 8 SQLite tables tracking workflows, tasks, dependencies, attempts, artifacts, leases, messages, and token usage.
- **Desktop UI**: Dedicated Multi-Agent Workflow Orchestrator tab (`WorkflowOrchestrator.tsx`).

### 2.3 Phase 3: Advanced Computer & Browser Agent
- **Windows Desktop Control**: Window enumeration, focus, coordinates, and workstation lock status detection.
- **Playwright Browser Automation**: Cross-browser navigation, DOM accessibility inspection, form filling, and file transfer tracking.
- **6-Tier Tool Strategy Router**: Routes through the lowest-cost tier (`Direct API` → `Filesystem` → `CLI` → `DOM` → `UIA` → `Vision`).
- **Safety Framework**: Single-controller lease, user takeover detection, and instant Emergency Stop (< 5ms).

### 2.4 Phase 2: Advanced Autonomous Coding Agent
- **OpenCode Architecture**: `CodeEditTool` exact-match replacement, `CodePatchTool` atomic multi-file rollback.
- **Multi-Language Intelligence**: Rust, TypeScript/JS, Python, and C/C++ diagnostic parsers and symbol maps.
- **Git Safety**: Safe status and diff inspection; destructive commands blocked.
- **Execution Modes**: `Ask` (read-only), `Plan` (diff preview), `Build` (autonomous repair loop).

### 2.5 Phase 1: General Agent Foundation
- Conversational chat with streaming tool cards, terminal executors (CMD & PowerShell), SQLite checkpointing, and global kill switch.

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

### Step 2 — Run the Test Suite (All 121 Tests)
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

## 5. Acceptance Test Verification (160 / 160 Passed)

### Phase 6 Acceptance Tests (34 / 34 Passed)
| Test Identifier | Capability Tested | Status |
| :--- | :--- | :--- |
| `test_p6_01_service_installation` | Windows Service installation into SCM adapter | **PASS** |
| `test_p6_02_service_start_running` | Service start and `Running` state transition | **PASS** |
| `test_p6_03_service_stop_stopped` | Service stop and `Stopped` state transition | **PASS** |
| `test_p6_04_service_restart` | Service restart cycle (`Stopped` -> `Running`) | **PASS** |
| `test_p6_05_service_uninstall` | Service uninstallation to `NotInstalled` | **PASS** |
| `test_p6_06_desktop_ui_close_service_persists` | Headless execution persists when Desktop UI window is closed | **PASS** |
| `test_p6_07_desktop_ui_reconnect_state_recovery` | UI reconnect queries active jobs, executions, and metrics | **PASS** |
| `test_p6_08_durable_task_queue_persistence` | Database drop/reopen verifies queue durability across process death | **PASS** |
| `test_p6_09_startup_crash_recovery_pure_tasks_reset` | Pure/idempotent tasks safely reset to `Ready` on startup reboot | **PASS** |
| `test_p6_10_startup_crash_recovery_side_effects_held` | External side-effect tasks held in `OutcomeUnknown` | **PASS** |
| `test_p6_11_operator_manual_reconciliation` | Operator reconciles `OutcomeUnknown` via `MarkCompleted` or `Retry` | **PASS** |
| `test_p6_12_multi_worker_lease_and_heartbeat` | Worker process registration, heartbeat, and resource recording | **PASS** |
| `test_p6_13_atomic_task_claiming_fencing_token` | Atomic task claiming prevents dual worker execution | **PASS** |
| `test_p6_14_stale_worker_fencing_token_invalidation` | Stale worker token rejected upon concurrent task version mutation | **PASS** |
| `test_p6_15_autonomous_scheduler_interval` | Interval schedule execution and next run timestamp advancement | **PASS** |
| `test_p6_16_autonomous_scheduler_missed_run_skip` | Overdue job with policy `Skip` jumps to future interval without flood | **PASS** |
| `test_p6_17_autonomous_scheduler_missed_run_run_once` | Overdue job with policy `RunOnce` triggers single catch-up run | **PASS** |
| `test_p6_18_overlapping_job_protection` | Job with policy `Skip` skips triggering when previous instance is running | **PASS** |
| `test_p6_19_worker_supervisor_heartbeat_tracking` | Process supervisor tracks healthy worker heartbeats | **PASS** |
| `test_p6_20_watchdog_crash_detection_restart` | Watchdog detects terminated child process and restarts worker | **PASS** |
| `test_p6_21_crash_loop_quarantine` | Circuit breaker quarantines worker after 3 crashes in 60s | **PASS** |
| `test_p6_22_clean_process_tree_termination` | Clean recursive process tree termination (`taskkill /T /F`) | **PASS** |
| `test_p6_23_resource_monitor_cpu_memory` | `SystemResourceMonitor` samples CPU, RAM MB, and memory percent | **PASS** |
| `test_p6_24_resource_monitor_disk_warning` | Disk threshold checking (< 5GB warning; < 1GB critical) | **PASS** |
| `test_p6_25_memory_backpressure_pause` | Memory backpressure detected when RAM utilization > 90% | **PASS** |
| `test_p6_26_pre_request_token_budget_reservation` | Pre-request reservation eliminates parallel multi-agent budget races | **PASS** |
| `test_p6_27_pre_request_cost_budget_reservation_rejection` | Reservation exceeding daily or workflow budget limit is rejected | **PASS** |
| `test_p6_28_budget_reservation_release_on_failure` | Failed/cancelled request releases reservation back to available pool | **PASS** |
| `test_p6_29_online_sqlite_backup_api` | Non-blocking online SQLite backup generates intact backup file | **PASS** |
| `test_p6_30_database_restore_verification` | Restored database preserves all tables, rows, and scheduled jobs | **PASS** |
| `test_p6_31_session0_deferral` | GUI automation in Session 0 defers to `WaitingForInteractiveSession` | **PASS** |
| `test_p6_32_locked_desktop_deferral` | GUI automation defers while desktop locked, executes on unlock | **PASS** |
| `test_p6_33_error_classifier` | Classifies Transient, Permanent, Resource, UserAction, UnknownOutcome | **PASS** |
| `test_p6_34_circuit_breaker_transitions` | Circuit breaker transitions Closed -> Open -> HalfOpen -> Closed | **PASS** |

### Phases 1–5 Acceptance & Unit Tests (126 / 126 Passed)
- **Phase 5 Acceptance**: 24 / 24 Passed
- **Phase 4 Acceptance**: 21 / 21 Passed
- **Phase 3 Acceptance**: 18 / 18 Passed
- **Phase 2 Acceptance**: 12 / 12 Passed
- **Phase 1 Acceptance**: 9 / 9 Passed
- **Workspace Unit Tests (All Crates)**: 42 / 42 Passed

**Total: 160 / 160 Passed (100% Pass Rate). Zero Regressions.**

---

## 6. Open Source Compliance & Attribution

This project strictly adheres to open-source licensing principles:
- **Playwright MCP**: Copyright (c) Microsoft Corporation under the Apache License 2.0.
- **Windows-MCP**: Copyright (c) 2024-2026 Jeomon George (CursorTouch) under the MIT License.
- **OpenCode**: Copyright (c) 2025 opencode under the MIT License. Architectural reference for exact-match mutation, patch rollback, and session compaction.
- **Tauri 2**: The Tauri Programme within The Commons Conservancy under MIT / Apache-2.0.
- **Zip-rs**: Copyright (c) 2014 Mathijs van de Nes under the MIT License.
- All third-party notices, license texts, and dependency inventories are maintained in [THIRD_PARTY_NOTICES.md](file:///THIRD_PARTY_NOTICES.md) and [docs/OPEN_SOURCE_INVENTORY.md](file:///docs/OPEN_SOURCE_INVENTORY.md).
