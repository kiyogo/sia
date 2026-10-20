# Agent Registry Specification

## 1. Registry Architecture
The `AgentRegistry` (`agent-core/src/orchestration/agent_registry.rs`) provides centralized registration, capability tracking, concurrency bounds, and operational lifecycle controls for all specialized agents.

---

## 2. Agent Catalog & Capability Profiles

| Agent Identifier | Agent Type | Functional Capabilities | Bound Concurrency | Selective Tools |
|---|---|---|---|---|
| `agent_supervisor` | `Supervisor` | `TaskDecomposition`, `ResultVerification`, `DocumentSummarization` | 1 | `plan_decompose`, `plan_verify`, `plan_replan` |
| `agent_coding` | `Coding` | `CodeAnalysis`, `CodeEditing`, `GitOperations`, `BuildAndTest` | 2 | `code_edit`, `code_patch`, `repo_map`, `code_search`, `file_outline`, `build_runner`, `test_runner`, `git_status`, `git_diff`, `git_commit` |
| `agent_browser` | `Browser` | `BrowserNavigation`, `DomInspection`, `WebFormInteraction` | 2 | `browser_navigate`, `browser_snapshot`, `browser_click`, `browser_fill`, `browser_download`, `browser_upload`, `browser_tabs` |
| `agent_computer` | `Computer` | `DesktopWindowControl`, `DesktopInput` | 1 (Exclusive) | `desktop_list_windows`, `desktop_switch_window`, `desktop_capture`, `desktop_uia_tree`, `desktop_mouse_click`, `desktop_key_input` |
| `agent_research` | `Research` | `WebSearch`, `DocumentSummarization`, `CodeAnalysis` | 3 | `web_search`, `doc_summarize`, `code_search` |

---

## 3. Dynamic Registry Management
- **Enable / Disable Toggle**: Disabling an agent marks it unavailable; the scheduler re-routes or pauses dependent tasks.
- **Concurrency Limiting**: Each agent has an independent concurrency ceiling preventing runaway process spawning.
- **Selective Tool Scoping**: Each agent's system prompt receives *only* its whitelisted tools, reducing context tokens by 500-1,000 tokens per invocation.
