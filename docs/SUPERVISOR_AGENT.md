# Supervisor Agent Specification

## 1. Role & Responsibilities
The **Supervisor Agent** operates as the cognitive conductor of the Multi-Agent Workspace. Its core responsibilities include:
1. **Goal Analysis & Scope Formulation**: Translating vague user requirements into structured technical deliverables.
2. **Task Decomposition & DAG Generation**: Breaking complex objectives into discrete, non-overlapping subtasks with explicit dependencies and acceptance criteria.
3. **Plan Validation**: Verifying graph acyclicity (Kahn's topological sort) and rejecting circular or dangling dependencies.
4. **Deterministic Outcome Review**: Inspecting worker artifacts, test runner exit codes, and DOM verification states before marking tasks complete.
5. **Adaptive Replanning**: Dynamically injecting corrective tasks when child workers encounter compiler errors or test failures.

---

## 2. Planning Protocols & Schema

The Supervisor produces a structured JSON Task DAG conforming to:
```json
{
  "tasks": [
    {
      "id": "t1",
      "title": "Scaffold Web Application",
      "description": "Initialize package.json and project files",
      "assigned_agent": "Coding",
      "priority": 10,
      "dependencies": [],
      "acceptance_criteria": "Project manifest and source entry point created on disk"
    },
    {
      "id": "t2",
      "title": "Verify UI in Headless Browser",
      "description": "Navigate to local server and verify DOM tree",
      "assigned_agent": "Browser",
      "priority": 8,
      "dependencies": ["t1"],
      "acceptance_criteria": "Root element visible with zero console errors"
    }
  ]
}
```

---

## 3. Limits & Guardrails
- **Max Graph Nodes**: Capped at 25 nodes (configurable via `WorkflowBudgetConfig`).
- **Max Delegation Depth**: Capped at 5 tiers.
- **Max Retries**: Capped at 3 retries per task.
- **No Self-Privilege Escalation**: The Supervisor cannot grant itself or child workers permissions beyond what the user authorized.
- **No Infinite Loops**: Repeated identical tool invocations or plan oscillations trigger the `LoopDetector` and pause the workflow for human input.
