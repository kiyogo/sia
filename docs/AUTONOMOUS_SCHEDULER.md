# Autonomous Scheduler Architecture

## 1. Overview

The `AutonomousScheduler` coordinates unattended periodic and time-based tasks in SQLite. Unlike memory-only schedulers, job schedules and past execution records persist permanently across system reboots.

## 2. Supported Schedule Models
- **Interval:** Runs every $N$ seconds (e.g. 3600s for hourly sync).
- **Daily:** Runs at a fixed time each day.
- **Weekly / Monthly:** Triggers on specified weekdays or calendar dates.
- **Cron Expressions:** Standard 5-field cron parsing (e.g. `0 9 * * 1-5` for business hours).

## 3. Missed-Run Policies

When a computer is powered off, in sleep mode, or the service is stopped, scheduled jobs may miss their due window. The scheduler evaluates the `missed_run_policy`:

| Policy | Behavior on Recovery | Recommended Use Case |
| :--- | :--- | :--- |
| **Skip** | Ignores overdue executions and advances `next_run_at` to the next future interval. Prevents backlog storms. | Health checks, live telemetry, market price polling |
| **RunOnce** | Executes exactly one catch-up run immediately upon service resumption, then resumes normal intervals. | Daily reports, email digests, backup jobs |
| **CatchUp** | Runs every individual missed execution sequentially. | Financial ledger reconciliation, audit event sync |
| **RequireApproval** | Suspends the job in `WaitingApproval` and alerts the operator via Desktop UI. | Large batch updates, bulk data export |

## 4. Overlap & Concurrency Policies

If a previous run is still executing when the next scheduled time arrives:
- **Skip (Default):** Skips the new trigger cycle. Prevents piling up duplicate tasks.
- **Queue:** Defers the execution until the running job finishes.
- **Replace:** Cancels the current execution and launches a fresh run.
- **RunConcurrently:** Launches an independent execution in parallel (requires separate budget allocation).
