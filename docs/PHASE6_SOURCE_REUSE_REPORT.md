# Phase 6 Source Reuse & Open-Source Research Report
## Universal Autonomous AI Workspace

**Date:** 2026-10-21  
**Project:** Universal Autonomous AI Workspace  
**Phase:** 06 — Autonomous 24/7 Runtime  
**Lead Architect:** Principal Autonomous AI Systems Architect & Open-Source Compliance Officer

---

### 1. Open-Source Technology Research & Evaluation

#### 1.1. Windows Service Integration (`windows-service-rs` vs. Custom Win32 FFI)
- **Repository:** https://github.com/mullvad/windows-service-rs
- **License:** MIT / Apache-2.0
- **Evaluation:**
  - `windows-service` is the de facto Rust library for integrating with the Windows Service Control Manager (SCM). Maintained by Mullvad VPN, it provides safe Rust abstractions for `RegisterServiceCtrlHandlerExW`, `StartServiceCtrlDispatcherW`, `SetServiceStatus`, and service installation/deletion via `OpenSCManagerW`.
  - Writing custom Win32 FFI bindings would be redundant, error-prone, and introduce unsafe pointer manipulations.
- **Selection Decision:** Use `windows-service` for SCM registration, service entry points, and status reporting on Windows, while wrapping it behind the cross-platform `RuntimeServiceAdapter` trait.

#### 1.2. Temporal vs. Embedded SQLite Durable DAG
- **Repository:** https://github.com/temporalio/temporal
- **Evaluation:**
  - Temporal is an enterprise distributed orchestrator. However, running Temporal requires a separate Temporal Server cluster (written in Go), an external database (PostgreSQL/Cassandra/MySQL), elasticsearch, and complex network configurations.
  - For a Windows Desktop application and local service running on a user's PC, requiring Docker or a background Go server daemon creates immense user friction, bloat (>500MB RAM), and deployment failure modes.
- **Selection Decision:** Do **NOT** integrate Temporal Server for Phase 6. Instead, enhance our existing embedded SQLite-backed durable scheduler (`agent-storage` + `agent-runtime`) with atomic task claiming, lease fencing, checkpoints, and event journaling. This provides Temporal-like durability guarantees with zero external dependencies, sub-millisecond local latency, and minimal memory footprint.

#### 1.3. System & Resource Monitoring (`sysinfo`)
- **Repository:** https://github.com/GuillaumeGomez/sysinfo
- **License:** MIT
- **Evaluation:**
  - `sysinfo` provides reliable, cross-platform APIs for querying CPU utilization, RAM usage, swap space, disk partition free space, and per-process memory/CPU metrics.
  - Pure Rust with native Windows Win32 / PDH / WMI bindings under the hood.
- **Selection Decision:** Adopt `sysinfo` in `agent-platform` / `agent-runtime` for system resource monitoring, backpressure detection, and process watchdog metrics.

#### 1.4. Async Task Supervision & Coordination (`tokio`)
- **Repository:** https://github.com/tokio-rs/tokio
- **License:** MIT
- **Evaluation:**
  - Tokio's `CancellationToken`, `tokio::time::interval`, `broadcast::channel`, and async process APIs provide all necessary primitives for watchdog heartbeats, graceful shutdown signaling, and non-blocking process management.
- **Selection Decision:** Leverage existing Tokio 1.40 workspace dependency.

---

### 2. Upstream Component Inventory & Attribution

| Dependency | Repository / Source | Version | License | Role in Phase 6 | Attribution & Compliance |
|---|---|---|---|---|---|
| **`windows-service`** | https://github.com/mullvad/windows-service-rs | `0.8.x` | MIT / Apache-2.0 | Windows Service Control Manager integration | Preserved MIT/Apache notices; Windows-specific feature |
| **`sysinfo`** | https://github.com/GuillaumeGomez/sysinfo | `0.33.x` | MIT | Real-time CPU, RAM, disk, and process monitoring | Preserved MIT notice |
| **`tokio`** | https://github.com/tokio-rs/tokio | `1.40` | MIT | Async task orchestration, cancellation tokens, timers | Existing dependency |
| **`rusqlite`** | https://github.com/rusqlite/rusqlite | `0.32` | MIT | Online SQLite backup API and durable tables | Existing dependency |
| **`uuid` / `chrono`** | crates.io | Latest | MIT / Apache-2.0 | Correlation IDs, UTC instants, and scheduling | Existing dependencies |

---

### 3. Reusable Architecture from Phases 1–5

1. **Supervisor Engine & Task DAG (`agent-core::orchestration`)**:
   - Kahn's topological sort and acyclic dependency resolution are 100% reusable.
2. **Resource Lock Manager (`agent-core::ResourceLockManager`)**:
   - Fencing tokens and path locks are extended with session-awareness.
3. **Multi-Agent Registry (`agent-core::AgentRegistry`)**:
   - Agent capability mapping supports both background and interactive workers.
4. **Token Optimizer & Budgets (`agent-core::TokenOptimizer`)**:
   - Extended with multi-tier pre-request reservation to prevent concurrent agent budget races.
5. **Storage Schema (`agent-storage`)**:
   - 19 existing tables retained; new migrations added for jobs, processes, reservations, and events.
