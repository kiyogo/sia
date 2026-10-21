import React, { useState, useMemo } from 'react';
import {
  MessageSquare,
  Plus,
  Search,
  Trash2,
  Edit2,
  Check,
  X,
  Settings,
  FolderGit2,
  ChevronDown,
} from 'lucide-react';
import { Conversation } from '../types';
import { useTheme } from '../context/ThemeContext';

interface ConversationSidebarProps {
  isOpen: boolean;
  conversations: Conversation[];
  activeConvId: string | null;
  onSelectConv: (id: string) => void;
  onNewChat: () => void;
  onDeleteConv?: (id: string) => void;
  onRenameConv?: (id: string, newTitle: string) => void;
  onOpenSettings: () => void;
  workspacePath?: string;
}

export const ConversationSidebar: React.FC<ConversationSidebarProps> = ({
  isOpen,
  conversations,
  activeConvId,
  onSelectConv,
  onNewChat,
  onDeleteConv,
  onRenameConv,
  onOpenSettings,
  workspacePath,
}) => {
  const { t } = useTheme();
  const [searchQuery, setSearchQuery] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState('');
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);

  // Filter conversations
  const filteredConversations = useMemo(() => {
    if (!searchQuery.trim()) return conversations;
    const query = searchQuery.toLowerCase();
    return conversations.filter((c) => c.title.toLowerCase().includes(query));
  }, [conversations, searchQuery]);

  // Group conversations by date: Today, Yesterday, Previous 7 Days, Older
  const groupedConversations = useMemo(() => {
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
    const yesterday = today - 86400000;
    const sevenDaysAgo = today - 86400000 * 7;

    const groups: {
      today: Conversation[];
      yesterday: Conversation[];
      previous7Days: Conversation[];
      older: Conversation[];
    } = {
      today: [],
      yesterday: [],
      previous7Days: [],
      older: [],
    };

    filteredConversations.forEach((conv) => {
      const convDate = new Date(conv.updated_at || conv.created_at).getTime();
      if (convDate >= today) {
        groups.today.push(conv);
      } else if (convDate >= yesterday) {
        groups.yesterday.push(conv);
      } else if (convDate >= sevenDaysAgo) {
        groups.previous7Days.push(conv);
      } else {
        groups.older.push(conv);
      }
    });

    return groups;
  }, [filteredConversations]);

  const handleStartRename = (conv: Conversation, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingId(conv.id);
    setEditingTitle(conv.title);
  };

  const handleSaveRename = (id: string, e: React.MouseEvent | React.FormEvent) => {
    e.stopPropagation();
    if (editingTitle.trim() && onRenameConv) {
      onRenameConv(id, editingTitle.trim());
    }
    setEditingId(null);
  };

  const handleCancelRename = (e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingId(null);
  };

  const handleDelete = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirmDeleteId === id) {
      if (onDeleteConv) onDeleteConv(id);
      setConfirmDeleteId(null);
    } else {
      setConfirmDeleteId(id);
    }
  };

  return (
    <aside
      className={`h-full bg-surface border-r border-border flex flex-col flex-shrink-0 transition-all duration-200 select-none ${
        isOpen ? 'w-64' : 'w-0'
      } overflow-hidden`}
    >
      {/* Top action: New chat */}
      <div className="p-3 border-b border-border-subtle flex flex-col gap-2">
        <button
          onClick={onNewChat}
          className="w-full flex items-center justify-center gap-2 py-2 px-3 text-sm font-medium rounded-lg bg-primary text-surface hover:opacity-90 transition-opacity shadow-sm"
        >
          <Plus className="w-4 h-4" />
          <span>{t('Cuộc trò chuyện mới', 'New Chat')}</span>
        </button>

        {/* Search Input */}
        <div className="relative">
          <Search className="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-muted" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={t('Tìm kiếm...', 'Search chats...')}
            className="w-full pl-8 pr-3 py-1.5 text-xs bg-muted/60 text-primary placeholder-muted rounded-md border border-transparent focus:border-border focus:outline-none transition-colors"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery('')}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-muted hover:text-primary"
            >
              <X className="w-3 h-3" />
            </button>
          )}
        </div>
      </div>

      {/* Conversation List */}
      <div className="flex-1 overflow-y-auto px-2 py-2 space-y-4">
        {filteredConversations.length === 0 ? (
          <div className="px-3 py-8 text-center text-xs text-muted">
            {searchQuery
              ? t('Không tìm thấy cuộc trò chuyện', 'No conversations found')
              : t('Chưa có lịch sử trò chuyện', 'No conversation history')}
          </div>
        ) : (
          <>
            {renderGroup(
              t('Hôm nay', 'Today'),
              groupedConversations.today,
              activeConvId,
              editingId,
              editingTitle,
              confirmDeleteId,
              onSelectConv,
              handleStartRename,
              handleSaveRename,
              handleCancelRename,
              handleDelete,
              setEditingTitle,
              setConfirmDeleteId
            )}
            {renderGroup(
              t('Hôm qua', 'Yesterday'),
              groupedConversations.yesterday,
              activeConvId,
              editingId,
              editingTitle,
              confirmDeleteId,
              onSelectConv,
              handleStartRename,
              handleSaveRename,
              handleCancelRename,
              handleDelete,
              setEditingTitle,
              setConfirmDeleteId
            )}
            {renderGroup(
              t('7 ngày trước', 'Previous 7 Days'),
              groupedConversations.previous7Days,
              activeConvId,
              editingId,
              editingTitle,
              confirmDeleteId,
              onSelectConv,
              handleStartRename,
              handleSaveRename,
              handleCancelRename,
              handleDelete,
              setEditingTitle,
              setConfirmDeleteId
            )}
            {renderGroup(
              t('Cũ hơn', 'Older'),
              groupedConversations.older,
              activeConvId,
              editingId,
              editingTitle,
              confirmDeleteId,
              onSelectConv,
              handleStartRename,
              handleSaveRename,
              handleCancelRename,
              handleDelete,
              setEditingTitle,
              setConfirmDeleteId
            )}
          </>
        )}
      </div>

      {/* Footer */}
      <div className="p-2 border-t border-border flex items-center justify-between text-xs text-secondary bg-surface">
        <div
          className="flex items-center gap-1.5 truncate max-w-[170px]"
          title={workspacePath || 'Workspace'}
        >
          <FolderGit2 className="w-3.5 h-3.5 text-muted flex-shrink-0" />
          <span className="truncate font-mono text-[11px]">
            {workspacePath ? workspacePath.split(/[/\\]/).filter(Boolean).pop() : 'Workspace'}
          </span>
        </div>

        <button
          onClick={onOpenSettings}
          title={t('Cài đặt', 'Settings')}
          className="p-1.5 rounded-md hover:bg-muted text-secondary hover:text-primary transition-colors"
        >
          <Settings className="w-4 h-4" />
        </button>
      </div>
    </aside>
  );
};

