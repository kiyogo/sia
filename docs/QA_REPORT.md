# Quality Assurance & Acceptance Test Report (Phase 1)

## 1. Test Execution Summary

- **Target OS**: Windows 11 (x86_64-pc-windows-msvc)
- **Rust Toolchain**: rustc 1.99.0 / cargo 1.99.0
- **Node Toolchain**: Node v24.11.1 / pnpm 12.10.1
- **Python Prerequisites**: Python 3.14.6 / uv 0.11.14
- **Test Date**: 2026-10-21
- **Overall Status**: **10 / 10 PASS (100% Verified)**

---

## 2. Acceptance Scenarios Matrix

| Scenario | Objective | Execution Command / Test Case | Actual Outcome | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Test 1 — Chat & Streaming** | Verify conversational exchange, streaming event broadcasting, and SQLite message persistence. | `test_acceptance_1_chat_and_persistence` in `crates/agent-runtime/tests/acceptance_test.rs` | Messages and conversation records successfully stored and retrieved with exact roles (`user`, `assistant`), timestamps, and token counts. | **PASS** |
| **Test 2 — Terminal Execution** | Verify native CMD and PowerShell execution, exit codes, and output streaming without simulated mock terminals. | `test_acceptance_2_terminal_execution` in `crates/agent-runtime/tests/acceptance_test.rs` | CMD (`echo CMD_SUCCESS`) returned exit code 0; PowerShell (`Write-Output 'POWERSHELL_SUCCESS'`) returned exit code 0 with captured stdout and measured duration. | **PASS** |
| **Test 3 — Real Coding Flow** | Generate a real project in a test folder, compile/check it, and verify output. | `test_acceptance_3_coding_project` in `crates/agent-runtime/tests/acceptance_test.rs` | Created `Cargo.toml` and `src/main.rs`, executed `cargo check` inside the generated project with exit code 0, and applied a unified diff patch. | **PASS** |
| **Test 4 — Windows-MCP** | Verify Windows-MCP connection, prerequisite diagnostics (Python 3.14+, uv), and graceful status reporting. | `test_acceptance_4_windows_mcp_prerequisites` in `crates/agent-runtime/tests/acceptance_test.rs` | Detected Python 3.14+ (3.14.6) and uv (0.11.14); verified dynamic tool discovery and validation error messages when prerequisites are absent. | **PASS** |
| **Test 5 — Permission Gateway** | Ensure file modifications outside authorized workspace root are blocked or routed to human approval. | `test_acceptance_5_permission_boundary` in `crates/agent-runtime/tests/acceptance_test.rs` | Scoped file operation permitted; outside access (`C:\Windows\System32\...`) intercepted and produced `SecurityError::ApprovalRequired`. | **PASS** |
| **Test 6 — Persistence Across Restart** | Verify database retains conversations, tasks, and checkpoints after process shutdown. | `test_acceptance_6_persistence_across_restart` in `crates/agent-runtime/tests/acceptance_test.rs` | Created task and checkpoint in Session 1, closed DB connection, reopened DB in Session 2; verified all records, status, and summaries remained intact. | **PASS** |
| **Test 7 — Safe Checkpoint Recovery** | Recover interrupted tasks from checkpoint, transitioning to `Paused` for human inspection instead of blind replay. | `test_acceptance_7_safe_checkpoint_recovery` in `crates/agent-runtime/tests/acceptance_test.rs` | Simulated interrupted task at Step 2; recovery safely converted status from `Running` to `Paused` with checkpoint milestone noted. | **PASS** |
| **Test 8 — Token Optimization** | Validate tool output truncation and long conversation compaction. | `test_acceptance_8_token_optimization` in `crates/agent-runtime/tests/acceptance_test.rs` | 50-line output truncated to 10 lines with pagination notice; 10-turn history compacted into structured summary. | **PASS** |
| **Test 9 — Desktop Build & Artifacts** | Verify build of Windows executable and frontend assets. | `cargo build -p desktop` & `pnpm run build` | Produced production frontend bundle in `apps/desktop/dist` and 23.6 MB native PE binary `target/debug/desktop.exe`. | **PASS** |
| **Test 10 — Open Source Compliance** | Comprehensive inventory and license notices for all third-party components. | Automated check of `THIRD_PARTY_NOTICES.md` & `docs/OPEN_SOURCE_INVENTORY.md` | All integrated libraries (Windows-MCP, OpenCode, Tauri, Tokio, Rusqlite) have full attribution, MIT/Apache-2.0 compatibility, and no copyleft contamination. | **PASS** |

---

## 3. Unit Test Results by Crate

```text
running 4 tests (agent-core)
test token_optimizer::tests::test_token_estimation ... ok
test token_optimizer::tests::test_history_compaction ... ok
test token_optimizer::tests::test_tool_output_truncation ... ok
test tests::test_agent_loop_with_mock_and_tools ... ok
test result: ok. 4 passed; 0 failed

running 4 tests (agent-platform)
test tests::test_filesystem_boundary ... ok
test tests::test_os_adapter ... ok
test tests::test_shell_executor_cmd ... ok
test tests::test_shell_executor_powershell ... ok
test result: ok. 4 passed; 0 failed

running 2 tests (agent-runtime)
test tests::test_recovery_from_checkpoints ... ok
test tests::test_runtime_host_lifecycle ... ok
test result: ok. 2 passed; 0 failed

running 2 tests (agent-security)
test tests::test_emergency_stop ... ok
test tests::test_security_tiers_and_approval ... ok
test result: ok. 2 passed; 0 failed

running 2 tests (agent-storage)
test tests::test_tasks_and_checkpoints ... ok
test tests::test_database_conversations_and_messages ... ok
test result: ok. 2 passed; 0 failed

running 2 tests (agent-tools)
test tests::test_filesystem_tools_crud ... ok
test tests::test_terminal_tool_real_execution ... ok
test result: ok. 2 passed; 0 failed
```
