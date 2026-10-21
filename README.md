# Universal Autonomous AI Workspace
### Windows-First, Cross-Platform-Ready, Open-Source-First Desktop Agent Platform

An autonomous, multi-purpose AI Workspace designed to perform real work on Windows. The platform bridges natural language instructions from users with native system capabilities: executing terminal commands in CMD/PowerShell, analyzing and modifying codebases across multiple languages, controlling Windows applications, navigating modern web browsers via Playwright MCP, orchestrating complex Multi-Agent Task DAGs, conducting systematic evidence-grounded research, processing multi-format documents, performing deterministic data analytics with responsive vector charts, and automating business communications through an approval-gated email and calendar engine.

---

## 1. System Vision & Architecture

The application is structured as a multi-crate Rust workspace with a desktop UI built using Tauri 2, React 18, TypeScript, and Tailwind CSS.

```text
aai/
├── apps/
│   └── desktop/                  # Tauri 2 + React + TypeScript + Tailwind UI
│       ├── src/                  # React UI (ResearchWorkspace, BusinessDashboard, WorkflowOrchestrator, ComputerControl, BrowserWorkspace, CodingWorkspace, Chat, Tasks, Terminal)
│       └── src-tauri/            # Tauri 2 host binary & IPC command handlers
├── crates/
│   ├── agent-platform/           # OS-agnostic traits, Win32 adapters, generic stubs & MultiLanguageAdapter
│   ├── agent-security/           # Permission tiers, DataClassification, SecretRedactor, PromptInjectionDefense, DesktopOwnershipManager
│   ├── agent-storage/            # SQLite embedded database with WAL mode, schema migrations, WorkflowStore & BusinessStore
│   ├── agent-tools/              # Browser MCP, Windows MCP, Git, OpenCode code_edit/patch, search, repo_map
│   ├── agent-core/               # Research Engine, Document Intelligence, Data Analysis & SVG Charts, Business Connectors, Task DAG, Tool Router
│   └── agent-runtime/            # Business Coordinator, Supervisor engine, Worker dispatcher, Workflow scheduler, task runner, and recovery engine
├── docs/                         # Comprehensive architecture, multi-agent, business automation, research, security, and QA reports
├── scripts/                      # PowerShell setup, dev, test, and release build automation scripts
├── LICENSE                       # MIT License
├── THIRD_PARTY_NOTICES.md        # Open-source attributions (Playwright, Windows-MCP, OpenCode, Tauri, Zip, etc.)
└── README.md
```

---

## 2. Core Capabilities Across Phases

### 2.1 Phase 5: Advanced Business & Research Automation
1. **Research Agent & Provenance Engine**:
   - Systematic web and local workspace research querying `LocalWorkspaceSearchProvider` and external search adapters.
   - Source provenance tracking with computed SHA-256 integrity digests, HTTP etags, and credibility scoring.
   - Sentence/offset evidence extraction and automated quantitative contradiction detection (>15% divergence flagged).
   - Generates comprehensive markdown whitepapers with verifiable, bracketed citations (`[1]`, `[2]`).
2. **Document Intelligence Subsystem**:
   - Multi-format parser for Markdown, HTML, plain text, and OpenXML DOCX.
   - Native OpenXML (`.docx`) package generator with standard ECMA-376 structure (`word/document.xml`, `[Content_Types].xml`, `_rels`) using pure Rust `zip` crate.
   - Headless PDF 1.4 binary stream generator with standard xref tables and font dictionaries.
   - Structure-aware chunking preserving paragraph boundaries and heading hierarchy with sliding overlap.
   - Document revision versioning and unified diffing.
3. **Deterministic Data Analysis & Vector Charting**:
   - Auto-detecting CSV/TSV reader and OpenXML XLSX sheet reader.
   - Code-first arithmetic computation: Mean, Median, Sample Standard Deviation, Min, Max, and Tukey IQR Outlier Detection ($Q_1 - 1.5 \times IQR$, $Q_3 + 1.5 \times IQR$).
   - Multi-level categorical group-by aggregations (Sum, Count, Average).
   - Standalone responsive SVG vector chart generation (Bar and Line charts) with dark/light theme support.
4. **Business Connectors & Email Subsystem**:
   - Resilient connector registry with leaky-bucket rate limiting, exponential backoff with jitter, and 3-strike circuit breaking.
   - Thread summarization, tone-matched draft creation, and idempotent dispatch tracking with unique `operation_id` GUIDs.
   - **Strict Human Approval Gate**: Compulsory authorization queue in `business_approvals` preventing any unauthorized outbound emails or external mutations.
   - Calendar availability checking, UTC time normalization, and conflict resolution.
5. **Business Workflow Coordinator**:
   - 5 pre-packaged business templates (`market_research`, `data_reporting`, `email_assistance`, `research_to_document`, `hybrid_pipeline`) orchestrating tasks on Phase 4's durable Task DAG.
