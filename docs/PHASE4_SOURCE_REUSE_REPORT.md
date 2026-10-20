# Phase 4 Open-Source Reuse & Architectural Research Report

## 1. Executive Summary
In accordance with the Open-Source-First principle, this report audits and compares industry-standard multi-agent frameworks, workflow orchestration systems, and our previously integrated components to guide the architectural design of Phase 4.

---

## 2. Framework Comparison & Evaluation Matrix

| Framework | Core Language | Primary Focus | License | Integration Feasibility in Rust Core | Decision & Rationale |
|---|---|---|---|---|---|
| **Rig** (`0xPlaygrounds/rig`) | Rust | Ergonomic modular LLM & agent abstraction for Rust | MIT | **HIGH**: Native Rust crate, Tokio async, clean agent/tool separation. | **Architectural Reference**: Adopt Rig's agent trait abstractions, tool calling pipelines, and typed completion models natively within `agent-core`. |
| **Microsoft Agent Framework** | Python / C# | Multi-agent orchestration, sequential/parallel flows, agent handoff | MIT | **LOW**: Heavily tied to Python/C# runtimes; embedding Python or .NET host inside desktop daemon introduces huge binary bloat and GIL issues. | **Pattern Reference**: Adopt architectural concepts: Agent Handoff, Structured Messages, and Human-in-the-loop approvals without adding external runtime dependencies. |
| **LangGraph** (`langchain-ai/langgraph`) | Python / JS | Graph-based state machine, cyclical workflows, durable checkpoints | MIT | **MEDIUM**: Excellent DAG/graph state machine semantics, but running Node/Python daemon for core orchestration adds unnecessary latency. | **Design Pattern**: Implement native Rust DAG state machine with Kahn's topological sort, conditional edge transitions, and checkpointing. |
| **Temporal** (`temporalio/temporal`) | Go (Server) / Multi-SDK | Fault-tolerant durable execution, activity retries, replay-based recovery | MIT | **LOW for Phase 4 MVP**: Requiring users to run a full Temporal Server (with Cassandra/PostgreSQL) contradicts our single-binary desktop distribution requirement. | **Future Target**: Model our workflow, task attempt, and lease state machine after Temporal semantics (activities, heartbeats, deterministic replays) backed by embedded SQLite WAL, leaving an interface for future Temporal integration in Phase 6/7. |
| **OpenCode** (`anomalyco/opencode`) | TypeScript / Rust pattern | High-precision code mutation, atomic patch rollback, repo mapping | MIT | **HIGH (Already Integrated)** | **Reused**: Direct reuse of Phase 2 native Rust `CodeEditTool`, `CodePatchTool`, `RepoMapTool`, and compiler diagnostic parsers as the engine of the `CodingAgent`. |
| **Windows-MCP** (`CursorTouch/Windows-MCP`) | Python / Stdio MCP | Windows desktop control, window management, accessibility tree | MIT | **HIGH (Already Integrated)** | **Reused**: Direct reuse of Phase 1/3 `WindowsMcpManager` via Python 3.14 stdio subprocess as the engine of the `ComputerAgent`. |
| **Playwright MCP** (`@playwright/mcp`) | TypeScript / Node Stdio MCP | Headless/headed browser automation, DOM accessibility trees | Apache-2.0 | **HIGH (Already Integrated)** | **Reused**: Direct reuse of Phase 3 `BrowserMcpManager` via Node stdio subprocess as the engine of the `BrowserAgent`. |

---

## 3. Detailed Component Analysis

### 3.1 Rig (0xPlaygrounds/rig)
- **Strengths**: Clean, idiomatic Rust with Tokio support. Strong typing around prompts, tools, and completions. Zero runtime overhead.
- **Adoption**: The `agent-core` crate utilizes Rig-style `AgentDefinition`, `ToolDefinition`, and completion request abstractions to ensure maximum memory safety and zero cost abstractions.

### 3.2 Microsoft Agent Framework
- **Strengths**: Elegant separation of duties between Supervisor and Workers, message routing protocols, and multi-agent handoff contracts.
- **Adoption**: We implement the structured message contracts (`TaskAssigned`, `ArtifactProduced`, `TaskCompleted`, `ApprovalRequired`) inspired by the Microsoft Agent Framework specifications.

### 3.3 LangGraph & State Machine Orchestration
- **Strengths**: Directed Acyclic Graph (DAG) task representation with explicit dependencies and conditional branches.
- **Adoption**: Native Rust `TaskDag` utilizing topological sorting for cycle detection and deterministic dependency resolution.

### 3.4 Temporal Durable Execution Patterns
- **Strengths**: Execution state persistence, non-blocking activity execution, heartbeat tracking, and idempotent retries.
- **Adoption**: We map Temporal's workflow/activity/attempt model directly onto our SQLite storage schema with WAL mode, transactions, and lease heartbeats.

---

## 4. Architectural Selection Summary

The Phase 4 architecture achieves the best of both worlds:
1. **Zero External Server Overhead**: Runs 100% self-contained in a single, lightweight Windows desktop binary (`desktop.exe`).
2. **Native Rust Performance**: Orchestration, DAG validation, scheduling, and state persistence execute in nanoseconds/microseconds without Python or Node intermediary hops.
3. **Established Worker Integrations**: Out-of-process subprocess isolation for Windows-MCP (Python 3.14) and Playwright MCP (Node) guarantees that worker crashes cannot crash the Rust orchestrator.
