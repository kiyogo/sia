import React from 'react';
import { PanelLeft, Plus, Settings2, ShieldAlert } from 'lucide-react';
import { useTheme } from '../context/ThemeContext';

interface WindowHeaderProps {
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
  onNewChat: () => void;
  onOpenSettings: () => void;
  activeConversationTitle?: string;
  isEmergencyStop?: boolean;
  onResetEmergencyStop?: () => void;
}

export const WindowHeader: React.FC<WindowHeaderProps> = ({
  sidebarOpen,
  onToggleSidebar,
  onNewChat,
  onOpenSettings,
  activeConversationTitle,
  isEmergencyStop,
  onResetEmergencyStop,
}) => {
  const { t } = useTheme();

  return (
    <header className="h-11 border-b border-border bg-surface flex items-center justify-between px-3 select-none flex-shrink-0 z-20">
      {/* Left controls */}
      <div className="flex items-center gap-2 overflow-hidden">
        <button
          onClick={onToggleSidebar}
          title={sidebarOpen ? t('Đóng thanh bên', 'Close sidebar') : t('Mở thanh bên', 'Open sidebar')}
          className="p-1.5 rounded-md text-secondary hover:text-primary hover:bg-muted transition-colors"
        >
          <PanelLeft className="w-4 h-4" />
        </button>

        <div className="flex items-center gap-2 overflow-hidden text-sm">
          <span className="font-semibold text-primary tracking-tight whitespace-nowrap">
            SIA
          </span>
          {activeConversationTitle && (
            <>
              <span className="text-muted">/</span>
              <span className="text-secondary truncate max-w-[280px] sm:max-w-[420px]">
                {activeConversationTitle}
              </span>
            </>
          )}
        </div>
      </div>

      {/* Right controls */}
      <div className="flex items-center gap-1.5">
        {isEmergencyStop && (
          <button
            onClick={onResetEmergencyStop}
            className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-full bg-red-500/15 text-red-500 hover:bg-red-500/25 border border-red-500/30 transition-colors animate-pulse"
            title={t('Dừng khẩn cấp đang kích hoạt. Bấm để khôi phục.', 'Emergency stop active. Click to reset.')}
          >
            <ShieldAlert className="w-3.5 h-3.5" />
            <span>{t('Dừng khẩn cấp', 'Emergency Stop')}</span>
          </button>
        )}

        <button
          onClick={onNewChat}
          title={t('Cuộc trò chuyện mới', 'New Chat')}
          className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-md text-secondary hover:text-primary hover:bg-muted transition-colors border border-border"
        >
          <Plus className="w-3.5 h-3.5" />
          <span className="hidden sm:inline">{t('Đoạn chat mới', 'New Chat')}</span>
        </button>

        <button
          onClick={onOpenSettings}
          title={t('Cài đặt', 'Settings')}
          className="p-1.5 rounded-md text-secondary hover:text-primary hover:bg-muted transition-colors"
        >
          <Settings2 className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};