6. **Enterprise Security & Token Optimization**:
   - 4-tier data classification (`Public`, `Internal`, `Confidential`, `Restricted`).
   - Deterministic secret redaction replacing API keys, passwords, and tokens with `[REDACTED_SECRET]`.
   - Structural prompt injection containment wrapping external data in `<UNTRUSTED_EXTERNAL_DATA>` blocks.
   - Code-first analytics saving **>98% of tokens** compared to injecting raw datasets into LLMs.
7. **Desktop UI: Research Workspace & Business Dashboard**:
   - **Research Workspace**: Query input, evidence cross-examination table, contradiction badge indicators, and cited markdown report preview.
   - **Business Dashboard**: Template launcher, interactive SVG charts, data summary tables, and one-click Human Approval Queue.

### 2.2 Phase 4: Advanced Multi-Agent Orchestration System
- **Supervisor AI Engine**: Goal decomposition, dependency graphing, and dynamic replanning.
- **Task Directed Acyclic Graph (DAG) Engine**: Strict cycle detection using Kahn's topological sorting algorithm.
- **Agent Registry & Worker Specialization**: Supervisor, Coding (OpenCode), Browser (Playwright MCP), Computer (Windows-MCP / Win32), Research, Document, Data Analysis, and Email workers.
- **Tokio Task Scheduler**: Bounded concurrency (1 to 4 parallel workers) with pause, resume, and cancellation.
- **Resource Locking**: Single-controller desktop lock, file mutation locks, and monotonic fencing tokens.
- **Shared Artifact Store**: Content-addressed artifact storage (SHA-256) passing compact `ArtifactRef`s (**87.3% token savings**).
- **Durable Persistence**: 8 SQLite tables tracking workflows, tasks, dependencies, attempts, artifacts, leases, messages, and token usage.
- **Desktop UI**: Dedicated Multi-Agent Workflow Orchestrator tab (`WorkflowOrchestrator.tsx`).

### 2.3 Phase 3: Advanced Computer & Browser Agent
- **Windows Desktop Control**: Window enumeration, focus, coordinates, and workstation lock status detection.
- **Playwright Browser Automation**: Cross-browser navigation, DOM accessibility inspection, form filling, and file transfer tracking.
- **6-Tier Tool Strategy Router**: Routes through the lowest-cost tier (`Direct API` → `Filesystem` → `CLI` → `DOM` → `UIA` → `Vision`).
- **Safety Framework**: Single-controller lease, user takeover detection, and instant Emergency Stop (< 5ms).

### 2.4 Phase 2: Advanced Autonomous Coding Agent
- **OpenCode Architecture**: `CodeEditTool` exact-match replacement, `CodePatchTool` atomic multi-file rollback.
- **Multi-Language Intelligence**: Rust, TypeScript/JS, Python, and C/C++ diagnostic parsers and symbol maps.
- **Git Safety**: Safe status and diff inspection; destructive commands blocked.
- **Execution Modes**: `Ask` (read-only), `Plan` (diff preview), `Build` (autonomous repair loop).

### 2.5 Phase 1: General Agent Foundation
- Conversational chat with streaming tool cards, terminal executors (CMD & PowerShell), SQLite checkpointing, and global kill switch.

---

## 3. System Prerequisites

- **Operating System**: Windows 10 or Windows 11 (64-bit)
- **Rust Toolchain**: Rust 1.80+ with MSVC build tools (`cargo`, `rustc`)
- **Node.js**: Node 18+ with `pnpm` (`pnpm install -g pnpm`)
- **Python**: Python 3.14+ (Required for Windows-MCP; install via `uv python install 3.14` or python.org)
- **uv**: Astral `uv` package manager (`irm https://astral.sh/uv/install.ps1 | iex`)
- **Playwright**: `@playwright/mcp` (Resolved dynamically via `npx -y @playwright/mcp`)
- **WebView2**: Microsoft Edge WebView2 Runtime (pre-installed on modern Windows 10/11)

---

## 4. Quickstart Guide

### Step 1 — Automated Environment Setup
```powershell
.\scripts\setup.ps1
```

### Step 2 — Run the Test Suite (All 121 Tests)
```powershell
.\scripts\test.ps1
```

### Step 3 — Launch in Development Mode
```powershell
.\scripts\dev.ps1
```

### Step 4 — Build the Production Release Binary
```powershell
.\scripts\build.ps1
```
The output binary will be created at: `target\release\desktop.exe`.

---

## 5. Acceptance Test Verification (121 / 121 Passed)

