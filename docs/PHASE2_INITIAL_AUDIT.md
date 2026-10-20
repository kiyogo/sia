# Phase 2 Initial System Audit & Heritage Assessment

## 1. Executive Summary
- **Target**: Universal Autonomous AI Workspace (`D:\DATA\source\AIcode\aai`).
- **Audit Date**: 2026-10-21.
- **Auditor**: Principal Systems Architect & Senior Rust Engineer.
- **Phase 1 Verification**: 100% verified. All 9 acceptance integration tests and crate unit tests pass cleanly without errors.
- **Phase 2 Mission**: Upgrade the existing Desktop AI Workspace with an **Advanced Autonomous Coding Agent**, integrating proven OpenCode architecture, multi-language tooling, compiler diagnostics, token-efficient repo mapping, and Git protection.

---

## 2. Inventory of Phase 1 Crates & Components

### 2.1 Backend Crates (Rust Workspace)
| Crate | Responsibility | Current State | Phase 2 Reuse & Extension |
|---|---|---|---|
| `crates/agent-platform` | OS abstractions, shell executors, filesystem boundaries | Fully operational (Windows CMD + PowerShell, stub for Linux/macOS) | Extend `LanguageToolingAdapter` & `CodingEngineAdapter` |
| `crates/agent-security` | `PermissionEngine`, scoped path sandbox, emergency stop, approval tiers | Fully operational (Strict, AskDangerous, FullAuto) | Add Git working-tree safety rules (block destructive resets) |
| `crates/agent-storage` | SQLite database, schema migrations, conversation/message/task/checkpoint store | Fully operational with thread-safe `Arc<Database>` | Add code diagnostics & plan steps storage |
| `crates/agent-tools` | Terminal tool, filesystem tools (`read`, `write`, `patch`, `search`, `list`), Windows-MCP manager | Fully operational | Add Git tools (`git_status`, `git_diff`), code search (`ripgrep`), repo map, build/test runner |
| `crates/agent-core` | Agent loop, model providers (OpenAI-compatible & Mock), token optimizer | Fully operational (loop with tool dispatch, context budget) | Add Coding Modes (`Ask`, `Plan`, `Build`), compiler diagnostic parser, error deduplicator |
| `crates/agent-runtime` | `RuntimeHost`, task scheduler, checkpoint recovery, event emitter | Fully operational (9 acceptance tests pass) | Add autonomous coding workflow runner |

### 2.2 Desktop Application (`apps/desktop`)
| Component | Responsibility | Current State | Phase 2 Upgrades |
|---|---|---|---|
| `apps/desktop/src-tauri` | Tauri 2 backend host bridging IPC commands and events | Fully operational | Expose coding agent commands: `run_build`, `run_test`, `get_diff`, `get_repo_map` |
| `apps/desktop/src` | React 18 + Vite + Tailwind CSS UI | Fully operational (Dashboard, Chat, Tasks, Workspace, Terminal, Settings, Approvals) | Add Coding Workspace tab with Code Viewer, Diff Inspector, Diagnostics Panel, Mode Selector |

---

## 3. Backwards Compatibility Guarantees
1. **AI Chat & Persistence**: All existing SQLite tables (`conversations`, `messages`, `tasks`, `checkpoints`, `tool_audits`) remain intact with zero schema breakage.
2. **Windows-MCP Control**: Windows-MCP integration and desktop accessibility tools continue to operate without interference.
3. **Terminal & Shell Execution**: CMD and PowerShell streaming executors remain the bedrock for native execution.
4. **Sandbox Integrity**: Any file modification from the Coding Agent must respect `SecurityEngine` boundary checking.
5. **No Regressions**: Existing 9 acceptance tests in `crates/agent-runtime/tests/acceptance_test.rs` must continue to pass 100%.

---

## 4. Technical Gaps & Required Phase 2 Enhancements
1. **Language Intelligence**:
   - Automated detection of project language (Rust, TS/JS, Python, C/C++), build system (`cargo`, `npm`/`pnpm`/`yarn`, `uv`/`pytest`), and test runners.
2. **Context & Repo Mapping**:
   - Compact hierarchical repo map respecting `.gitignore` and token ceilings (preventing context blowout).
3. **Safe Multi-File Patching**:
   - Atomic multi-file changes with unified diff preview and automatic rollback on failure.
4. **Compiler Diagnostics & Self-Correction**:
   - Parsers for `rustc`, `tsc`, `python`, and test runner outputs to extract root cause errors and deduplicate cascading errors.
5. **UI Coding Workspace**:
   - Interactive visual diff viewer and diagnostics panel in the Tauri desktop app.
