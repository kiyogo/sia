import React from 'react';
import { DashboardStats, ViewMode } from '../types';
import {
  Activity,
  Cpu,
  Bot,
  Layers,
  CheckCircle,
  AlertTriangle,
  Clock,
  Shield,
  Folder,
  ArrowRight,
} from 'lucide-react';

interface DashboardProps {
  stats: DashboardStats | null;
  onNavigate: (view: ViewMode) => void;
  onRefresh: () => void;
}

export const Dashboard: React.FC<DashboardProps> = ({ stats, onNavigate, onRefresh }) => {
  if (!stats) {
    return (
      <div className="flex-1 flex items-center justify-center p-8 text-slate-400">
        <div className="flex items-center gap-3">
          <Activity className="w-5 h-5 animate-spin text-indigo-400" />
          <span>Loading workspace dashboard metrics...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto p-8 space-y-6">
      {/* Top Welcome & Refresh */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight text-slate-100">Workspace Dashboard</h2>
          <p className="text-sm text-slate-400 mt-1">
            Real-time runtime state, tool execution health, and agent activity metrics.
          </p>
        </div>
        <button
          onClick={onRefresh}
          className="px-3.5 py-1.5 text-xs font-medium rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 transition"
        >
          Refresh State
        </button>
      </div>

      {/* Primary Status Metric Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Runtime Card */}
        <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 space-y-2">
          <div className="flex items-center justify-between text-slate-400">
            <span className="text-xs font-medium">Rust Core Runtime</span>
            <Cpu className="w-4 h-4 text-indigo-400" />
          </div>
          <div className="text-xl font-bold text-slate-100 flex items-center gap-2">
            <span
              className={`w-2.5 h-2.5 rounded-full ${
                stats.runtime_active ? 'bg-emerald-400' : 'bg-rose-500'
              }`}
            />
            {stats.runtime_active ? 'Operational' : 'Halted'}
          </div>
          <p className="text-xs text-slate-400">Process running with async Tokio engine</p>
        </div>

        {/* AI Provider Card */}
        <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 space-y-2">
          <div className="flex items-center justify-between text-slate-400">
            <span className="text-xs font-medium">AI Model Provider</span>
            <Bot className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-xl font-bold text-slate-100 flex items-center gap-2">
            <span
              className={`w-2.5 h-2.5 rounded-full ${
                stats.provider_connected ? 'bg-emerald-400' : 'bg-amber-400'
              }`}
            />
            {stats.provider_connected ? stats.model_name || 'Connected' : 'Needs Config'}
          </div>
          <p className="text-xs text-slate-400 truncate">{stats.provider_name}</p>
        </div>

        {/* Windows-MCP Card */}
        <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 space-y-2">
          <div className="flex items-center justify-between text-slate-400">
            <span className="text-xs font-medium">Windows-MCP</span>
            <Layers className="w-4 h-4 text-blue-400" />
          </div>
          <div className="text-xl font-bold text-slate-100 flex items-center gap-2">
            <span
              className={`w-2.5 h-2.5 rounded-full ${
                stats.windows_mcp_connected
                  ? 'bg-emerald-400'
                  : stats.windows_mcp_prerequisites
                  ? 'bg-blue-400'
                  : 'bg-amber-400'
              }`}
            />
            {stats.windows_mcp_connected
              ? 'Connected'
              : stats.windows_mcp_prerequisites
              ? 'Ready to Attach'
              : 'Prereqs Missing'}
          </div>
          <p className="text-xs text-slate-400 truncate">{stats.windows_mcp_hint}</p>
        </div>

        {/* Security Policy Card */}
        <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 space-y-2">
          <div className="flex items-center justify-between text-slate-400">
            <span className="text-xs font-medium">Security Policy</span>
            <Shield className="w-4 h-4 text-amber-400" />
          </div>
          <div className="text-xl font-bold text-slate-100 flex items-center gap-2">
            <span className="w-2.5 h-2.5 rounded-full bg-emerald-400" />
            {stats.permission_mode}
          </div>
          <p className="text-xs text-slate-400">
            {stats.pending_approvals > 0
              ? `${stats.pending_approvals} actions pending approval`
              : 'Sandbox boundary enforced'}
          </p>
        </div>
      </div>

      {/* Task Execution Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="p-5 rounded-xl bg-slate-900/40 border border-slate-800 flex items-center gap-4">
          <div className="w-12 h-12 rounded-lg bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400">
            <Clock className="w-6 h-6" />
          </div>
          <div>
            <div className="text-2xl font-bold text-slate-100">{stats.tasks_running}</div>
            <div className="text-xs text-slate-400">Active Tasks In Progress</div>
          </div>
        </div>

        <div className="p-5 rounded-xl bg-slate-900/40 border border-slate-800 flex items-center gap-4">
          <div className="w-12 h-12 rounded-lg bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
            <CheckCircle className="w-6 h-6" />
          </div>
          <div>
            <div className="text-2xl font-bold text-slate-100">{stats.tasks_completed}</div>
            <div className="text-xs text-slate-400">Successfully Completed Tasks</div>
          </div>
        </div>

        <div className="p-5 rounded-xl bg-slate-900/40 border border-slate-800 flex items-center gap-4">
          <div className="w-12 h-12 rounded-lg bg-rose-500/10 border border-rose-500/20 flex items-center justify-center text-rose-400">
            <AlertTriangle className="w-6 h-6" />
          </div>
          <div>
            <div className="text-2xl font-bold text-slate-100">{stats.tasks_failed}</div>
            <div className="text-xs text-slate-400">Failed or Interrupted Tasks</div>
          </div>
        </div>
      </div>

      {/* Workspace Environment Card */}
      <div className="p-5 rounded-xl bg-slate-900/40 border border-slate-800 space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2 text-sm font-semibold text-slate-200">
            <Folder className="w-4 h-4 text-indigo-400" />
            <span>Active Project Workspace</span>
          </div>
          <button
            onClick={() => onNavigate('workspace')}
            className="text-xs text-indigo-400 hover:text-indigo-300 flex items-center gap-1 transition"
          >
            <span>Open Explorer</span>
            <ArrowRight className="w-3.5 h-3.5" />
          </button>
        </div>
        <div className="p-3 bg-slate-950/80 rounded-lg border border-slate-800/80 font-mono text-xs text-slate-300 break-all select-text">
          {stats.workspace_path}
        </div>
        <p className="text-xs text-slate-400">
          File operations are restricted to this root directory unless explicitly approved.
        </p>
      </div>

      {/* Quick Launch Action Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div
          onClick={() => onNavigate('chat')}
          className="p-5 rounded-xl bg-gradient-to-br from-indigo-950/40 to-slate-900 border border-indigo-500/20 hover:border-indigo-500/40 cursor-pointer transition group"
        >
          <div className="flex items-center justify-between mb-2">
            <h3 className="font-semibold text-slate-100 group-hover:text-indigo-300 transition">
              Start Autonomous Task in Chat
            </h3>
            <ArrowRight className="w-4 h-4 text-slate-400 group-hover:text-indigo-300 group-hover:translate-x-1 transition" />
          </div>
          <p className="text-xs text-slate-400">
            Give a goal in natural language. AI will plan, execute shell commands, edit files, and check results.
          </p>
        </div>

        <div
          onClick={() => onNavigate('terminal')}
          className="p-5 rounded-xl bg-gradient-to-br from-slate-900 to-slate-950 border border-slate-800 hover:border-slate-700 cursor-pointer transition group"
        >
          <div className="flex items-center justify-between mb-2">
            <h3 className="font-semibold text-slate-100 group-hover:text-slate-300 transition">
              Open Native Terminal
            </h3>
            <ArrowRight className="w-4 h-4 text-slate-400 group-hover:text-slate-300 group-hover:translate-x-1 transition" />
          </div>
          <p className="text-xs text-slate-400">
            Execute real CMD or PowerShell commands with live streaming, exit codes, and process timeouts.
          </p>
        </div>
      </div>
    </div>
  );
};
