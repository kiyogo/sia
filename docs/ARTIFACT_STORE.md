# Shared Artifact Store Specification

## 1. Overview
The `SharedArtifactStore` (`agent-core/src/orchestration/artifact_store.rs` and `agent-storage/src/workflow_store.rs`) provides reference-based artifact exchange across workers without prompt bloating.

---

## 2. Artifact Schema & Data Model

- **Artifact Descriptor (`ArtifactRef`)**:
  - `id`: Unique `Uuid`
  - `workflow_id`: Workflow `Uuid`
  - `task_id`: Producing Task `Uuid`
  - `artifact_type`: `CodePatch` | `UnifiedDiff` | `DomSnapshot` | `Screenshot` | `BuildLog` | `DocumentReport`
  - `name`: Human-readable identifier (e.g. `App.tsx`, `research_summary.md`)
  - `hash`: SHA-256 content hash (`hash:0123456789abcdef`)
  - `path_or_uri`: Local filesystem path or URI reference
  - `summary`: Compressed preview (max 3 lines + byte count)
  - `byte_size`: Content length in bytes
  - `created_at`: UTC Timestamp

---

## 3. Token Savings Analysis
Passing an `ArtifactRef` consumes **~45 tokens** compared to injecting a 500-line source file or multi-kilobyte DOM dump (**~2,500-4,000 tokens**). Downstream agents read full contents only when performing surgical edits on that specific artifact.
