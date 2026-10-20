# OpenCode Architecture & Integration Analysis

## 1. Executive Overview
- **Project**: OpenCode (`github.com/opencode/opencode` or equivalent open-source repository).
- **Inspection Path**: `C:\Users\sonng\AppData\Local\Temp\opencode-inspect`.
- **License**: MIT License (Permissive, commercial and private use permitted with attribution).
- **Evaluation Purpose**: Analyze OpenCode's architectural patterns, tool definitions, agent loops, context compaction, and file-editing mechanics to integrate proven coding agent practices into the Universal Autonomous AI Workspace.

---

## 2. OpenCode Architectural Anatomy

OpenCode is structured as a TypeScript/Effect-TS monorepo under `packages/`:

```text
packages/
├── ai/                 # LLM provider routing, token streaming, schema codecs
├── core/               # Core runtime services:
│   ├── agent.ts        # Agent profiles (build, plan, ask)
│   ├── tool.ts         # Tool registration, execution snapshot, permission gates
│   ├── tool/plugin/    # Built-in tools: edit, patch, grep, glob, read, write, shell, todo
│   ├── session/        # Session runner, compaction, model requests, history
│   ├── vcs/            # Git integration, diff calculation, patch management
│   ├── ripgrep.ts      # Fast filesystem grep adapter
│   └── instructions/   # Dynamic prompt injections (<env>, repo context)
├── cli/                # Terminal interface and interactive runner
└── desktop/            # Desktop client integration
```

### 2.1 Core Tooling Mechanics

#### A. Exact Match File Editing (`tool/plugin/edit.ts`)
- **Philosophy**: Replaces exact text occurrences rather than hallucinating entire file rewrites.
- **Key Parameters**:
  - `path`: Target file path.
  - `oldString`: Exact substring to locate.
  - `newString`: Exact replacement string.
  - `replaceAll`: Boolean flag. If `false`, `oldString` must match exactly once; otherwise, it rejects with an ambiguity error.
- **Normalization**: Handles quote variations (curly vs straight quotes), hyphens, and whitespace discrepancies to prevent brittle model edits.

#### B. Unified Patch Tool (`tool/plugin/patch.ts`)
- **Philosophy**: Handles multi-file atomic edits with hunk application (`add`, `update`, `delete`).
- **Safety**: Validates that all hunks match clean targets before applying. If any hunk fails, the batch aborts to avoid leaving the working directory in a corrupted state.

#### C. Ripgrep & Fast Search (`ripgrep.ts`, `tool/plugin/grep.ts`)
- Uses `ripgrep` for line-by-line pattern matching with configurable limits (e.g. 100 matches) and automatic exclusion of binary files, `.git`, and build caches.

#### D. Git & Worktree Protection (`git.ts`, `vcs/patch.ts`)
- Maintains explicit knowledge of repository state (`Repository`, `TreeID`).
- Rejects destructive operations like `git reset --hard` or forced deletions without human authorization.

#### E. Context Compaction (`session/compaction.ts`)
- Evaluates token consumption against model context limits.
- When approaching budget (or on overflow), triggers compaction:
  - Preserves a `keep` window of recent messages verbatim.
  - Summarizes preceding turns, tool call arguments, and tool outputs.

#### F. Agent Modes (`agent.ts`)
- Pre-configured agent personas:
  - `build` (Default): Full agent with file write, terminal execution, and testing capabilities.
  - `plan`: Read-only agent focused on researching and formulating implementation plans.
  - `ask`: Conversational agent answering architecture and codebase questions without side-effects.

---

## 3. Comparative Integration Strategies

| Strategy | Feasibility | Maintenance | Performance | Recommendation |
|---|---|---|---|---|
| **A. Monolithic JS Ingestion**<br/>(Copying full TS repo into workspace) | Low | High (50,000+ lines of TS, Bun/Node runtime required) | Medium | **Rejected** (Unnecessary weight, bloat, and runtime fragmentation) |
| **B. Native Rust Implementation**<br/>(Reimplementing OpenCode patterns natively in Rust) | Very High | Low (Single unified Rust workspace, zero extra runtimes) | Extreme (Native C-speed regex, ripgrep walk, memory safe) | **Primary Architecture** |
| **C. Out-of-Process Sidecar Adapter**<br/>(CLI/RPC bridge to OpenCode daemon if needed) | High | Medium | High | **Optional Extension Hook** |

---

## 4. Implementation Blueprint for Universal AI Workspace

1. **Native Rust Tools (`crates/agent-tools`)**:
   - `CodeEditTool`: Implementing OpenCode's exact-match edit with `oldString`/`newString`, normalization, and single-match enforcement.
   - `CodePatchTool`: Implementing atomic multi-file patch with rollback on error.
   - `CodeSearchTool`: Ripgrep-like fast pattern search with ignore rules.
   - `RepoMapTool`: Compact symbol-aware codebase map.
   - `GitTools`: `git_status`, `git_diff`, `git_commit`, `git_branch`.
   - `BuildTestTool`: Multi-language build and test runner (`cargo`, `npm`, `pytest`).

2. **Diagnostics & Self-Correction Pipeline (`crates/agent-core`)**:
   - Multi-language compiler diagnostic parser (`rustc`, `tsc`, `python`).
   - Cascading error deduplicator to isolate root causes and conserve context tokens.

3. **Autonomous Coding Loop Modes (`crates/agent-core`)**:
   - `CodingMode::Ask`: Read-only analysis and symbol navigation.
   - `CodingMode::Plan`: Plan generation with task breakdown.
   - `CodingMode::Build`: End-to-end plan execution with automated build, test, and self-correction.
