# Durable Execution Engine & State Machine

## 1. Storage & Concurrency Model

The Durable Execution Engine guarantees that any workflow, once accepted, will run to completion or safe containment regardless of software crashes, OS reboots, or hardware interruptions.

### SQLite WAL Mode
- **Concurrency:** Write-Ahead Logging (`PRAGMA journal_mode=WAL;`) allows concurrent readers (Tauri UI, monitoring dæmons) while the background service writes.
- **Synchronous Mode:** `PRAGMA synchronous=NORMAL;` balances ACID durability with high write throughput.
- **Fencing Tokens:** Every task claim and mutation uses monotonic versioning (`retry_count`) to detect and reject stale workers.

## 2. Expanded Task State Machine

Phase 6 introduces resilient states for network, provider, budget, session, and recovery boundaries:

```mermaid
stateDiagram-v2
    [*] --> Pending
    Pending --> Ready: Dependencies Met
    Ready --> Assigned: Claimed by Worker
    Assigned --> Running: Worker Heartbeat Active
    
    Running --> WaitingNetwork: Network Outage Detected
    WaitingNetwork --> Running: Connectivity Restored
    
    Running --> WaitingProvider: Provider 429 / 503
    WaitingProvider --> Running: Cooldown Elapsed
    
    Running --> WaitingBudget: Budget Reservation Reached
    WaitingBudget --> Running: Budget Replenished
    
    Running --> WaitingForInteractiveSession: Session 0 / Desktop Locked
    WaitingForInteractiveSession --> Running: User Logs In / Unlocks
    
    Running --> Pausing: Operator / Backpressure
    Pausing --> Paused: Checkpoint Saved
    Paused --> Ready: Operator Resumed
    
    Running --> Recovering: System Restart Recovery
    Recovering --> Ready: Pure / Idempotent Task
    Recovering --> OutcomeUnknown: Side-Effect Task Interrupted
    
    OutcomeUnknown --> Completed: Operator Verified Succeeded
    OutcomeUnknown --> Ready: Operator Decided Safe to Retry
    OutcomeUnknown --> Failed: Operator Aborted
    
    Running --> Verifying: ActionVerifier Evaluates Criteria
    Verifying --> Completed: Acceptance Criteria Satisfied
    Verifying --> Failed: Retry Limit Exceeded
```

## 3. Atomic Task Claiming

To prevent race conditions among multiple worker threads:
1. Worker queries tasks with `status = 'Ready'`.
2. Worker executes an atomic conditional update:
   ```sql
   UPDATE workflow_tasks
   SET status = 'Running', started_at = datetime('now')
   WHERE id = ?1 AND status = 'Ready'
   ```
3. Only the worker that successfully updates the row proceeds with execution.
