# Resource Management & Backpressure

## 1. System Resource Monitoring

The platform integrates the `sysinfo` crate (v0.33) in `agent-platform` to collect continuous hardware metrics without relying on external diagnostic executables:
- **CPU:** Real-time multi-core CPU utilization percentage.
- **RAM:** Total installed memory (MB), used memory (MB), free memory (MB), and percentage consumed.
- **Disk:** Available free storage (GB) on the primary workspace partition.
- **Processes:** Active system process count.

## 2. Thresholds & Backpressure Policies

| Resource | Warning Threshold | Critical Threshold | Automated Action |
| :--- | :--- | :--- | :--- |
| **System Memory** | > 80% RAM | > 90% RAM | **Backpressure Engaged:** New non-essential jobs are paused or deferred. In-memory LLM context caches are truncated. |
| **Disk Space** | < 5.0 GB Free | < 1.0 GB Free | Database backups and heavy artifact downloads are blocked. Warning banner displayed in UI. |
| **CPU Load** | > 85% CPU for 60s | > 95% CPU for 30s | Worker pool concurrency is throttled from $N$ to 1 until load stabilizes. |

## 3. Backpressure Mechanics
When `SystemResourceMonitor` detects `memory_used_percent > 90.0%`:
1. `AgentRuntimeService` signals running schedulers to defer non-critical jobs.
2. The `AutonomousScheduler` transitions queue intake to `Pausing`.
3. High-priority workflows are allowed to complete their immediate step and checkpoint to disk.
