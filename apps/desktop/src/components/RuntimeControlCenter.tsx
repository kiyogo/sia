import React, { useState, useEffect } from 'react';
import {
  ScheduledJobDto,
  SupervisedProcessDto,
  SystemResourceSnapshotDto,
  BudgetStatusDto,
} from '../types';
import { invokeCommand } from '../lib/tauriBridge';
import {
  Activity,
  CheckCircle2,
  Clock,
  Cpu,
  Database,
  Play,
  RefreshCw,
  RotateCcw,
  Server,
  Shield,
  Trash2,
} from 'lucide-react';

export const RuntimeControlCenter: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'service' | 'scheduler' | 'watchdog' | 'recovery'>('service');
  const [serviceStatus, setServiceStatus] = useState<string>('Stopped');
  const [resourceSnapshot, setResourceSnapshot] = useState<SystemResourceSnapshotDto | null>(null);
  const [budgetStatus, setBudgetStatus] = useState<BudgetStatusDto | null>(null);
  const [scheduledJobs, setScheduledJobs] = useState<ScheduledJobDto[]>([]);
  const [supervisedProcesses, setSupervisedProcesses] = useState<SupervisedProcessDto[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  // New Job Form State
  const [newJobName, setNewJobName] = useState<string>('Daily Market Report');
  const [newJobType, setNewJobType] = useState<string>('interval');
  const [newJobInterval, setNewJobInterval] = useState<number>(3600);
  const [newJobTemplate, setNewJobTemplate] = useState<string>('market_research');

  // Close behavior setting
  const [closeBehavior, setCloseBehavior] = useState<string>('keep_service');

  useEffect(() => {
    refreshAll();
    const interval = setInterval(refreshAll, 4000);
    return () => clearInterval(interval);
  }, []);

  const refreshAll = async () => {
    try {
      const s = await invokeCommand<string>('get_service_status');
      setServiceStatus(s);

      const res = await invokeCommand<SystemResourceSnapshotDto>('get_system_resource_snapshot');
      setResourceSnapshot(res);

      const b = await invokeCommand<BudgetStatusDto>('get_budget_status');
      setBudgetStatus(b);

      const jobs = await invokeCommand<ScheduledJobDto[]>('list_scheduled_jobs');
      setScheduledJobs(jobs);

      const procs = await invokeCommand<SupervisedProcessDto[]>('list_supervised_processes');
      setSupervisedProcesses(procs);
    } catch (e) {
      console.error('Failed to refresh runtime stats', e);
    }
  };

  const handleServiceAction = async (action: 'install' | 'start' | 'stop' | 'restart') => {
    setLoading(true);
    setStatusMessage(null);
    try {
      let res = '';
      if (action === 'install') {
        res = await invokeCommand<string>('install_windows_service', {});
      } else if (action === 'start') {
        res = await invokeCommand<string>('start_windows_service', {});
      } else if (action === 'stop') {
        res = await invokeCommand<string>('stop_windows_service', {});
      } else if (action === 'restart') {
        res = await invokeCommand<string>('restart_windows_service', {});
      }
      setStatusMessage(res);
      await refreshAll();
    } catch (e: any) {
      setStatusMessage(`Action failed: ${e?.message || e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateJob = async () => {
    setLoading(true);
    try {
      await invokeCommand<string>('create_scheduled_job', {
        name: newJobName,
        schedule_type: newJobType,
        interval_seconds: newJobInterval,
        workflow_template: newJobTemplate,
      });
      setStatusMessage(`Created scheduled job: ${newJobName}`);
      await refreshAll();
    } catch (e: any) {
      setStatusMessage(`Job creation failed: ${e?.message || e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleTriggerJob = async () => {
    try {
      const triggered = await invokeCommand<any[]>('trigger_scheduled_job_now');
      setStatusMessage(`Triggered ${triggered.length} due jobs`);
      await refreshAll();
    } catch (e: any) {
      setStatusMessage(`Trigger failed: ${e?.message || e}`);
    }
  };

  const handleKillProcess = async (uuid: string) => {
    try {
      await invokeCommand('kill_supervised_process', { process_uuid: uuid });
      setStatusMessage(`Terminated process ${uuid}`);
      await refreshAll();
    } catch (e: any) {
      setStatusMessage(`Kill failed: ${e?.message || e}`);
    }
  };

  const handleBackupNow = async () => {
    setLoading(true);
    try {
      const path = await invokeCommand<string>('backup_database_now', {});
      setStatusMessage(`Database backup created: ${path}`);
    } catch (e: any) {
      setStatusMessage(`Backup failed: ${e?.message || e}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex-1 flex flex-col h-full bg-[#181825] text-[#cdd6f4] p-6 overflow-y-auto">
      {/* Header */}
      <div className="flex items-center justify-between pb-6 border-b border-[#313244]">
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-3">
            <Server className="w-7 h-7 text-[#89b4fa]" />
            24/7 Autonomous Runtime & Control Center
          </h1>
          <p className="text-sm text-[#a6adc8] mt-1">
            Continuous background execution, Windows Service SCM, durable scheduler, process watchdog, and self-recovery
          </p>
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={handleBackupNow}
            disabled={loading}
            className="flex items-center gap-2 px-3 py-2 bg-[#313244] hover:bg-[#45475a] text-white rounded text-sm transition"
          >
            <Database className="w-4 h-4 text-[#a6e3a1]" />
            Backup DB
          </button>
          <button
            onClick={refreshAll}
            className="flex items-center gap-2 px-3 py-2 bg-[#313244] hover:bg-[#45475a] text-white rounded text-sm transition"
          >
            <RefreshCw className="w-4 h-4" />
            Refresh
          </button>
        </div>
      </div>

      {statusMessage && (
        <div className="mt-4 p-3 bg-[#45475a] text-white rounded text-sm flex items-center justify-between">
          <span>{statusMessage}</span>
          <button onClick={() => setStatusMessage(null)} className="text-xs text-[#a6adc8] hover:text-white">✕</button>
        </div>
      )}

      {/* Metrics Banner */}
      <div className="grid grid-cols-4 gap-4 mt-6">
        <div className="bg-[#1e1e2e] p-4 rounded-lg border border-[#313244]">
          <div className="flex items-center justify-between text-[#a6adc8] text-xs">
            <span>SERVICE LIFECYCLE</span>
            <Activity className="w-4 h-4 text-[#89b4fa]" />
          </div>
          <div className="mt-2 flex items-center gap-2">
            <span
              className={`w-3 h-3 rounded-full ${
                serviceStatus === 'Running'
                  ? 'bg-[#a6e3a1]'
                  : serviceStatus === 'StartPending'
                  ? 'bg-[#f9e2af]'
                  : 'bg-[#f38ba8]'
              }`}
            />
            <span className="text-xl font-bold">{serviceStatus}</span>
          </div>
          <p className="text-xs text-[#6c7086] mt-1">Independent of Tauri UI window</p>
        </div>

        <div className="bg-[#1e1e2e] p-4 rounded-lg border border-[#313244]">
          <div className="flex items-center justify-between text-[#a6adc8] text-xs">
            <span>SYSTEM HARDWARE</span>
            <Cpu className="w-4 h-4 text-[#fab387]" />
          </div>
          <div className="mt-2 flex items-baseline gap-2">
            <span className="text-xl font-bold">
              {resourceSnapshot ? `${resourceSnapshot.cpu_usage_percent.toFixed(1)}%` : '--'}
            </span>
            <span className="text-xs text-[#a6adc8]">
              {resourceSnapshot ? `${resourceSnapshot.memory_used_mb}MB / ${resourceSnapshot.memory_total_mb}MB` : ''}
            </span>
          </div>
          <p className="text-xs text-[#6c7086] mt-1">
            Disk Free: {resourceSnapshot ? `${resourceSnapshot.disk_free_gb.toFixed(1)} GB` : '--'}
          </p>
        </div>

        <div className="bg-[#1e1e2e] p-4 rounded-lg border border-[#313244]">
          <div className="flex items-center justify-between text-[#a6adc8] text-xs">
            <span>TOKEN BUDGET (DAILY)</span>
            <Shield className="w-4 h-4 text-[#a6e3a1]" />
          </div>
          <div className="mt-2 flex items-baseline gap-2">
            <span className="text-xl font-bold">
              ${budgetStatus ? budgetStatus.daily_spent.toFixed(2) : '0.00'}
            </span>
            <span className="text-xs text-[#a6adc8]">/ ${budgetStatus?.daily_limit.toFixed(2) || '25.00'}</span>
          </div>
          <p className="text-xs text-[#6c7086] mt-1">
            Active Reserved: ${budgetStatus ? budgetStatus.active_reserved.toFixed(2) : '0.00'}
          </p>
        </div>

        <div className="bg-[#1e1e2e] p-4 rounded-lg border border-[#313244]">
          <div className="flex items-center justify-between text-[#a6adc8] text-xs">
            <span>SCHEDULED & SUPERVISED</span>
            <Clock className="w-4 h-4 text-[#cba6f7]" />
          </div>
          <div className="mt-2 flex items-baseline gap-2">
            <span className="text-xl font-bold">{scheduledJobs.length}</span>
            <span className="text-xs text-[#a6adc8]">Jobs | {supervisedProcesses.length} Procs</span>
          </div>
          <p className="text-xs text-[#6c7086] mt-1">Auto-recovery & Watchdog Active</p>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 mt-6 border-b border-[#313244]">
        <button
          onClick={() => setActiveTab('service')}
          className={`pb-3 px-4 text-sm font-medium border-b-2 transition ${
            activeTab === 'service'
              ? 'border-[#89b4fa] text-[#89b4fa]'
              : 'border-transparent text-[#a6adc8] hover:text-white'
          }`}
        >
          Windows Service & SCM
        </button>
        <button
          onClick={() => setActiveTab('scheduler')}
          className={`pb-3 px-4 text-sm font-medium border-b-2 transition ${
            activeTab === 'scheduler'
              ? 'border-[#89b4fa] text-[#89b4fa]'
              : 'border-transparent text-[#a6adc8] hover:text-white'
          }`}
        >
          Autonomous Scheduler ({scheduledJobs.length})
        </button>
        <button
          onClick={() => setActiveTab('watchdog')}
          className={`pb-3 px-4 text-sm font-medium border-b-2 transition ${
            activeTab === 'watchdog'
              ? 'border-[#89b4fa] text-[#89b4fa]'
              : 'border-transparent text-[#a6adc8] hover:text-white'
          }`}
        >
          Process Watchdog ({supervisedProcesses.length})
        </button>
        <button
          onClick={() => setActiveTab('recovery')}
          className={`pb-3 px-4 text-sm font-medium border-b-2 transition ${
            activeTab === 'recovery'
              ? 'border-[#89b4fa] text-[#89b4fa]'
              : 'border-transparent text-[#a6adc8] hover:text-white'
          }`}
        >
          Self-Recovery & Reconciliation
        </button>
      </div>

      {/* Tab 1: Service Manager */}
      {activeTab === 'service' && (
        <div className="mt-6 space-y-6">
          <div className="bg-[#1e1e2e] p-6 rounded-lg border border-[#313244]">
            <h2 className="text-lg font-semibold flex items-center gap-2">
              <Server className="w-5 h-5 text-[#89b4fa]" />
              Windows Service Control Manager (SCM) Integration
            </h2>
            <p className="text-sm text-[#a6adc8] mt-1">
              Configure and manage the background AI service. When installed, the runtime boots automatically with Windows, independent of whether the desktop GUI is running.
            </p>

            <div className="flex items-center gap-4 mt-6">
              <button
                onClick={() => handleServiceAction('install')}
                disabled={loading}
                className="px-4 py-2 bg-[#89b4fa] hover:bg-[#b4befe] text-[#11111b] font-medium rounded text-sm transition"
              >
                Install Service (Admin)
              </button>
              <button
                onClick={() => handleServiceAction('start')}
                disabled={loading}
                className="px-4 py-2 bg-[#a6e3a1] hover:bg-[#94e2d5] text-[#11111b] font-medium rounded text-sm transition"
              >
                Start Service
              </button>
              <button
                onClick={() => handleServiceAction('stop')}
                disabled={loading}
                className="px-4 py-2 bg-[#f38ba8] hover:bg-[#eba0ac] text-[#11111b] font-medium rounded text-sm transition"
              >
                Stop Service
              </button>
              <button
                onClick={() => handleServiceAction('restart')}
                disabled={loading}
                className="px-4 py-2 bg-[#313244] hover:bg-[#45475a] text-white font-medium rounded text-sm transition flex items-center gap-2"
              >
                <RotateCcw className="w-4 h-4" />
                Restart
              </button>
            </div>
          </div>

          <div className="bg-[#1e1e2e] p-6 rounded-lg border border-[#313244]">
            <h3 className="text-md font-semibold">Desktop Window Close Behavior</h3>
            <p className="text-sm text-[#a6adc8] mt-1">
              Choose how the runtime responds when the Tauri desktop window is closed by the user:
            </p>

            <div className="space-y-3 mt-4">
              <label className="flex items-center gap-3 text-sm cursor-pointer">
                <input
                  type="radio"
                  name="closeBehavior"
                  value="keep_service"
                  checked={closeBehavior === 'keep_service'}
                  onChange={(e) => setCloseBehavior(e.target.value)}
                  className="accent-[#89b4fa]"
                />
                <span>
                  <strong>Keep background service active (24/7 Mode):</strong> Tasks, schedules, and workers continue in background.
                </span>
              </label>

              <label className="flex items-center gap-3 text-sm cursor-pointer">
                <input
                  type="radio"
                  name="closeBehavior"
                  value="tray"
                  checked={closeBehavior === 'tray'}
                  onChange={(e) => setCloseBehavior(e.target.value)}
                  className="accent-[#89b4fa]"
                />
                <span>
                  <strong>Minimize to system notification tray:</strong> Quick access from taskbar.
                </span>
              </label>

              <label className="flex items-center gap-3 text-sm cursor-pointer">
                <input
                  type="radio"
                  name="closeBehavior"
                  value="shutdown"
                  checked={closeBehavior === 'shutdown'}
                  onChange={(e) => setCloseBehavior(e.target.value)}
                  className="accent-[#89b4fa]"
                />
                <span>
                  <strong>Stop service on exit:</strong> Halt all background executions upon window close.
                </span>
              </label>
            </div>
          </div>
        </div>
      )}

      {/* Tab 2: Scheduler */}
      {activeTab === 'scheduler' && (
        <div className="mt-6 space-y-6">
          {/* Create Job Form */}
          <div className="bg-[#1e1e2e] p-6 rounded-lg border border-[#313244]">
            <h3 className="text-md font-semibold">Add New Scheduled Job</h3>
            <div className="grid grid-cols-4 gap-4 mt-4">
              <div>
                <label className="text-xs text-[#a6adc8]">Job Name</label>
                <input
                  type="text"
                  value={newJobName}
                  onChange={(e) => setNewJobName(e.target.value)}
                  className="w-full bg-[#11111b] border border-[#45475a] rounded p-2 text-sm mt-1"
                />
              </div>

              <div>
                <label className="text-xs text-[#a6adc8]">Schedule Type</label>
                <select
                  value={newJobType}
                  onChange={(e) => setNewJobType(e.target.value)}
                  className="w-full bg-[#11111b] border border-[#45475a] rounded p-2 text-sm mt-1"
                >
                  <option value="interval">Interval (Every X seconds)</option>
                  <option value="daily">Daily</option>
                  <option value="weekly">Weekly</option>
                  <option value="once">Run Once</option>
                </select>
              </div>

              <div>
                <label className="text-xs text-[#a6adc8]">Interval (Seconds)</label>
                <input
                  type="number"
                  value={newJobInterval}
                  onChange={(e) => setNewJobInterval(Number(e.target.value))}
                  className="w-full bg-[#11111b] border border-[#45475a] rounded p-2 text-sm mt-1"
                />
              </div>

              <div>
                <label className="text-xs text-[#a6adc8]">Workflow Template</label>
                <select
                  value={newJobTemplate}
                  onChange={(e) => setNewJobTemplate(e.target.value)}
                  className="w-full bg-[#11111b] border border-[#45475a] rounded p-2 text-sm mt-1"
                >
                  <option value="market_research">Market Research</option>
                  <option value="data_reporting">Data Reporting</option>
                  <option value="email_assistance">Email Assistance</option>
                  <option value="research_to_document">Research to Document</option>
                  <option value="hybrid_pipeline">Hybrid Enterprise Pipeline</option>
                </select>
              </div>
            </div>

            <div className="flex items-center justify-between mt-4">
              <button
                onClick={handleCreateJob}
                disabled={loading}
                className="px-4 py-2 bg-[#89b4fa] hover:bg-[#b4befe] text-[#11111b] font-medium rounded text-sm transition"
              >
                Schedule Job
              </button>

              <button
                onClick={handleTriggerJob}
                className="px-4 py-2 bg-[#313244] hover:bg-[#45475a] text-white rounded text-sm transition flex items-center gap-2"
              >
                <Play className="w-4 h-4 text-[#a6e3a1]" />
                Trigger Due Jobs Now
              </button>
            </div>
          </div>

          {/* Job List */}
          <div className="bg-[#1e1e2e] p-6 rounded-lg border border-[#313244]">
            <h3 className="text-md font-semibold">Active & Scheduled Jobs</h3>
            {scheduledJobs.length === 0 ? (
              <p className="text-sm text-[#6c7086] mt-4">No recurring jobs configured.</p>
            ) : (
              <div className="mt-4 overflow-x-auto">
                <table className="w-full text-left text-sm">
                  <thead className="bg-[#11111b] text-[#a6adc8]">
                    <tr>
                      <th className="p-3">Job Name</th>
                      <th className="p-3">Type</th>
                      <th className="p-3">Template</th>
                      <th className="p-3">Next Run (UTC)</th>
                      <th className="p-3">Status</th>
                      <th className="p-3">Policy</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[#313244]">
                    {scheduledJobs.map((j) => (
                      <tr key={j.id} className="hover:bg-[#313244]/50">
                        <td className="p-3 font-medium">{j.name}</td>
                        <td className="p-3 text-xs">{j.schedule_type} ({j.interval_seconds || 0}s)</td>
                        <td className="p-3 text-xs text-[#89b4fa]">{j.workflow_template}</td>
                        <td className="p-3 text-xs">{j.next_run_at ? new Date(j.next_run_at).toLocaleString() : 'N/A'}</td>
                        <td className="p-3">
                          <span
                            className={`px-2 py-0.5 rounded text-xs ${
                              j.status === 'Active'
                                ? 'bg-[#a6e3a1]/20 text-[#a6e3a1]'
                                : 'bg-[#f38ba8]/20 text-[#f38ba8]'
                            }`}
                          >
                            {j.status}
                          </span>
                        </td>
                        <td className="p-3 text-xs text-[#a6adc8]">{j.missed_run_policy}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Tab 3: Process Watchdog */}
      {activeTab === 'watchdog' && (
        <div className="mt-6 space-y-6">
          <div className="bg-[#1e1e2e] p-6 rounded-lg border border-[#313244]">
            <h3 className="text-md font-semibold">Process Supervisor & Health Watchdog</h3>
            <p className="text-sm text-[#a6adc8] mt-1">
              Tracks sub-worker processes (Windows-MCP, Playwright MCP, OpenCode, background jobs). Automatic circuit breakers prevent crash loops (max 3 restarts per 60s).
            </p>

            {supervisedProcesses.length === 0 ? (
              <p className="text-sm text-[#6c7086] mt-4">No active child processes currently supervised.</p>
            ) : (
              <div className="mt-4 overflow-x-auto">
                <table className="w-full text-left text-sm">
                  <thead className="bg-[#11111b] text-[#a6adc8]">
                    <tr>
                      <th className="p-3">PID</th>
                      <th className="p-3">Type</th>
                      <th className="p-3">Health Status</th>
                      <th className="p-3">Restarts</th>
                      <th className="p-3">Last Heartbeat</th>
                      <th className="p-3">Action</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[#313244]">
                    {supervisedProcesses.map((p) => (
                      <tr key={p.id} className="hover:bg-[#313244]/50">
                        <td className="p-3 font-mono">{p.process_id}</td>
                        <td className="p-3 font-medium">{p.process_type}</td>
                        <td className="p-3">
                          <span
                            className={`px-2 py-0.5 rounded text-xs ${
                              p.health_status === 'Healthy'
                                ? 'bg-[#a6e3a1]/20 text-[#a6e3a1]'
                                : p.health_status === 'Quarantined'
                                ? 'bg-[#f38ba8]/20 text-[#f38ba8]'
                                : 'bg-[#f9e2af]/20 text-[#f9e2af]'
                            }`}
                          >
                            {p.health_status}
                          </span>
                        </td>
                        <td className="p-3">{p.restart_count}</td>
                        <td className="p-3 text-xs">{new Date(p.last_heartbeat).toLocaleTimeString()}</td>
                        <td className="p-3">
                          <button
                            onClick={() => handleKillProcess(p.id)}
                            className="p-1 hover:bg-[#f38ba8]/20 text-[#f38ba8] rounded"
                            title="Kill Process Tree"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Tab 4: Recovery */}
      {activeTab === 'recovery' && (
        <div className="mt-6 space-y-6">
          <div className="bg-[#1e1e2e] p-6 rounded-lg border border-[#313244]">
            <h3 className="text-md font-semibold flex items-center gap-2">
              <RotateCcw className="w-5 h-5 text-[#89b4fa]" />
              Crash Recovery & Side-Effect Reconciliation
            </h3>
            <p className="text-sm text-[#a6adc8] mt-1">
              Pure or idempotent tasks are automatically resumed on startup. Tasks with external side-effects (e.g. Email dispatch) interrupted by unexpected shutdown are held in <strong>OutcomeUnknown</strong> to prevent accidental double-execution until reconciled.
            </p>

            <div className="mt-6 p-4 bg-[#11111b] rounded border border-[#313244]">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="font-medium text-sm">Startup Recovery Protocol</h4>
                  <p className="text-xs text-[#a6adc8] mt-0.5">
                    1. Re-open WAL SQLite database & verify schema integrity.<br />
                    2. Scan unfinished task attempts and invalidate stale leases.<br />
                    3. Rehydrate ready DAG nodes; flag uncommitted mutations for operator confirmation.
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <CheckCircle2 className="w-5 h-5 text-[#a6e3a1]" />
                  <span className="text-xs text-[#a6e3a1]">Protocol Ready</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
