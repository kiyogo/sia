# Phase 1 Scope & Boundary Specification

## 1. Phase 1 Deliverables (Completed & Verified)

1. **Desktop Chat UI**:
   - Modern, professional dark/light UI built with Tauri 2, React 18, Tailwind CSS, and Lucide Icons.
   - Real-time streaming response and visual tool execution cards.
   - Message cancellation and multi-session switching.

2. **Rust Agent Core Runtime**:
   - Multi-step agent loop handling sequential tool calls.
   - Loop bounds protection (`max_loops`).
   - Clean event broadcasting (`StepStarted`, `ToolCallStarted`, `ToolCallFinished`, `TokenUsageUpdated`, `FinalAnswer`).

3. **AI Provider Connectivity**:
   - Standard OpenAI-compatible HTTP client supporting OpenAI (`gpt-4o`), Ollama (`llama3.2`), LM Studio, Groq, DeepSeek, vLLM.
   - Connection validation test and token usage tracking.

4. **Terminal Executor**:
   - Native CMD and PowerShell execution using `tokio::process::Command`.
   - Streaming stdout and stderr buffer capture.
   - Exit code tracking, execution duration measurement, and process timeout kill.

5. **Filesystem Tools**:
   - `file_read` (line offset and limit).
   - `file_write` (create, overwrite, append, directory auto-creation).
   - `file_list` (recursive directory traversal with size metadata).
   - `file_search` (file name pattern search and full-text content search).
   - `file_patch` (targeted block search-and-replace with unified diff generation).

6. **Windows-MCP Desktop Control**:
   - Standard MCP Client supporting JSON-RPC 2.0 over stdio.
   - Dynamic tool discovery (`tools/list`) and invocation (`tools/call`).
   - Pre-flight diagnostic checking Python 3.14+ and uv prerequisites.

7. **Task Management & Execution History**:
   - SQLite-backed task store with status lifecycle: `Pending`, `Running`, `WaitingApproval`, `Paused`, `Completed`, `Failed`, `Cancelled`.
   - Goal and current step tracking.

8. **Crash Resilience & Checkpoint Recovery**:
   - Checkpoint creation after intermediate agent steps.
   - Safe startup scan: transitions incomplete tasks to `Paused` for human inspection instead of executing dangerous blind side-effects.

9. **Security & Permission Engine**:
   - Multi-tier permission model (`ReadOnly`, `WorkspaceWrite`, `ShellExecution`, `DesktopControl`, `DestructiveAction`).
   - Strict workspace boundary enforcement.
   - Human-in-the-loop approval dialog for dangerous actions.
   - Instant Emergency Stop mechanism.

10. **Token Optimization Engine**:
    - Context budget manager.
    - Tool result line and byte truncation with pagination guidance.
    - Conversation compaction for long-running histories.
    - Accurate token estimation and usage tracking.

---

## 2. Explicitly Deferred to Phases 2–8

- Complex Multi-Agent Swarms with autonomous sub-agent spawning.
- Deep Browser Agent with DOM-tree mutation and stealth anti-bot automation.
- Enterprise ERP / CRM automated connectors.
- Local quantized on-device LLM inference via embedded candle/llama.cpp (Phase 2).
