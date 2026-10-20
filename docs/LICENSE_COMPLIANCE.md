# Open Source License Compliance Report

## 1. Compliance Principles
1. **Accurate Licensing**: All direct dependencies and upstream projects were audited at their exact published versions.
2. **Attribution Integrity**: Copyright statements and author attributions (Jeomon George for Windows-MCP, opencode for OpenCode, Tauri Programme for Tauri, etc.) are strictly maintained in `THIRD_PARTY_NOTICES.md`.
3. **No Copyleft Contamination**: No GPL/AGPL licensed libraries or code snippets have been statically linked or incorporated into the workspace. All dependencies are permissive (MIT, Apache-2.0, ISC, BSD-3-Clause, Public Domain).
4. **Trademark & Brand Protection**: No trademarks, logos, or brand identifiers of third-party projects (e.g. OpenCode, Cursor, Microsoft) are used as the product name or primary branding.
5. **Clear Separation of Concerns**: Windows-MCP is connected as a decoupled client-server protocol over stdio JSON-RPC 2.0.

## 2. Dependency License Breakdown

| License Type | Crates / Packages | Compatibility Status |
| :--- | :--- | :--- |
| **MIT** | `tokio`, `serde`, `serde_json`, `uuid`, `chrono`, `rusqlite`, `react`, `lucide-react`, `tailwindcss` | **COMPLIANT** — Compatible with MIT/Apache-2.0 distribution |
| **Apache-2.0** | `similar`, portions of `reqwest`, `tauri` | **COMPLIANT** — Proper NOTICE maintained |
| **ISC** | `lucide-react` | **COMPLIANT** — Standard permissive attribution |
| **Public Domain** | SQLite core C engine | **COMPLIANT** — Unrestricted reuse |

## 3. Upstream Code Ingestion Rules
Any future contributions or external code imports must:
- Record entry in `docs/OPEN_SOURCE_INVENTORY.md`.
- Preserve the original author's copyright header.
- Provide clean wrapper traits rather than inline modifications to upstream code.
