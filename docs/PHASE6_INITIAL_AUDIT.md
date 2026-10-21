# Phase 6 Initial Audit: Current Architecture & 24/7 Runtime Readiness
## Universal Autonomous AI Workspace

**Date:** 2026-10-21  
**Auditor:** Principal Autonomous AI Systems Architect & SRE Team  
**Workspace:** `D:\DATA\source\AIcode\aai`  
**Focus:** 24/7 Continuous Execution, Process Lifecycle, Service Isolation, Durability, Self-Recovery

---

### 1. Executive Summary

This audit evaluates the codebase following Phase 5 completion. The system possesses a durable Tokio multi-agent runtime, Kahn-sort Task DAG, SQLite WAL storage with 19 tables, OpenCode mutation tools, Windows-MCP & Playwright MCP adapters, and business/research engines.

However, operating continuously **24/7 independently of the desktop UI** requires fundamental architectural enhancements:
1. Currently, `desktop.exe` hosts both the Tauri WebView2 GUI and the `RuntimeHost` in the **same OS process**. Closing the GUI window terminates the Tokio runtime and any active tasks.
2. The runtime is not registered with the Windows Service Control Manager (SCM), meaning it cannot start at system boot without user login.
3. Windows-MCP and Win32 UI automation currently run within the same host context, which would fail if executed directly inside Windows Service Session 0 (Session 0 Isolation).
4. Task scheduling currently triggers workflows ad-hoc; there is no persistent recurring cron/interval job scheduler surviving application restarts.
5. While checkpoints exist in SQLite, recovery on startup only flags interrupted tasks as Failed or re-plans from scratch, rather than atomically reconciling in-flight side effects and resuming exact execution points.

---

### 2. Comprehensive Architectural Audit Questions (Section II.6)

#### 2.1. Hiện đang chạy nền bằng cơ chế nào? (Current Background Execution Mechanism)
- **Current State:** Tasks execute as Tokio asynchronous tasks (`tokio::spawn`) within the `RuntimeHost` process. There is no OS-level background service (Windows Service / Daemon).
- **Gap:** When the desktop application window is closed by the user, the operating system terminates the process, instantly dropping active network calls and killing subtasks.
- **Phase 6 Target:** Split into a 3-layer execution model:
  - **Layer 1: Background Runtime Service** (`AgentRuntimeService`) running as a Windows Service or headless daemon.
  - **Layer 2: Interactive Worker** (`WindowsInteractiveWorker`) running in user session (Session 1+) for desktop automation.
  - **Layer 3: Desktop UI** (`Tauri Desktop`) acting as a client dashboard connecting to the background runtime.

#### 2.2. UI và Runtime có tách tiến trình thực sự không? (Are UI and Runtime Process-Separated?)
- **Current State:** No. In `apps/desktop/src-tauri/src/main.rs`, Tauri initializes `RuntimeHost` directly inside the Tauri main process.
- **Phase 6 Target:** The runtime host will support running either in-process (development mode) or out-of-process as a standalone background Windows Service / daemon. The Tauri UI interacts with the runtime via IPC or local shared SQLite queue with IPC notification channels.

#### 2.3. Runtime chết thì task được xử lý ra sao? (What happens if Runtime Crashes?)
- **Current State:** Tasks marked as `Running` remain in the database as `Running` indefinitely (orphaned tasks).
- **Phase 6 Target:** Startup reconciliation scans for tasks with expired worker leases, checks if child processes are still running, evaluates whether side-effects occurred, marks indeterminate states as `OutcomeUnknown`, and recovers safe tasks back to `Ready`.

#### 2.4. Máy restart thì task có còn không? (Do Tasks Persist Across OS Reboots?)
- **Current State:** Yes, SQLite persists the records across reboots because WAL mode flushes commits to disk. However, the runtime does NOT automatically resume execution upon reboot because no service is registered to boot on startup.
- **Phase 6 Target:** Windows Service configured with `AutoStart` or `DelayedAutoStart` to initialize upon system boot. The recovery engine will rehydrate pending workflows immediately.

#### 2.5. Có checkpoint thật không? (Are There Real Checkpoints?)
- **Current State:** Yes. `agent-storage::checkpoints` table records task ID, step index, state JSON, and timestamp. Furthermore, Phase 4 added `task_attempts` and Phase 5 added `document_metadata` and `email_drafts`.
- **Phase 6 Target:** Expand checkpointing to record exact tool invocation boundaries, fencing tokens, and external resource states.

#### 2.6. Có thể chạy đồng thời nhiều Worker không? (Can Multiple Workers Run Concurrently?)
- **Current State:** Yes. `WorkflowScheduler` enforces bounded concurrency (default 1 to 4 parallel workers) using Tokio async worker pools.
- **Phase 6 Target:** Expand to multi-process supervision with CPU/RAM-aware backpressure.

