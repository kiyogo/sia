# Coding Agent Security & Sandboxing Architecture

## 1. Security Architecture Principles

The Coding Agent operates with defense-in-depth principles:
1. **Workspace Containment**: File access is strictly restricted to designated workspace root directories.
2. **Lexical Path Sanitization**: Relative path traversals (`../`) and absolute escape attempts are neutralized before reaching the filesystem.
3. **Multi-Tier Permission Control**: Every tool is classified into explicit permission tiers.
4. **Emergency Stop Protection**: Instant global kill switch halting active executions and terminating child processes.

---

## 2. Scoped Workspace Sandboxing

The `SecurityEngine` validates every target file path through `is_within_boundary`:

```rust
fn is_within_boundary(&self, boundary: &Path, path: &Path) -> bool {
    let b = self.normalize_path(boundary);
    let p = self.normalize_path(path);
    let b_str = b.to_string_lossy().to_lowercase();
    let p_str = p.to_string_lossy().to_lowercase();
    p_str.starts_with(&b_str)
}
```

### Lexical Normalization
To prevent bypasses via uncanonicalized relative components (such as `C:\workspace\..\..\Windows\system32`), `normalize_path` folds `ParentDir` (`..`) and `CurDir` (`.`) lexically before checking boundary containment:
- Strips Windows UNC prefixes (`\\?\`).
- Evaluates directory traversal components without requiring target files to exist on disk.
- Rejects any path where the resolved prefix does not match the active workspace root.

---

## 3. Tool Permission Tiers

| Tool | Assigned Tier | Strict Mode | WorkspaceAuto Mode | FullAuto Mode |
|---|---|---|---|---|
| `project_detect` | `ReadOnly` | Allowed | Allowed | Allowed |
| `repo_map` | `ReadOnly` | Allowed | Allowed | Allowed |
| `code_search` | `ReadOnly` | Allowed | Allowed | Allowed |
| `file_outline` | `ReadOnly` | Allowed | Allowed | Allowed |
| `file_read` | `ReadOnly` | Allowed | Allowed | Allowed |
| `diff_preview` | `ReadOnly` | Allowed | Allowed | Allowed |
| `git_status` | `ReadOnly` | Allowed | Allowed | Allowed |
| `git_log` | `ReadOnly` | Allowed | Allowed | Allowed |
| `git_diff` | `ReadOnly` | Allowed | Allowed | Allowed |
| `code_edit` | `WorkspaceWrite` | Approval Required | Allowed (in-boundary) | Allowed (in-boundary) |
| `code_patch` | `WorkspaceWrite` | Approval Required | Allowed (in-boundary) | Allowed (in-boundary) |
| `git_commit` | `WorkspaceWrite` | Approval Required | Allowed (in-boundary) | Allowed (in-boundary) |
| `run_build` | `ShellExecution` | Approval Required | Approval Required | Allowed (safe builds) |
| `run_test` | `ShellExecution` | Approval Required | Approval Required | Allowed (safe tests) |
| `terminal_execute` | `ShellExecution` | Approval Required | Approval Required | Allowed (safe cmds) |
| Destructive Ops | `DestructiveAction` | **Always Approval** | **Always Approval** | **Always Approval** |

---

## 4. Emergency Stop Integration

- **Trigger**: Activated via UI top banner, keyboard shortcut, or IPC command `trigger_emergency_stop`.
- **Enforcement**: Atomic boolean flag `AtomicBool::store(true, Ordering::SeqCst)`.
- **Response**:
  - Halts the active `AgentLoop` and `CodingWorkflowEngine` immediately at the current step.
  - Aborts child processes spawned by `ShellExecutor`.
  - Rejects any pending tool call execution with `SecurityError::EmergencyStopActive`.
  - Preserves checkpoint database records for post-mortem review.
