# Computer & Browser Agent Architecture
**Universal Autonomous AI Workspace — Phase 3**
**Target OS:** Windows 10/11 | **Cross-Platform Ready:** Linux & macOS

---

## 1. System Vision
The Advanced Computer & Browser Agent enables autonomous, multi-step computer tasks by combining:
- **Direct OS & Application Control**: via Windows-MCP and native Win32/UIA interfaces.
- **Modern Web Automation**: via Playwright MCP with headless and interactive browser contexts.
- **Intelligent Tool Strategy Routing**: choosing Direct APIs, Shell/CLI, Coding Agent, Browser MCP, and Windows-MCP based on efficiency and reliability.
- **Deterministic Observe-Act-Verify Loop**: validating explicit post-conditions after every interaction.
- **Desktop Ownership Manager**: maintaining safe single-controller leases and instant user takeover.
- **Token Efficiency First**: replacing full-page screenshots with structured accessibility and DOM trees.

---

## 2. Layered Architecture

```mermaid
flowchart TD
    subgraph Presentation & UI Layer
        UI["Tauri 2 + React 18 UI"]
        CCP["Computer Control Panel & Ownership Status"]
        BW["Browser Workspace (Tabs & DOM View)"]
        AT["Action Timeline & Verification Status"]
        TD["Token & Latency Dashboard"]
    end

    subgraph Rust Agent Runtime Core
        Router["ToolStrategyRouter (Tiers 1-6)"]
        StateMachine["Action State Machine (Observe-Act-Verify)"]
        Verifier["ActionVerifier (Post-condition engine)"]
        TokenOpt["TokenOptimizer (Accessibility-first, Delta)"]
    end

    subgraph Security & Ownership Supervisor
        SecEngine["SecurityEngine (Permission Tiers)"]
        OwnershipMgr["DesktopOwnershipManager (Lease / Takeover / Emergency Stop)"]
        Audit["SQLite Security Audit Log"]
    end

    subgraph Execution & Tool Layer
        NativeTools["Native Tools (Filesystem, Coding, Terminal)"]
        WinMcpMgr["WindowsMcpManager (Windows-MCP Client)"]
        BrowserMcpMgr["BrowserMcpManager (Playwright MCP Client)"]
    end

    subgraph External Platforms
        WinOS["Windows 10/11 OS & Applications"]
        Browser["Chromium / Edge / Webkit Browser Engine"]
    end

    UI --> Router
    Router --> SecEngine
    SecEngine --> OwnershipMgr
    OwnershipMgr --> WinMcpMgr
    OwnershipMgr --> BrowserMcpMgr
    WinMcpMgr --> WinOS
    BrowserMcpMgr --> Browser
    WinOS --> StateMachine
    Browser --> StateMachine
    StateMachine --> Verifier
    Verifier --> TokenOpt
    TokenOpt --> UI
```

---

## 3. Core Subsystems

### 3.1 Smart Tool Strategy Router
Evaluates incoming goals and delegates tasks according to a strict priority hierarchy:
1. **Tier 1 — Direct API**: If an official REST/gRPC API is available, invoke it directly (lowest token cost, highest speed).
2. **Tier 2 — Filesystem / SQLite**: For reading, editing, searching files or querying databases, use native Rust file tools.
3. **Tier 3 — Shell / CLI / Coding Agent**: For compilation, testing, package management, git, use terminal or coding engines.
4. **Tier 4 — Browser Automation (Playwright MCP)**: For web research, form submissions, web applications, use DOM accessibility tree interaction.
5. **Tier 5 — Windows UI Automation (Windows-MCP)**: For desktop apps (Notepad, Office, IDEs, system settings), use UI automation trees and window switching.
6. **Tier 6 — Vision / Screenshot**: Visual recognition is only used when accessibility nodes are absent (e.g. canvas elements, game viewports).

### 3.2 Observe -> Interpret -> Act -> Verify State Machine
Every action progresses through a deterministic lifecycle:
```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Observing: Task Started
    Observing --> Interpreting: UI State / DOM Captured
    Interpreting --> PlanningAction: State Analyzed
    PlanningAction --> ValidatingOwnership: Action Selected
    ValidatingOwnership --> Executing: Lease Confirmed
    Executing --> Verifying: Action Dispatched
    Verifying --> Observing: Verification Succeeded (Next Step)
    Verifying --> Retrying: Verification Failed (Attempts < Max)
    Retrying --> PlanningAction: Strategy Adjusted
    Verifying --> Failed: Verification Failed (Attempts Exhausted)
    Observing --> Completed: Goal Verified
```

### 3.3 Desktop Ownership Management
- The AI never shares interactive input control simultaneously with the human user.
- **Lease Mechanism**: The AI requests a timed lease (10–30s).
- **User Takeover**: If the user moves the mouse or types, the lease is immediately revoked, and AI actions are paused or aborted.
- **Emergency Stop**: Releases all active leases, kills active MCP action routines, and returns the system to a clean state.
