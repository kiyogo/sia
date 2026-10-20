# Third-Party Source Code Management Rules

## 1. Scope & Objective
This directory and policy document define the guidelines for evaluating, vetting, ingesting, and maintaining third-party or open-source dependencies within the Universal Autonomous AI Workspace.

## 2. Ingestion Rules
1. **License Pre-Verification**:
   - Only permissive open-source licenses are permitted (MIT, Apache-2.0, ISC, BSD-2/3-Clause).
   - Copyleft licenses (GPL, AGPL, LGPL) are prohibited from static inclusion in core binaries.
2. **Attribution Preservation**:
   - Every vendored file or adapter must retain the original copyright notice and license header.
   - All references must be logged in `docs/OPEN_SOURCE_INVENTORY.md` and `THIRD_PARTY_NOTICES.md`.
3. **Decoupled Architecture**:
   - Prioritize crate dependencies or out-of-process protocol adapters (e.g., Model Context Protocol over stdio) rather than copying full third-party codebases into the repository.
   - Keep platform boundaries modular so the core remains cross-platform ready.