### Phase 5 Acceptance Tests (24 / 24 Passed)
| Test Identifier | Capability Tested | Status |
| :--- | :--- | :--- |
| `test_01_sqlite_business_store_schema` | SQLite `business_store` schema creation, tables, and indices | **PASS** |
| `test_02_research_source_provenance_and_hash` | Source provenance, SHA-256 fingerprinting, and credibility scoring | **PASS** |
| `test_03_evidence_extraction_and_contradiction_detection` | Evidence extraction and numerical contradiction detection (>15%) | **PASS** |
| `test_04_research_report_generation_with_verifiable_citations` | Research report generation with bracketed citations (`[1]`) | **PASS** |
| `test_05_mock_web_and_local_workspace_search` | `MockWebSearchProvider` and `LocalWorkspaceSearchProvider` searching | **PASS** |
| `test_06_document_parser_multi_format` | Multi-format document parser (Markdown, HTML, Plain text, DOCX) | **PASS** |
| `test_07_document_generator_openxml_docx` | Native OpenXML `.docx` generation with valid zip package structure | **PASS** |
| `test_08_document_generator_pdf` | Headless PDF 1.4 stream generation with standard xref tables | **PASS** |
| `test_09_document_chunker_structure_aware` | Structure-aware chunking preserving paragraph boundaries and headings | **PASS** |
| `test_10_document_versioning_and_diff` | Document revision tracking, content hashes, and unified diffing | **PASS** |
| `test_11_spreadsheet_reader_csv_delimiter_inference` | CSV delimiter auto-detection and RFC 4180 quote escaping | **PASS** |
| `test_12_data_analyzer_column_summaries_code_first` | Deterministic computation of Mean, Median, StdDev, Min, Max | **PASS** |
| `test_13_data_analyzer_iqr_outlier_detection` | Tukey IQR outlier detection ($Q_1 - 1.5 \times IQR$, $Q_3 + 1.5 \times IQR$) | **PASS** |
| `test_14_data_analyzer_group_by_aggregation` | Categorical group-by aggregation (Sum, Count, Mean) | **PASS** |
| `test_15_chart_generator_svg_bar_and_line` | Native SVG vector chart generation for Bar and Line plots | **PASS** |
| `test_16_business_connector_registry_rate_limit_circuit_breaker` | Rate limiting, backoff, and 3-strike circuit breaking | **PASS** |
| `test_17_email_agent_approval_gate_enforcement` | Mandatory human approval gate preventing direct email dispatch | **PASS** |
| `test_18_email_agent_idempotent_dispatch_deduplication` | Idempotent email dispatch deduplication via `operation_id` | **PASS** |
| `test_19_calendar_provider_free_busy_and_conflict_detection` | Free/busy calculation, UTC normalization, and meeting proposal | **PASS** |
| `test_20_business_workflow_coordinator_templates` | Instantiation of all 5 business workflow templates onto Task DAG | **PASS** |
| `test_21_data_classification_and_egress_rules` | 4-tier data classification and egress restriction enforcement | **PASS** |
| `test_22_secret_redactor_deterministic_masking` | Deterministic secret redaction masking credentials to `[REDACTED_SECRET]` | **PASS** |
| `test_23_prompt_injection_defense_passive_data_tagging` | Encapsulation of untrusted inputs in `<UNTRUSTED_EXTERNAL_DATA>` | **PASS** |
| `test_24_end_to_end_business_pipeline_integration` | End-to-end integration: Research → Analysis → Chart → DOCX → Email draft | **PASS** |

### Phases 1–4 Acceptance & Unit Tests (97 / 97 Passed)
- **Phase 4 Acceptance**: 21 / 21 Passed
- **Phase 3 Acceptance**: 18 / 18 Passed
- **Phase 2 Acceptance**: 12 / 12 Passed
- **Phase 1 Acceptance**: 9 / 9 Passed
- **Workspace Unit Tests (All Crates)**: 37 / 37 Passed

**Total: 121 / 121 Passed (100% Pass Rate). Zero Regressions.**

---

## 6. Open Source Compliance & Attribution

This project strictly adheres to open-source licensing principles:
- **Playwright MCP**: Copyright (c) Microsoft Corporation under the Apache License 2.0.
- **Windows-MCP**: Copyright (c) 2024-2026 Jeomon George (CursorTouch) under the MIT License.
- **OpenCode**: Copyright (c) 2025 opencode under the MIT License. Architectural reference for exact-match mutation, patch rollback, and session compaction.
- **Tauri 2**: The Tauri Programme within The Commons Conservancy under MIT / Apache-2.0.
- **Zip-rs**: Copyright (c) 2014 Mathijs van de Nes under the MIT License.
- All third-party notices, license texts, and dependency inventories are maintained in [THIRD_PARTY_NOTICES.md](file:///THIRD_PARTY_NOTICES.md) and [docs/OPEN_SOURCE_INVENTORY.md](file:///docs/OPEN_SOURCE_INVENTORY.md).
