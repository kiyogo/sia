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
    <header className="h-11 border-b border-zinc-200 dark:border-zinc-800/80 bg-white dark:bg-[#121215] flex items-center justify-between px-3 select-none flex-shrink-0 z-20 transition-colors">
      {/* Left controls */}
      <div className="flex items-center gap-2 overflow-hidden">
        <button
          onClick={onToggleSidebar}
          title={sidebarOpen ? t('Đóng thanh bên', 'Close sidebar') : t('Mở thanh bên', 'Open sidebar')}
          className="p-1.5 rounded-lg text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <PanelLeft className="w-4 h-4" />
        </button>

        <div className="flex items-center gap-2 overflow-hidden text-sm">
          <span className="font-semibold text-zinc-900 dark:text-zinc-100 tracking-tight whitespace-nowrap">
            SIA
          </span>
          {activeConversationTitle && (
            <>
              <span className="text-zinc-300 dark:text-zinc-700">/</span>
              <span className="text-zinc-600 dark:text-zinc-400 truncate max-w-[280px] sm:max-w-[420px]">
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
            className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-full bg-red-500/15 text-red-600 dark:text-red-400 hover:bg-red-500/25 border border-red-500/30 transition-colors animate-pulse"
            title={t('Dừng khẩn cấp đang kích hoạt. Bấm để khôi phục.', 'Emergency stop active. Click to reset.')}
          >
            <ShieldAlert className="w-3.5 h-3.5" />
            <span>{t('Dừng khẩn cấp', 'Emergency Stop')}</span>
          </button>
        )}

        <button
          onClick={onNewChat}
          title={t('Cuộc trò chuyện mới', 'New Chat')}
          className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg text-zinc-700 dark:text-zinc-300 hover:text-zinc-900 dark:hover:text-zinc-100 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors border border-zinc-200 dark:border-zinc-800"
        >
          <Plus className="w-3.5 h-3.5" />
          <span className="hidden sm:inline">{t('Đoạn chat mới', 'New Chat')}</span>
        </button>

        <button
          onClick={onOpenSettings}
          title={t('Cài đặt', 'Settings')}
          className="p-1.5 rounded-lg text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <Settings2 className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};
