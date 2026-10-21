# Resilience Patterns & Circuit Breakers

## 1. Error Classification Engine

The `ErrorClassifier` categorizes all operational and tool failures into distinct classes to dictate deterministic recovery workflows:

```mermaid
graph TD
    Err[Failure Event] --> Classify[ErrorClassifier::classify]
    
    Classify --> Transient["Transient\n(HTTP 429, 502, 503, 504, Connection Reset)\n-> Exponential Backoff with Jitter"]
    Classify --> Permanent["Permanent\n(HTTP 401, 403, 404, Syntax Error, Missing File)\n-> Terminate Step Immediately, Alert Operator"]
    Classify --> Resource["Resource\n(OOM, Disk Full, Quota Exhausted)\n-> Trigger Backpressure, Pause Non-Essential Tasks"]
    Classify --> UserAction["UserActionRequired\n(MFA Prompt, Permission Gate, Locked Desktop)\n-> Defer to WaitingApproval / WaitingForSession"]
    Classify --> Unknown["UnknownOutcome\n(Mutation Sent + Connection Dropped)\n-> Hold in OutcomeUnknown, Never Blindly Retry"]
```

## 2. Circuit Breaker State Machine

To prevent cascading failures across the multi-agent mesh, providers and workers are protected by `CircuitBreaker`:

```mermaid
stateDiagram-v2
    [*] --> Closed: Initial State (Normal Execution)
    Closed --> Open: Consecutive Failures >= threshold (e.g. 5)
    Open --> Open: Execution Denied (Fast Fail)
    Open --> HalfOpen: Cooldown Window Elapsed (e.g. 60s)
    HalfOpen --> Closed: Probe Request Succeeds
    HalfOpen --> Open: Probe Request Fails
```

### Parameters
- **Failure Threshold:** 3 to 5 consecutive failures.
- **Cooldown Window:** 30 to 120 seconds.
- **Probe Concurrency:** Exactly 1 probe request in `HalfOpen` state.
