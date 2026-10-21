# Self-Recovery Engine & Side-Effect Containment

## 1. Startup Crash Reconciliation

When `AgentRuntimeService` starts, the `CrashRecoveryEngine` scans the SQLite database for tasks left in an in-flight state (`Running`, `Assigned`, `Checkpointing`).

The engine classifies each interrupted task based on whether it entails **external side effects**:

```mermaid
flowchart TD
    Scan["Scan In-Flight Tasks on Startup"] --> Eval{"Retry Count >= Max Retries?"}
    Eval -- "Yes" --> MarkFail["Mark Task Status: Failed"]
    
    Eval -- "No" --> CheckSideEffect{"Has External Side Effects?\n(Email, Payment, Commit, Delete)"}
    
    CheckSideEffect -- "No (Pure / Idempotent)" --> RecoverReady["Reset Status: Ready\nIncrement retry_count\nRe-enqueue for execution"]
    
    CheckSideEffect -- "Yes (External Side Effect)" --> HoldUnknown["Mark Status: OutcomeUnknown\nRecord RuntimeEventRecord\nHold for Operator Verification"]
    
    HoldUnknown --> OperatorChoice{"Operator Review in UI"}
    OperatorChoice -- "Action Succeeded in Real World" --> Succeeded["UnknownReconciliationDecision::MarkCompleted"]
    OperatorChoice -- "Action Did Not Execute" --> Retry["UnknownReconciliationDecision::RetrySafeStep"]
    OperatorChoice -- "Uncertain / Unsafe" --> Cancel["UnknownReconciliationDecision::Cancel"]
```

## 2. Preventing Duplicate Side Effects
Blindly retrying tasks after a power cut or crash is catastrophic for external mutations:
- Sending duplicate emails to customers.
- Double-charging credit cards or initiating duplicate bank transfers.
- Deleting files twice or committing broken git branches.

By placing interrupted side-effect tasks into `OutcomeUnknown`, the system guarantees **at-most-once execution** for non-idempotent operations until human operator verification.

## 3. Operator Reconciliation Actions
Inside `RuntimeControlCenter.tsx` (Recovery tab), operators can review held tasks and apply one of three decisions:
1. `RetrySafeStep`: Resets the task to `Ready` and increments `retry_count`.
2. `MarkCompleted`: Marks the task `Completed` and allows downstream dependent tasks to unblock.
3. `Cancel`: Marks the task `Failed` / `Cancelled` and safely halts the workflow DAG.
