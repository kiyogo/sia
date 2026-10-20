# Phase 3 Open-Source Reuse & Compliance Report
**Universal Autonomous AI Workspace**
**Target OS:** Windows 10/11 | **Phase:** 03 — Advanced Computer & Browser Agent

---

## 1. Executive Summary
In strict compliance with our Open-Source-First and Production-Ready philosophy, Phase 3 maximizes the direct reuse of mature, battle-tested open-source libraries and protocols, avoiding custom wheel reinvention for OS UI automation and headless browser engines.

This report documents every external package, its license, upstream source, version, and the architectural boundary through which it integrates into the Rust Core runtime.

---

## 2. Inventory of Reused Technologies in Phase 3

### 2.1 Primary Desktop Automation: Windows-MCP
- **Repository:** `https://github.com/CursorTouch/Windows-MCP`
- **Version:** `0.8.8` (Manifest version `0.4`)
- **Author:** CursorTouch / Jeomon George
- **License:** MIT License (Permissive, commercial and private use allowed)
- **Integration Mechanism:** Process boundary via stdio JSON-RPC 2.0 (MCP Protocol standard `2024-11-05`). Rust runtime acts as MCP host supervisor.
- **Capabilities Reused:**
  - Windows UI Automation (UIA) tree walker and COM interfaces.
  - Win32 SendInput dispatch for deterministic mouse coordinates, clicks, drags, and keyboard shortcuts.
  - Multi-display coordinate resolution and DPI scale factor mapping.
  - Window manager (EnumWindows, SetForegroundWindow, ShowWindow).
  - Screen capture backends (DXGI Desktop Duplication API, MSS, Pillow fallback).

### 2.2 Primary Browser Automation: Playwright MCP
- **Package:** `@playwright/mcp`
- **Version:** `0.0.83` (Playwright Core `1.64.0-alpha`)
- **Author:** Microsoft Corporation & Open-Source Contributors
- **License:** Apache-2.0 License (Permissive with patent grant and notice requirements)
- **Integration Mechanism:** Process boundary via stdio JSON-RPC 2.0. Rust runtime acts as client supervisor.
- **Capabilities Reused:**
  - Chromium/WebKit/Firefox browser process lifecycle management.
  - Accessibility tree snapshot generation with unique element references.
  - Form filling, input event dispatch, select option controls.
  - Local HTTP and file-based navigation, cookie/storage isolation.
  - Multi-tab management, file upload, and download lifecycle events.

### 2.3 Model Context Protocol Specification
- **Specification:** Model Context Protocol (MCP) by Anthropic / Open-Source Community
- **Protocol Version:** `2024-11-05`
- **License:** MIT License
- **Integration Mechanism:** Clean Rust native implementation in `crates/agent-tools/src/mcp_client.rs` using Tokio async pipes (`tokio::process::Command`, `tokio::io::BufReader`).

### 2.4 Existing Phase 1 & 2 Reused Assets
- **Tauri 2**: Apache-2.0 / MIT (Desktop windowing & IPC bridge)
- **Tokio**: MIT (Async runtime, child process spawning, pipe streaming)
- **SQLite (rusqlite)**: MIT (Embedded transactional database)
- **React 18 & Lucide React**: MIT (Frontend component architecture & iconography)
- **Similar**: Apache-2.0 (Unified diff calculation and patch preview)
- **Ignore / Walkdir**: Unlicense / MIT (Gitignore-aware filesystem traversal)

---

## 3. License Compatibility Matrix

| Project | License | Compatible with Our Project? | Attribution Required? | Action Taken |
|---|---|---|---|---|
| **Windows-MCP** | MIT | Yes | Yes | Retain copyright notice in `THIRD_PARTY_NOTICES.md` |
| **Playwright MCP** | Apache-2.0 | Yes | Yes | Include Apache 2.0 notice in `THIRD_PARTY_NOTICES.md` |
| **Playwright Core** | Apache-2.0 | Yes | Yes | Covered under Microsoft Playwright Apache 2.0 notice |
| **Tokio** | MIT | Yes | Yes | Retained in Cargo dependencies |
| **Tauri 2** | MIT / Apache-2.0 | Yes | Yes | Retained in Desktop harness |

---

## 4. Compliance Verification
- No GPL / AGPL contaminated dependencies are linked into the binary.
- All reused components communicate through well-defined process boundaries (stdio IPC) or permissive Cargo crate imports.
- `THIRD_PARTY_NOTICES.md` and `docs/OPEN_SOURCE_INVENTORY.md` are updated to reflect the inclusion of Windows-MCP and Playwright MCP.
