import React, { useState, useEffect } from 'react';
import { Conversation, ApprovalRequest, DashboardStats } from './types';
import { invokeCommand } from './lib/tauriBridge';
import { ThemeProvider, useTheme } from './context/ThemeContext';
import { WindowHeader } from './components/WindowHeader';
import { ConversationSidebar } from './components/ConversationSidebar';
import { ChatView } from './components/ChatView';
import { SettingsDialog } from './components/SettingsDialog';

const MainApp: React.FC = () => {
  const { t } = useTheme();

  // Sidebar toggle state (persisted)
  const [sidebarOpen, setSidebarOpen] = useState<boolean>(() => {
    const saved = localStorage.getItem('app_sidebar_open');
    return saved !== null ? saved === 'true' : true;
  });

  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [activeConvId, setActiveConvId] = useState<string | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [pendingApprovals, setPendingApprovals] = useState<ApprovalRequest[]>([]);
  const [isEmergencyStop, setIsEmergencyStop] = useState<boolean>(false);

  // Toggle sidebar and persist
  const handleToggleSidebar = () => {
    setSidebarOpen((prev) => {
      const next = !prev;
      localStorage.setItem('app_sidebar_open', String(next));
      return next;
    });
  };

  // Initial load
  useEffect(() => {
    loadConversations();
    refreshSystemData();

    // Periodic check for approvals and stats (every 3 seconds)
    const interval = setInterval(refreshSystemData, 3000);
    return () => clearInterval(interval);
  }, []);

  const loadConversations = async () => {
    try {
      const list = await invokeCommand<Conversation[]>('list_conversations');
      setConversations(list);
      if (list.length > 0 && !activeConvId) {
        setActiveConvId(list[0].id);
      }
    } catch (e) {
      console.error('Failed to load conversations', e);
    }
  };

  const refreshSystemData = async () => {
    try {
      const s = await invokeCommand<DashboardStats>('get_dashboard_stats');
      setStats(s);
      setIsEmergencyStop(s.is_emergency_stopped);

      const apps = await invokeCommand<ApprovalRequest[]>('list_pending_approvals');
      setPendingApprovals(apps);
    } catch (e) {
      console.error('Failed to refresh system stats', e);
    }
  };

  const handleNewChat = async () => {
    const title = `${t('Hội thoại', 'Chat')} ${new Date().toLocaleTimeString([], {
      hour: '2-digit',
      minute: '2-digit',
    })}`;
    try {
      const created = await invokeCommand<Conversation>('create_conversation', { title });
      setConversations((prev) => [created, ...prev]);
      setActiveConvId(created.id);
    } catch (e) {
      console.error('Failed to create new conversation', e);
    }
  };

  const handleNewConversationNeeded = async (firstPrompt: string): Promise<string> => {
    const title = firstPrompt.trim().slice(0, 30);
    try {
      const created = await invokeCommand<Conversation>('create_conversation', { title });
      setConversations((prev) => [created, ...prev]);
      setActiveConvId(created.id);
      return created.id;
    } catch (e) {
      console.error('Failed to create conversation for prompt', e);
      return '';
    }
  };

  const handleRenameConversation = (id: string, newTitle: string) => {
    setConversations((prev) =>
      prev.map((c) => (c.id === id ? { ...c, title: newTitle } : c))
    );
  };

  const handleDeleteConversation = (id: string) => {
    setConversations((prev) => {
      const updated = prev.filter((c) => c.id !== id);
      if (activeConvId === id) {
        setActiveConvId(updated.length > 0 ? updated[0].id : null);
      }
      return updated;
    });
  };

  const handleResolveApproval = async (id: string, approved: boolean) => {
    try {
      await invokeCommand('resolve_approval', { id, approved });
      setPendingApprovals((prev) => prev.filter((a) => a.id !== id));
      refreshSystemData();
    } catch (e) {
      console.error('Failed to resolve approval', e);
    }
  };

  const handleResetEmergencyStop = async () => {
    try {
      await invokeCommand('reset_emergency_stop');
      setIsEmergencyStop(false);
      refreshSystemData();
    } catch (e) {
      console.error('Failed to reset emergency stop', e);
    }
  };

  const activeConversation =
    conversations.find((c) => c.id === activeConvId) || null;

  return (
    <div className="flex flex-col h-screen w-screen bg-app text-primary overflow-hidden font-sans">
      {/* 1. Ultra Minimal Window Header */}
      <WindowHeader
        sidebarOpen={sidebarOpen}
        onToggleSidebar={handleToggleSidebar}
        onNewChat={handleNewChat}
        onOpenSettings={() => setSettingsOpen(true)}
        activeConversationTitle={activeConversation?.title}
        isEmergencyStop={isEmergencyStop}
        onResetEmergencyStop={handleResetEmergencyStop}
      />

      {/* 2. Main Workspace Layout */}
      <div className="flex-1 flex overflow-hidden">
        {/* Collapsible Conversation History Sidebar */}
        <ConversationSidebar
          isOpen={sidebarOpen}
          conversations={conversations}
          activeConvId={activeConvId}
          onSelectConv={(id) => setActiveConvId(id)}
          onNewChat={handleNewChat}
          onDeleteConv={handleDeleteConversation}
          onRenameConv={handleRenameConversation}
          onOpenSettings={() => setSettingsOpen(true)}
          workspacePath={stats?.workspace_path}
        />

        {/* Core Chat-First Experience */}
        <main className="flex-1 flex flex-col h-full overflow-hidden">
          <ChatView
            activeConversation={activeConversation}
            onNewConversationNeeded={handleNewConversationNeeded}
            modelName={stats?.model_name || 'Universal AI'}
            pendingApprovals={pendingApprovals}
            onResolveApproval={handleResolveApproval}
          />
        </main>
      </div>

      {/* 3. Minimal Settings Dialog */}
      <SettingsDialog
        isOpen={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        onConfigUpdated={refreshSystemData}
      />
    </div>
  );
};

export const App: React.FC = () => {
  return (
    <ThemeProvider>
      <MainApp />
    </ThemeProvider>
  );
};
