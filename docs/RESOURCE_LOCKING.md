# Resource Locking & Concurrency Control Specification

## 1. Overview
The `ResourceLockManager` (`agent-core/src/orchestration/resource_lock.rs`) prevents race conditions, conflicting desktop inputs, and lost file updates across concurrent workers.

---

## 2. Lock Domains

### 2.1 Desktop Interactive Session Lock
- **Single Controller Invariant**: Only one agent can control interactive Windows input (mouse, keyboard, foreground window switching) at any moment.
- **Resource ID**: `"desktop:primary"`.
- **Duration**: Default lease of 30 seconds (configurable, max 120s) with atomic fencing tokens.
- **Concurrency Guard**: Subsequent attempts by another agent receive `ResourceLockError::ResourceBusy`.

### 2.2 Workspace & File Mutation Locks
- **File-Level Granularity**: Before modifying a file (e.g. `CodeEditTool` or `CodePatchTool`), the worker must acquire a lock on the target file path.
- **Conflict Detection**: If two coding agents attempt to patch the same file concurrently, the second agent is blocked until the first completes and unlocks.

### 2.3 Browser Context Isolation
- Parallel browser operations execute in isolated browser contexts or tabs, preventing session and cookie crosstalk.

---

## 3. Fencing Tokens
To prevent zombie workers (whose lease timed out during network or compiler lag) from committing out-of-order changes, every lock acquisition yields a strictly monotonic integer `fencing_token`. Disk operations and lease releases verify this token; stale tokens are rejected immediately with `ResourceLockError::StaleFencingToken`.
