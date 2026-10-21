# Budget Governance & Pre-Request Token Reservation

## 1. Multi-Agent Budget Race Condition

In multi-agent systems with concurrent workers (e.g. Research Agent, Coding Agent, and Data Analyst all running simultaneously):
- If workers only check their budget *after* making LLM inference requests, multiple workers checking at the same time will all see sufficient remaining budget.
- Each worker dispatches large prompts simultaneously, causing cumulative expenditure to blow past daily, monthly, or workflow caps by hundreds of percent.

## 2. Pre-Request Reservation Architecture

To eliminate parallel budget races, `BudgetGovernanceEngine` enforces **two-phase budget allocation**:

```mermaid
sequenceDiagram
    participant Worker as Worker Agent
    participant Engine as BudgetGovernanceEngine
    participant LLM as Model Provider API

    Worker->>Engine: reserve(entity_type, entity_id, estimated_tokens, estimated_cost)
    Note over Engine: Check: Committed Spent + Active Pending Reservations + Requested <= Limit
    alt Budget Exceeded
        Engine-->>Worker: Err("Budget exceeded: limit reached")
        Worker->>Worker: Task transitions to WaitingBudget
    else Budget Available
        Engine-->>Worker: Ok(ReservationReceipt { id, reserved_cost, expires_at })
        Worker->>LLM: Dispatch Prompt Request
        LLM-->>Worker: Model Response (actual_tokens, actual_cost)
        Worker->>Engine: reconcile(receipt, actual_tokens, actual_cost)
        Note over Engine: Deduct reservation, commit actual_cost to daily/monthly totals
    end
```

## 3. Failure & Expiration Handling
- **Failure Before Sending:** If a network error or permission block occurs before reaching the model, the worker calls `engine.release(receipt.id)`. The reserved funds are instantly returned to the available pool.
- **Worker Hang / Crash:** Reservations include a 5-minute TTL (`expires_at`). Expired reservations are automatically purged during periodic cleanup cycles.