// Helper to render date section
function renderGroup(
  title: string,
  items: Conversation[],
  activeConvId: string | null,
  editingId: string | null,
  editingTitle: string,
  confirmDeleteId: string | null,
  onSelectConv: (id: string) => void,
  handleStartRename: (conv: Conversation, e: React.MouseEvent) => void,
  handleSaveRename: (id: string, e: React.MouseEvent | React.FormEvent) => void,
  handleCancelRename: (e: React.MouseEvent) => void,
  handleDelete: (id: string, e: React.MouseEvent) => void,
  setEditingTitle: (val: string) => void,
  setConfirmDeleteId: (id: string | null) => void
) {
  if (items.length === 0) return null;

  return (
    <div className="space-y-1">
      <div className="px-2 text-[11px] font-semibold tracking-wider uppercase text-muted">
        {title}
      </div>
      <div className="space-y-0.5">
        {items.map((conv) => {
          const isActive = conv.id === activeConvId;
          const isEditing = conv.id === editingId;
          const isConfirmingDelete = conv.id === confirmDeleteId;

          return (
            <div
              key={conv.id}
              onClick={() => onSelectConv(conv.id)}
              className={`group relative flex items-center justify-between px-2.5 py-2 rounded-lg text-xs cursor-pointer transition-colors ${
                isActive
                  ? 'bg-muted text-primary font-medium'
                  : 'text-secondary hover:text-primary hover:bg-muted/50'
              }`}
            >
              {isEditing ? (
                <form
                  onSubmit={(e) => handleSaveRename(conv.id, e)}
                  className="flex items-center gap-1 w-full"
                  onClick={(e) => e.stopPropagation()}
                >
                  <input
                    type="text"
                    value={editingTitle}
                    onChange={(e) => setEditingTitle(e.target.value)}
                    autoFocus
                    className="flex-1 px-1.5 py-0.5 text-xs bg-surface border border-border rounded text-primary focus:outline-none"
                  />
                  <button
                    type="submit"
                    className="p-1 hover:text-emerald-500 text-muted"
                  >
                    <Check className="w-3 h-3" />
                  </button>
                  <button
                    type="button"
                    onClick={handleCancelRename}
                    className="p-1 hover:text-red-500 text-muted"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </form>
              ) : (
                <>
                  <div className="flex items-center gap-2 truncate pr-2">
                    <MessageSquare className="w-3.5 h-3.5 flex-shrink-0 text-muted" />
                    <span className="truncate">{conv.title || 'Untitled'}</span>
                  </div>

                  {/* Actions on hover */}
                  <div
                    className={`flex items-center gap-0.5 transition-opacity ${
                      isActive || isConfirmingDelete
                        ? 'opacity-100'
                        : 'opacity-0 group-hover:opacity-100'
                    }`}
                  >
                    {isConfirmingDelete ? (
                      <div
                        className="flex items-center gap-1 bg-red-500/10 text-red-500 px-1 py-0.5 rounded text-[10px]"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <button
                          onClick={(e) => handleDelete(conv.id, e)}
                          title="Xác nhận xóa"
                          className="hover:underline font-bold"
                        >
                          Xóa
                        </button>
                        <button
                          onClick={() => setConfirmDeleteId(null)}
                          className="hover:opacity-75"
                        >
                          Hủy
                        </button>
                      </div>
                    ) : (
                      <>
                        <button
                          onClick={(e) => handleStartRename(conv, e)}
                          className="p-1 text-muted hover:text-primary rounded transition-colors"
                          title="Đổi tên"
                        >
                          <Edit2 className="w-3 h-3" />
                        </button>
                        <button
                          onClick={(e) => handleDelete(conv.id, e)}
                          className="p-1 text-muted hover:text-red-500 rounded transition-colors"
                          title="Xóa"
                        >
                          <Trash2 className="w-3 h-3" />
                        </button>
                      </>
                    )}
                  </div>
                </>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
