# Multi-Agent Security & Prompt Injection Defense

## 1. Threat Model & Principles
In a Multi-Agent environment, untrusted external data (web pages visited by Browser Agent, third-party repositories analyzed by Coding Agent) can contain adversarial prompt injections or malicious instructions.

---

## 2. Security Invariants

### 2.1 Tool Outputs Are Strictly Untrusted Data
- Content returned from browsers, shells, or APIs is tagged as passive data payloads (`WebPayload`, `ToolOutput`).
- Untrusted data is never evaluated as system instructions or supervisor directives.

### 2.2 No Privilege Escalation Through Delegation
- A child worker receives a permission scope that is strictly equal to or narrower than the parent workflow's authorized scope.
- If a user has granted read-only permissions on a directory, the Supervisor cannot spawn a worker with write permissions on that directory.

### 2.3 Tool-Level Gateway Enforcement
- Security checks reside in the Rust runtime (`SecurityEngine`, `DesktopOwnershipManager`), completely independent of LLM system prompt compliance.
- Direct Win32 or process execution calls outside the workspace sandbox trigger `SecurityError::ApprovalRequired`.

### 2.4 Separation of Agent Identities
- Agent identities (`AgentType`, `id`) and message origins are cryptographically bound in the Rust message bus.
- A worker cannot impersonate the Supervisor or forge approval messages.
