# UI Redesign & Modernization Audit: Chat-First Desktop Experience

## 1. Executive Summary

This document outlines the architectural transition of the **Universal Autonomous AI Workspace** (`apps/desktop`) from a complex, multi-dashboard admin application into a **100% Chat-First, Ultra-Minimal, Native Desktop Workspace** inspired by the refined user experience of Claude Desktop, OpenAI Codex, and Google Antigravity.

## 2. Legacy Component Inventory & Replacement Strategy

| Legacy Component | Previous Purpose | New Chat-First Paradigm | Action |
| :--- | :--- | :--- | :--- |
| `Dashboard.tsx` | Heavy KPI cards, system status overview, agent counts | Invisible Complexity. App opens directly into chat; stats only in settings or contextual pills. | **Removed** |
| `Sidebar.tsx` | 13-item navigation bar switching between standalone views | Replaced by `ConversationSidebar.tsx` focused solely on conversation history, search, and new chat. | **Replaced** |
| `EmergencyStopBanner.tsx` | Persistent red banner across the top of every screen | Replaced by subtle Window Header pill and contextual inline alerts when triggered. | **Replaced** |
| `Tasks.tsx` | Standalone task manager & checkpoint inspector | Rendered contextually in-line within the chat stream when tasks execute. | **Removed** |
| `CodingWorkspace.tsx` | Separate Ask/Plan/Build IDE view with repo map & file explorer | Code generation, diff previews, and build/test executions appear directly in chat activity pills. | **Removed** |
| `ComputerControl.tsx` | Separate window list, screenshot viewer, lease controls | Desktop takeover & window interaction events appear as contextual action cards in the conversation. | **Removed** |
| `BrowserWorkspace.tsx` | Separate Playwright browser viewer & element inspector | Web browsing actions, snapshots, and search tasks appear contextually in chat activity pills. | **Removed** |
| `WorkflowOrchestrator.tsx` | Standalone DAG workflow and multi-agent manager | Multi-step workflows and DAG tasks run seamlessly in the background and report progress in chat. | **Removed** |
| `ResearchWorkspace.tsx` | Standalone sources, claims, and citation manager | Evidence items and sources are cited directly in the assistant's formatted markdown output. | **Removed** |
| `BusinessDashboard.tsx` | Standalone spreadsheet & data analysis views | Data summaries, tables, and charts are rendered natively via chat markdown. | **Removed** |
| `RuntimeControlCenter.tsx` | Standalone 24/7 Windows service and watchdog manager | Accessible via minimal Settings dialog; background service runs transparently. | **Removed** |
| `TerminalView.tsx` | Standalone shell command executor | Command execution runs via chat tool calls with expandable stdout/stderr blocks. | **Removed** |
| `Workspace.tsx` | Standalone file tree & editor | Files read and written appear as clean expandable widgets in chat stream. | **Removed** |
| `ApprovalDialog.tsx` | Full-screen modal interrupting the user | Replaced by `InlineApprovalCard.tsx` directly inside the conversation stream. | **Replaced** |
| `Settings.tsx` | Heavy 10-tab configuration panel | Replaced by `SettingsDialog.tsx`, a clean 3-tab modal dialog (Appearance, AI Model, Security). | **Replaced** |

## 3. New Component Architecture

```
apps/desktop/src/
├── App.tsx                     # Ultra-clean app shell, ThemeProvider, layout manager
├── types.ts                    # Core TypeScript definitions and DTOs
├── index.css                   # Refined CSS tokens for Light & Dark modes
├── context/
│   └── ThemeContext.tsx        # Light, Dark, System theme & bilingual (vi/en) state
├── components/
│   ├── WindowHeader.tsx        # Minimal window bar: Sidebar toggle, Title, New Chat, Stop, Settings
│   ├── ConversationSidebar.tsx # Collapsible drawer: Search, History grouped by date, Rename, Delete
│   ├── ChatView.tsx            # Main conversation timeline & auto-scroll engine
│   ├── ChatComposer.tsx        # Auto-resizing textarea, Enter to send, Vietnamese IME guard, Stop
│   ├── MessageItem.tsx         # Clean bubble-less user & assistant message renderers
│   ├── CodeBlock.tsx           # Syntax highlighting, language badge, one-click copy
│   ├── ContextualActivity.tsx  # Expandable inline cards for tool execution (build, test, file, shell, browser)
│   ├── InlineApprovalCard.tsx  # In-chat interactive card for permission authorization
│   └── SettingsDialog.tsx      # Minimal popover dialog: Appearance, AI Provider, Security
└── lib/
    └── tauriBridge.ts          # Safe Tauri IPC invoker & event listener with dev fallbacks
```

## 4. Design Standards & Token System

- **Dual-Theme Support**: Seamless native Light Mode and Dark Mode.
- **Minimalist Aesthetic**: Zero garish gradients, zero artificial AI glows, crisp 1px borders, subtle hover transitions.
- **Typography & Readability**: Clean sans-serif font stack, balanced whitespace, readable line heights.
- **Bilingual Interface**: Full Tiếng Việt (🇻🇳) and English (🇺🇸) localization persisted in `localStorage`.
- **Zero Backend Regression**: All Rust Core crates, SQLite storage, Windows SCM service, and test suites remain untouched and 100% operational.
