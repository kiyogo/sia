# Open-Source Inventory & Audit Matrix

| Component | Upstream Repository | Version / Commit | License | Usage Mode | Integration Location | Obligations & Attribution |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Playwright MCP** | https://github.com/microsoft/playwright-mcp | @playwright/mcp 0.0.4+ | Apache-2.0 | Out-of-process subprocess via stdio MCP JSON-RPC protocol | `crates/agent-tools/src/browser_mcp.rs` | Preserved Apache-2.0 copyright notice; external process invocation; attribution in `THIRD_PARTY_NOTICES.md` |
| **Windows-MCP** | https://github.com/CursorTouch/Windows-MCP | 0.8.8 / latest release | MIT | Out-of-process subprocess via stdio MCP JSON-RPC protocol | `crates/agent-tools/src/windows_mcp.rs`, `mcp_client.rs` | Preserved MIT copyright notice; external process invocation |
| **OpenCode** | https://github.com/opencode/opencode | v2.x architecture reference | MIT | Architecture reference and native Rust implementation of exact-match editing, atomic patch rollback, compact repo mapping, and session compaction | `crates/agent-tools/src/code_edit.rs`, `code_intelligence.rs`, `crates/agent-core/src/coding_loop.rs` | Preserved MIT notice; attribution in `THIRD_PARTY_NOTICES.md` |
| **Tauri 2** | https://github.com/tauri-apps/tauri | 2.12.x | MIT / Apache-2.0 | Host runtime & WebView2 IPC | `apps/desktop/src-tauri` | Cargo dependency attribution |
| **Tokio** | https://github.com/tokio-rs/tokio | 1.40+ | MIT | Async executor & task supervision | All workspace crates | Cargo dependency attribution |
| **Rusqlite** | https://github.com/rusqlite/rusqlite | 0.32.x | MIT | Embedded SQLite persistent store | `crates/agent-storage` | Cargo dependency attribution |
| **Reqwest** | https://github.com/seanmonstar/reqwest | 0.12.x | MIT / Apache-2.0 | HTTP client for OpenAI-compatible LLM endpoints | `crates/agent-core` | Cargo dependency attribution |
| **Similar** | https://github.com/mitsuhiko/similar | 2.6+ | Apache-2.0 | Unified diff generator for `code_edit` and `diff_preview` | `crates/agent-tools/src/code_edit.rs` | Apache-2.0 notice & license compliance |
| **Walkdir** | https://github.com/BurntSushi/walkdir | 2.5+ | Unlicense OR MIT | Fast recursive directory traversal skipping ignore directories | `crates/agent-tools/src/code_intelligence.rs` | MIT notice preserved |
| **Regex** | https://github.com/rust-lang/regex | 1.11+ | MIT OR Apache-2.0 | AST-aware symbol extraction and compiler diagnostic parsing | `crates/agent-platform/src/language.rs`, `agent-tools` | MIT notice preserved |
| **Lucide React** | https://github.com/lucide-icons/lucide | 0.453+ | ISC | UI iconography | `apps/desktop/src` | ISC notice preserved |
| **Tailwind CSS**| https://github.com/tailwindlabs/tailwindcss | 3.4+ | MIT | CSS design system | `apps/desktop` | MIT notice preserved |

---

## Evaluation & Selection Rationale

1. **Playwright MCP (Phase 3)**:
   - Evaluated custom headless Chromium binding via DevTools Protocol (CDP) in Rust vs Microsoft Playwright MCP.
   - Selected `@playwright/mcp` over stdio JSON-RPC: Microsoft-maintained cross-browser automation suite exposing 25 rich tools, accessibility trees, snapshotting, and input emulation. Running out-of-process via Node guarantees process isolation and zero Rust-level browser crash propagation.

2. **Windows-MCP (Phase 1 & 3)**:
   - Evaluated out-of-process stdio integration vs embedding Python C-API.
   - Selected out-of-process stdio MCP client (`uv run --python 3.14 windows-mcp`) to prevent GIL locks, memory leaks, or native Python runtime crashes from destabilizing the Rust host daemon.

3. **OpenCode Integration (Phase 2)**:
   - Evaluated wholesale vendoring of 50,000+ lines of TypeScript vs native Rust clean-room implementation of proven algorithms.
   - Selected native Rust implementation for `CodeEditTool` (exact match replacement, quote normalization, single-match uniqueness), `CodePatchTool` (atomic multi-file rollback), `RepoMapTool` (compact symbol index), and `CodingWorkflowEngine` (`Ask`, `Plan`, `Build` modes).
   - This delivers maximum speed, zero extra runtime bloat, and 100% memory safety.

4. **Multi-Language Compiler Adapters**:
   - Implemented native structured parsers for Rust `rustc`, TypeScript `tsc`, and Python tracebacks, paired with cascading error deduplication to minimize context token usage.
