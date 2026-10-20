# Open-Source Inventory & Audit Matrix

| Component | Upstream Repository | Version / Commit | License | Usage Mode | Integration Location | Obligations & Attribution |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Windows-MCP** | https://github.com/CursorTouch/Windows-MCP | 0.8.8 / latest release | MIT | Out-of-process subprocess via stdio MCP JSON-RPC protocol | `crates/agent-tools/src/windows_mcp.rs`, `mcp_client.rs` | Preserved MIT copyright notice; external process invocation; no modified binaries bundled directly in core |
| **OpenCode** | https://github.com/anomalyco/opencode | v2.x architecture reference | MIT | Architecture reference and `CodingEngineAdapter` trait interface | `crates/agent-platform/src/traits.rs` | Preserved MIT notice; separate adapter abstraction; no trademark infringement |
| **Tauri 2** | https://github.com/tauri-apps/tauri | 2.12.x | MIT / Apache-2.0 | Host runtime & WebView2 IPC | `apps/desktop/src-tauri` | Cargo dependency attribution |
| **Tokio** | https://github.com/tokio-rs/tokio | 1.40+ | MIT | Async executor & task supervision | All workspace crates | Cargo dependency attribution |
| **Rusqlite** | https://github.com/rusqlite/rusqlite | 0.32.x | MIT | Embedded SQLite persistent store | `crates/agent-storage` | Cargo dependency attribution |
| **Reqwest** | https://github.com/seanmonstar/reqwest | 0.12.x | MIT / Apache-2.0 | HTTP client for OpenAI-compatible LLM endpoints | `crates/agent-core` | Cargo dependency attribution |
| **Similar** | https://github.com/mitsuhiko/similar | 2.6+ | Apache-2.0 | Unified diff generator for file patch tool | `crates/agent-tools` | Apache-2.0 notice & license compliance |
| **Lucide React** | https://github.com/lucide-icons/lucide | 0.453+ | ISC | UI iconography | `apps/desktop/src` | ISC notice preserved |
| **Tailwind CSS**| https://github.com/tailwindlabs/tailwindcss | 3.4+ | MIT | CSS design system | `apps/desktop` | MIT notice preserved |

---

## Evaluation & Selection Rationale

1. **Windows-MCP**:
   - Evaluated out-of-process stdio integration vs embedding Python C-API.
   - Selected out-of-process stdio MCP client to prevent GIL issues, Python runtime crashes affecting the Rust core, and ensuring the MCP server runs cleanly in the interactive Windows user session.
   - Verified that `python >= 3.14` and `uv` are strict prerequisites; implemented automatic pre-flight diagnostic so the user is never misled with a fake connection.

2. **OpenCode**:
   - Evaluated full Rust rewrite vs decoupled adapter architecture.
   - Selected clean `CodingEngineAdapter` trait in `agent-platform` to allow interfacing with OpenCode via CLI/SDK in Phase 2 without delaying Phase 1 core functionality or duplicating 50,000 lines of unrelated TypeScript code.

3. **Database (Rusqlite with WAL mode)**:
   - High-throughput, zero-latency local embedded SQL storage with ACID durability.
   - Native migrations handle `conversations`, `messages`, `tasks`, `checkpoints`, and `tool_audit_logs`.
