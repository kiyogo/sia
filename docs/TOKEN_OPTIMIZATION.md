# Token Optimization Engine

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
