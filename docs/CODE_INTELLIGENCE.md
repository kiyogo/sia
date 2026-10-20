# Code Intelligence & Multi-Language Tooling

## 1. Overview
The Code Intelligence engine provides automated codebase comprehension, symbol navigation, compact architectural mapping, and compiler diagnostic parsing across multiple programming languages.

---

## 2. Multi-Language Detection Matrix

The `MultiLanguageAdapter` automatically scans target directories for manifest signatures:

| Language | Primary Manifests | Secondary Manifests | Default Build Command | Default Test Command | Entry Point Candidates |
|---|---|---|---|---|---|
| **Rust** | `Cargo.toml` | `Cargo.lock` | `cargo check` | `cargo test` | `src/main.rs`, `src/lib.rs`, `main.rs` |
| **TypeScript** | `tsconfig.json`, `package.json` | `pnpm-lock.yaml`, `yarn.lock`, `bun.lockb` | `pnpm build` / `npm run build` | `pnpm test` / `npm test` | `src/index.ts`, `src/main.tsx`, `index.ts` |
| **JavaScript** | `package.json` | `package-lock.json` | `npm run build` | `npm test` | `src/index.js`, `index.js`, `src/App.jsx` |
| **Python** | `pyproject.toml`, `requirements.txt` | `setup.py`, `Pipfile`, `uv.lock`, `poetry.lock` | `python -m py_compile` | `pytest` / `uv run pytest` | `main.py`, `app.py`, `src/main.py` |
| **C / C++** | `CMakeLists.txt` | `Makefile`, `meson.build` | `cmake --build .` | `ctest` | `main.cpp`, `src/main.cpp`, `main.c` |

---

## 3. Compact Repo Mapping Algorithm

To avoid dumping tens of thousands of tokens of raw code into LLM context windows, the `RepoMapTool` implements an AST-aware compact outlining algorithm:

1. **Exclusion Filtering**: Automatically skips non-essential directories:
   - Git: `.git`
   - Build caches: `target`, `dist`, `build`, `out`
   - Dependencies: `node_modules`, `vendor`, `.venv`, `venv`
   - IDE/editor configs: `.idea`, `.vscode`, `.zed`
   - Python caches: `__pycache__`, `.pytest_cache`
2. **Symbol Extraction (Regex-driven, Zero AST bloat)**:
   - Rust: `pub fn`, `struct`, `enum`, `trait`, `type`, `impl`
   - TypeScript/JS: `export function`, `class`, `interface`, `type`, `const`
   - Python: `def`, `class`, `async def`
   - C/C++: `class`, `struct`, function signatures
3. **Budget Enforcement**:
   - Accepts a `max_chars` budget (e.g. 2000-4000 characters, ~500-1000 tokens).
   - Generates compact hierarchical bullet points:
     ```text
     - crates/agent-tools/src/git.rs (GitStatusTool, GitDiffTool, GitLogTool, GitCommitTool)
     - crates/agent-tools/src/code_intelligence.rs (ProjectDetectTool, RepoMapTool, CodeSearchTool)
     ```
   - Automatically truncates with a warning note if the budget is reached.

---

## 4. Multi-Language Diagnostic Parsers

The `MultiLanguageAdapter` ingests raw compiler stderr/stdout and extracts structured `DiagnosticMessage` instances:

### 4.1 Rust (`rustc` / `cargo check`)
- Parses both `--message-format=json` JSON records and human-readable error banners:
  ```text
  error[E0425]: cannot find value `my_var` in this scope
    --> src/main.rs:14:9
  ```
- Extracts: File (`src/main.rs`), Line (14), Column (9), Code (`E0425`), Severity (`Error`).

### 4.2 TypeScript (`tsc` / ESLint)
- Parses standard TypeScript error strings:
  ```text
  src/components/App.tsx:42:10 - error TS2304: Cannot find name 'UnknownWidget'.
  ```
- Extracts: File (`src/components/App.tsx`), Line (42), Column (10), Code (`TS2304`).

### 4.3 Python (`py_compile` / `pytest` / tracebacks)
- Parses Python stack traces and syntax errors:
  ```text
  File "app.py", line 25, in <module>
  ZeroDivisionError: division by zero
  ```
- Extracts: File (`app.py`), Line (25), Severity (`Error`), Code (`ZeroDivisionError`).

### 4.4 Diagnostic Deduplication
- `deduplicate_diagnostics()` aggregates cascading errors occurring at identical source file/line locations.
- Surfaces the primary root cause diagnostic to the agent prompt while summarizing repetitive cascading warnings, saving up to 85% of error tokens.
