# Agent Communication Protocol & Message Contracts

## 1. Overview
The Agent Communication subsystem (`agent-core/src/orchestration/messages.rs`) defines the structured, type-safe messaging format used across workers and the Supervisor.

---

## 2. Message Contract

Each message contains:
- `id`: Unique `Uuid`
- `workflow_id`: Parent workflow `Uuid`
- `task_id`: Optional associated task `Uuid`
- `source_agent`: `AgentType` (Supervisor, Coding, Browser, Computer, Research)
- `target_agent`: `AgentType`
- `message_type`: `MessageType`
- `payload`: Structured JSON object
- `artifact_refs`: Vector of lightweight `ArtifactRef`s
- `timestamp`: UTC Timestamp

---

## 3. Message Taxonomy

| Message Type | Purpose | Typical Payload |
|---|---|---|
| `TaskAssigned` | Dispatch task to worker | Task inputs, priority, deadlines |
| `TaskStarted` | Worker informs scheduler | Started timestamp, worker process ID |
| `ProgressUpdated` | Heartbeat / incremental step | Percent complete, step description |
| `ArtifactProduced` | Worker registers an output | `ArtifactRef` (hash, name, size, type) |
| `TaskCompleted` | Worker announces completion | Exit codes, verified output references |
| `TaskFailed` | Worker announces error | Error message, diagnostic traceback |
| `ApprovalRequired` | Request human authorization | Required permission tier, command preview |
| `ResourceBlocked` | Waiting for lock release | Target resource ID ("desktop:primary") |
| `TaskCancelled` | Worker abort acknowledgement | Cleanup confirmation |
