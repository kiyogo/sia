import React, { useState, useEffect } from 'react';
import { ApprovalRequest, DashboardStats, ViewMode } from './types';
import { invokeCommand } from './lib/tauriBridge';
import { Sidebar } from './components/Sidebar';
import { EmergencyStopBanner } from './components/EmergencyStopBanner';
import { Dashboard } from './components/Dashboard';
import { Chat } from './components/Chat';
import { Tasks } from './components/Tasks';
import { Workspace } from './components/Workspace';
import { CodingWorkspace } from './components/CodingWorkspace';
import { ComputerControl } from './components/ComputerControl';
import { BrowserWorkspace } from './components/BrowserWorkspace';
import { WorkflowOrchestrator } from './components/WorkflowOrchestrator';
import { ResearchWorkspace } from './components/ResearchWorkspace';
import { BusinessDashboard } from './components/BusinessDashboard';
import { TerminalView } from './components/TerminalView';
import { Settings } from './components/Settings';
import { ApprovalDialog } from './components/ApprovalDialog';

export const App: React.FC = () => {
  const [currentView, setCurrentView] = useState<ViewMode>('dashboard');
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [pendingApprovals, setPendingApprovals] = useState<ApprovalRequest[]>([]);
  const [isEmergencyStop, setIsEmergencyStop] = useState<boolean>(false);
  const [showApprovalModal, setShowApprovalModal] = useState<boolean>(false);

  useEffect(() => {
    refreshData();
    const interval = setInterval(refreshData, 3000);
    return () => clearInterval(interval);
  }, []);

  const refreshData = async () => {
    try {
      const s = await invokeCommand<DashboardStats>('get_dashboard_stats');
      setStats(s);
      const apps = await invokeCommand<ApprovalRequest[]>('list_pending_approvals');
      setPendingApprovals(apps);
    } catch (e) {
      console.error('Failed to refresh app stats', e);
    }
  };

  const handleTriggerStop = async () => {
    try {
      await invokeCommand('trigger_emergency_stop');
      setIsEmergencyStop(true);
      refreshData();
    } catch (e) {
      console.error('Failed to trigger emergency stop', e);
    }
  };

  const handleResetStop = async () => {
    try {
      await invokeCommand('reset_emergency_stop');
      setIsEmergencyStop(false);
      refreshData();
    } catch (e) {
      console.error('Failed to reset emergency stop', e);
    }
  };

  return (
    <div className="flex h-screen w-screen bg-slate-950 text-slate-100 overflow-hidden font-sans">
      {/* Sidebar Navigation */}
      <Sidebar
        currentView={currentView}
        onSelectView={setCurrentView}
        runtimeActive={stats?.runtime_active ?? true}
        providerConnected={stats?.provider_connected ?? false}
        mcpConnected={stats?.windows_mcp_connected ?? false}
        pendingApprovalsCount={pendingApprovals.length}
      />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col h-screen overflow-hidden">
        {/* Top Emergency Stop & Safety Banner */}
        <EmergencyStopBanner
          isEmergencyStop={isEmergencyStop}
          pendingApprovalsCount={pendingApprovals.length}
          onTriggerStop={handleTriggerStop}
          onResetStop={handleResetStop}
          onOpenApprovals={() => setShowApprovalModal(true)}
        />

        {/* View Routing */}
        <main className="flex-1 flex overflow-hidden">
          {currentView === 'dashboard' && (
            <Dashboard stats={stats} onNavigate={setCurrentView} onRefresh={refreshData} />
          )}
          {currentView === 'chat' && <Chat onTaskStarted={() => refreshData()} />}
          {currentView === 'tasks' && <Tasks />}
          {currentView === 'coding' && <CodingWorkspace />}
          {currentView === 'computer' && (
            <ComputerControl workspacePath={stats?.workspace_path || 'C:\\workspace'} />
          )}
          {currentView === 'browser' && (
            <BrowserWorkspace workspacePath={stats?.workspace_path || 'C:\\workspace'} />
          )}
          {currentView === 'workflows' && <WorkflowOrchestrator />}
          {currentView === 'research' && <ResearchWorkspace />}
          {currentView === 'business' && <BusinessDashboard />}
          {currentView === 'workspace' && <Workspace />}
          {currentView === 'terminal' && <TerminalView />}
          {currentView === 'settings' && <Settings />}
        </main>
      </div>

      {/* Approval Dialog Modal */}
      {showApprovalModal && (
        <ApprovalDialog
          approvals={pendingApprovals}
          onResolved={() => {
            refreshData();
            if (pendingApprovals.length <= 1) setShowApprovalModal(false);
          }}
          onClose={() => setShowApprovalModal(false)}
        />
      )}
    </div>
  );
};
