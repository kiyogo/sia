# Shared Memory & Workflow Context Architecture

## 1. Multi-Tier Memory Hierarchy
Rather than concatenating entire conversation threads, the Multi-Agent Workspace implements a structured 4-tier memory architecture:

1. **Working Memory**:
   - Ephemeral in-memory context required for the active subtask.
   - Contains task prompt, targeted inputs, and active tool schemas.
2. **Workflow Memory**:
   - Global progress, completed task outcomes, and artifact references.
   - Stored in SQLite (`workflows`, `workflow_tasks`, `task_dependencies`).
3. **Project Memory**:
   - Repository outline, compact AST symbol maps, and build configurations.
   - Cached via `RepoMapTool` and `TokenOptimizer`.
4. **User Preferences & Security Policies**:
   - Permission modes (`FullAuto`, `WorkspaceAuto`, `Strict`), workspace directory boundaries, and external side-effect approvals.

---

## 2. Memory Provenance & Verification
Every key piece of memory stored in the shared store includes:
- **Source Agent**: Identifying who generated the finding.
- **Verification Status**: `Verified` (proven by test exit code, build, or DOM check) vs `Inferred` (LLM conjecture).
- **Integrity Hash**: Unique hash guaranteeing uncorrupted state across workers.
