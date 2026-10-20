import React from 'react';
import { ViewMode } from '../types';
import {
  LayoutDashboard,
  MessageSquare,
  ListTodo,
  FolderTree,
  Terminal,
  Settings,
  Activity,
  Cpu,
} from 'lucide-react';

interface SidebarProps {
  currentView: ViewMode;
  onSelectView: (view: ViewMode) => void;
  runtimeActive: boolean;
  providerConnected: boolean;
  mcpConnected: boolean;
  pendingApprovalsCount: number;
}

export const Sidebar: React.FC<SidebarProps> = ({
  currentView,
  onSelectView,
  runtimeActive,
  providerConnected,
  mcpConnected,
  pendingApprovalsCount,
}) => {
  const navItems: { id: ViewMode; label: string; icon: React.ReactNode; badge?: number }[] = [
    { id: 'dashboard', label: 'Dashboard', icon: <LayoutDashboard className="w-5 h-5" /> },
    { id: 'chat', label: 'AI Chat', icon: <MessageSquare className="w-5 h-5" /> },
    { id: 'tasks', label: 'Tasks', icon: <ListTodo className="w-5 h-5" /> },
    { id: 'workspace', label: 'Workspace', icon: <FolderTree className="w-5 h-5" /> },
    { id: 'terminal', label: 'Terminal', icon: <Terminal className="w-5 h-5" /> },
    {
      id: 'settings',
      label: 'Settings',
      icon: <Settings className="w-5 h-5" />,
      badge: pendingApprovalsCount > 0 ? pendingApprovalsCount : undefined,
    },
  ];

  return (
    <aside className="w-64 bg-slate-900/90 backdrop-blur-md border-r border-slate-800 flex flex-col justify-between p-4 h-screen select-none">
      <div>
        {/* App Title */}
        <div className="flex items-center gap-3 px-3 py-4 mb-4 border-b border-slate-800">
          <div className="w-9 h-9 rounded-xl bg-indigo-600/30 border border-indigo-500/50 flex items-center justify-center text-indigo-400 font-bold shadow-lg shadow-indigo-500/10">
            <Cpu className="w-5 h-5" />
          </div>
          <div>
            <h1 className="font-semibold text-sm tracking-wide text-slate-100">Universal AI</h1>
            <p className="text-xs text-slate-400">Windows Workspace</p>
          </div>
        </div>

        {/* Navigation Items */}
        <nav className="space-y-1">
          {navItems.map((item) => {
            const isActive = currentView === item.id;
            return (
              <button
                key={item.id}
                onClick={() => onSelectView(item.id)}
                className={`w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-150 ${
                  isActive
                    ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 shadow-sm'
                    : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'
                }`}
              >
                <div className="flex items-center gap-3">
                  {item.icon}
                  <span>{item.label}</span>
                </div>
                {item.badge !== undefined && (
                  <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-amber-500/20 text-amber-300 border border-amber-500/40">
                    {item.badge}
                  </span>
                )}
              </button>
            );
          })}
        </nav>
      </div>

      {/* System Status Indicators */}
      <div className="bg-slate-950/60 border border-slate-800/80 rounded-xl p-3 space-y-2 text-xs">
        <div className="text-slate-400 font-medium mb-1 flex items-center gap-1.5">
          <Activity className="w-3.5 h-3.5 text-slate-400" />
          System Health
        </div>

        <div className="flex items-center justify-between text-slate-300">
          <span>Rust Runtime</span>
          <span className="flex items-center gap-1.5">
            <span
              className={`w-2 h-2 rounded-full ${
                runtimeActive ? 'bg-emerald-400 ring-2 ring-emerald-500/20' : 'bg-rose-500'
              }`}
            />
            {runtimeActive ? 'Active' : 'Offline'}
          </span>
        </div>

        <div className="flex items-center justify-between text-slate-300">
          <span>AI Provider</span>
          <span className="flex items-center gap-1.5">
            <span
              className={`w-2 h-2 rounded-full ${
                providerConnected ? 'bg-emerald-400 ring-2 ring-emerald-500/20' : 'bg-amber-400'
              }`}
            />
            {providerConnected ? 'Ready' : 'Config'}
          </span>
        </div>

        <div className="flex items-center justify-between text-slate-300">
          <span>Windows-MCP</span>
          <span className="flex items-center gap-1.5">
            <span
              className={`w-2 h-2 rounded-full ${
                mcpConnected ? 'bg-emerald-400 ring-2 ring-emerald-500/20' : 'bg-slate-600'
              }`}
            />
            {mcpConnected ? 'Connected' : 'Standby'}
          </span>
        </div>
      </div>
    </aside>
  );
};