#### 2.7. Có chống duplicate execution không? (Is There Duplicate Execution Prevention?)
- **Current State:** Partially. `ResourceLockManager` manages desktop leases and file locks. Phase 5 added unique `operation_id` tracking for emails.
- **Phase 6 Target:** Introduce atomic task claiming (`UPDATE ... WHERE status = 'Ready' AND lease_id IS NULL`), monotonic fencing tokens, and persistent idempotency keys across all side-effect tools.

#### 2.8. Có phân biệt GUI Worker và Background Worker không? (Is There Separation Between GUI & Background Workers?)
- **Current State:** In Phase 4, `AgentRegistry` has agent types (`Computer`, `Browser`, `Coding`, `Research`, `Document`, `DataAnalysis`, `Email`), but they all execute in the current user context.
- **Phase 6 Target:** Strict separation of Session 0 (Headless background service for Coding, Research, Data Analysis, Document generation) and Session 1+ (Interactive Worker for Windows-MCP, Win32 mouse/keyboard). GUI tasks enter `WaitingForInteractiveSession` if the user is logged out or screen is locked.

#### 2.9. Có giới hạn token không? (Is There Token & Budget Enforcement?)
- **Current State:** Yes. `TokenOptimizer` tracks context budgets and estimates costs. Phase 4 added hierarchical workflow budgets.
- **Phase 6 Target:** Introduce **Pre-Request Budget Reservation** to prevent concurrent agents from exceeding the global/daily budget in a race condition.

#### 2.10. Có quản lý retry không? (Is There Retry Management?)
- **Current State:** Basic retry count on task failure in `WorkflowScheduler`.
- **Phase 6 Target:** Structured error classification (`Transient`, `Permanent`, `Resource`, `UserActionRequired`, `UnknownOutcome`) with exponential backoff, jitter, and circuit breaking to prevent crash loops.

#### 2.11. Có kiểm tra health thật không? (Is There Real Health Monitoring?)
- **Current State:** Basic status queries for MCP and prerequisites.
- **Phase 6 Target:** Dedicated `Watchdog` and `HealthMonitor` tracking PID liveness, heartbeats, memory/CPU usage via `sysinfo`, and progress metrics.

---

### 3. Reusable Components vs. Components Requiring Enhancement

| Component | Current State | Phase 6 Action |
|---|---|---|
| `agent-storage::workflow_store` | Robust SQLite tables for workflows, tasks, artifacts | **Enhance**: Add `scheduled_jobs`, `supervised_processes`, `budget_reservations`, `runtime_events`. |
| `agent-storage::Database` | Embedded SQLite with WAL mode | **Enhance**: Add online SQLite backup API (`rusqlite::backup::Backup`). |
| `agent-runtime::WorkflowScheduler` | Tokio async DAG scheduler | **Enhance**: Add durable persistent queue, atomic claiming, fencing, and backpressure. |
| `agent-runtime::SupervisorEngine` | Task decomposition and DAG generation | **Reuse**: Fully operational. |
| `agent-core::ResourceLockManager` | Desktop and file lock leases | **Enhance**: Add session-aware ownership and process fencing tokens. |
| `agent-tools::WindowsMcpManager` | Out-of-process Windows-MCP | **Enhance**: Isolate into Interactive Worker to prevent Session 0 failure. |
| `agent-security` | Permissions, redaction, injection defense | **Reuse**: Fully operational. |
| `apps/desktop` (Tauri UI) | Desktop application | **Enhance**: Add close behavior (minimize to tray / keep service running), and add `RuntimeControlCenter` view. |

---

### 4. Implementation Action Plan

1. **Research & Dependencies**: Integrate `windows-service` (for Windows SCM) and `sysinfo` (for system metrics).
2. **Storage Durability**: Migrate schema to add persistent scheduled jobs, process supervision, budget reservations, and backup mechanisms.
3. **Core Services**:
   - `AgentRuntimeService`: Background daemon / Windows Service.
   - `WindowsInteractiveWorker`: Interactive session isolation.
   - `ProcessSupervisor` & `Watchdog`: Process registry, heartbeats, and circuit-breaker restart policy.
   - `AutonomousScheduler`: Recurring cron/interval engine with timezone and missed-run policies.
   - `CrashRecoveryEngine`: Safe reconciliation of unknown outcomes.
   - `ResourceMonitor`: System metrics and backpressure.
4. **Desktop UI**: `RuntimeControlCenter.tsx` for 24/7 service management, schedules, recovery, and watchdog.
5. **Testing**: 34 comprehensive Phase 6 acceptance tests verifying durability, recovery, and zero regressions.
