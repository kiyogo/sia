# Task DAG & Dependency Engine Specification

## 1. Directed Acyclic Graph (DAG) Model
The Task DAG (`agent-core/src/orchestration/task_dag.rs`) coordinates multi-step workflows with strict validation, dependency resolution, and dynamic replanning.

---

## 2. Structural Schema

- **Node (`DagNode`)**:
  - `task_id`: `Uuid`
  - `title`: String
  - `description`: String
  - `assigned_agent`: `AgentType`
  - `priority`: `i32` (Higher values dispatched first when multiple tasks are ready)
  - `inputs`: `serde_json::Value`
  - `acceptance_criteria`: String
- **Edge (`DagEdge`)**:
  - `from`: `Uuid` (Prerequisite task)
  - `to`: `Uuid` (Dependent task)
  - `condition`: String ("Success", "Completed")

---

## 3. Algorithmic Validation

### 3.1 Cycle Detection via Kahn's Algorithm
The DAG validates acyclicity using Kahn's topological sort algorithm:
1. Compute the in-degree of every node in the graph.
2. Initialize a queue with all nodes having `in_degree == 0`.
3. Dequeue nodes, append to topological order list, and decrement in-degrees of neighboring nodes.
4. If the count of visited nodes is strictly less than total graph nodes, an illegal cycle exists and `DagError::CycleDetected` is raised.

### 3.2 Integrity Assertions
- No self-dependencies (`edge.from != edge.to`).
- No dangling edges (both endpoints must exist in `nodes`).
- Duplicate node IDs rejected immediately.

---

## 4. Ready Task Resolution
`TaskDag::get_ready_tasks(&completed_tasks, &in_progress_tasks)` evaluates all graph nodes and returns only those whose incoming prerequisites have all succeeded, sorted by priority descending.
