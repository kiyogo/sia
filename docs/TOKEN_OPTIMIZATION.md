# Token Optimization Engine
## Universal Autonomous AI Workspace

## 1. Overview & Objectives

In autonomous agent systems, token consumption directly drives operational cost and latency. Poor context management often leads to:
1. Exponential token inflation from dumping multi-megabyte log files into context.
2. Context window saturation, causing model truncation or forgotten instructions.
3. Slow response latencies.

The **Token Optimization Engine** in `agent-core` enforces strict budget policies to keep context compact, informative, and cost-effective.

---

## 2. Architecture & Modules

### A. Context Budget Allocation
Context is partitioned into distinct reserves to prevent any single component from starving others:

```text
Total Context Window (e.g. 32,000 tokens)
├── System Prompt Reserve:       2,500 tokens (8%)
├── Conversation History:       18,000 tokens (56%)
├── Tool Execution Results:      7,500 tokens (23%)
└── Completion Generation:       4,000 tokens (13%)
```

### B. Tool Result Truncation
- Long command stdout or file reads are capped by line count (default: 150 lines) and byte size (default: 16 KB).
- When truncation occurs:
  * Key header lines and status codes are preserved.
  * Omitted lines and bytes are calculated.
  * A structured note is appended informing the AI of how many lines were omitted and prompting it to use line offset/limit parameters (e.g. `file_read { offset: 151, limit: 100 }`) if more detail is required.
  * The full untruncated output is saved locally in the database audit log.

### C. Conversation History Compaction
- When conversational message turns exceed the history budget, older intermediate turns are compacted.
- The compactor preserves:
  * The initial system prompt and primary user instruction.
  * A structured summary of intermediate progress (files created/modified, commands run, test outcomes).
  * The most recent 4 message turns verbatim.
- This ensures the model retains full situational awareness without carrying thousands of obsolete intermediate tokens.

### D. Selective Context
- The agent loop never dumps the entire project repository or full terminal buffers into prompt context.
- Files and directory listings are inspected selectively through targeted tool calls (`file_read`, `file_list`, `file_search`).

### E. Usage Measurement & Cost Estimation
- Every model response updates cumulative usage statistics (`prompt_tokens`, `completion_tokens`, `total_tokens`, `cached_tokens`).
- Real-time cost estimation is computed using standard pricing tables (e.g., $0.15/1M input tokens, $0.60/1M output tokens).

---

## 3. Phase 3 Specialized Computer & Browser Token Optimizations

### A. Accessibility-First & DOM-First Representation
Rather than passing multi-megapixel screenshots (~1,800–2,500 tokens per step) into multimodal vision models:
- Desktop: Uses Windows UI Automation (UIA) to query element names, control types, and bounds (~180 tokens).
- Browser: Uses Playwright MCP `browser_snapshot` to extract an accessibility tree with element ref IDs (`ref_1`, `ref_2`) (~220 tokens).
- **Result:** Over 90% reduction in tokens per observation cycle.

### B. DOM Tree Pruning & Depth Limiting
- Non-interactive container nodes (`<div>`, `<span>` without semantics, scripts, stylesheets, svgs) are recursively stripped.
- The snapshot engine caps traversal depth (default: 5 levels) to prevent runaway DOM trees on complex web portals.
- Long text nodes exceeding 60 characters are truncated into snippets with trailing ellipsis.

### C. Observation Delta Engine
On consecutive interactions with the same page or application:
- Consecutive snapshots are compared line-by-line.
- Identical subtrees are suppressed; only modified, added, or removed nodes are sent to the LLM context.
- Reduces token footprint on iterative form filling by up to 95%.

### D. Observation State Hashing & Cache
- Fast 64-bit state hashing (`TokenOptimizer::hash_state`).
- If an action resulted in an invariant UI state, the agent detects the unchanged hash and avoids redundant re-interpretation by the LLM.

---

## 4. Phase 5 Business & Research Token Optimization Strategies

### A. Code-First Mathematical & Statistical Analysis
- Instead of dumping raw CSV/XLSX spreadsheets (which can consume 50,000 to 200,000+ tokens) into the LLM context to ask for averages or trends, **100% of data calculations run natively in Rust**.
- The LLM only receives compact, pre-computed `ColumnSummary` structures (Mean, Median, StdDev, Min, Max, Outliers) (~120 tokens total).
- **Token Savings:** >98% reduction on analytical tasks, with 100% arithmetic precision.

### B. Structure-Aware Document Chunking & Selective Retrieval
- Multi-hundred page documents (PDFs, Word documents, research papers) are parsed, chunked, and indexed locally in SQLite.
- Only the top $K$ relevant chunks matching the user's specific query are loaded into prompt context.
- Chunks retain section breadcrumbs, eliminating redundant contextual preamble tokens.

### C. Bracketed Citation Fingerprints
- Sources are identified by short integer bracketed keys (`[1]`, `[2]`).
- Source metadata (long URLs, SHA-256 hashes, timestamps) is held outside the LLM context in the `business_store` database and appended deterministically during final markdown/DOCX/PDF rendering.
- Prevents repetitive URL regurgitation in intermediate reasoning steps.

### D. Email Thread Normalization & Signature Stripping
- Redundant quote pyramids (`> On Monday, John wrote:...`), standard corporate disclaimers, and tracking pixel URLs are stripped before generating email summaries.
- Reduces inbound email thread context by 60–80%.
