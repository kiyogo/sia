import React, { useState, useEffect, useRef, useCallback } from 'react';
import { ArrowDown, Sparkles, Code2, Terminal, Globe, Cpu, Loader2 } from 'lucide-react';
import { Conversation, Message, ApprovalRequest, AgentEvent } from '../types';
import { invokeCommand, listenToEvent } from '../lib/tauriBridge';
import { MessageItem } from './MessageItem';
import { ChatComposer } from './ChatComposer';
import { ToolActivityItem } from './ContextualActivity';
import { InlineApprovalCard } from './InlineApprovalCard';
import { useTheme } from '../context/ThemeContext';

interface ChatViewProps {
  activeConversation: Conversation | null;
  onNewConversationNeeded?: (firstPrompt: string) => Promise<string>;
  modelName?: string;
  pendingApprovals: ApprovalRequest[];
  onResolveApproval: (id: string, approved: boolean) => Promise<void>;
}

export const ChatView: React.FC<ChatViewProps> = ({
  activeConversation,
  onNewConversationNeeded,
  modelName = 'Universal AI',
  pendingApprovals,
  onResolveApproval,
}) => {
  const { t } = useTheme();
  const [messages, setMessages] = useState<Message[]>([]);
  const [isGenerating, setIsGenerating] = useState(false);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const [liveActivities, setLiveActivities] = useState<ToolActivityItem[]>([]);
  const [liveThinking, setLiveThinking] = useState<string | null>(null);
  const [showScrollBottom, setShowScrollBottom] = useState(false);

  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const isAutoScrollEnabled = useRef(true);

  // Load message history when active conversation changes
  useEffect(() => {
    if (activeConversation) {
      loadMessages(activeConversation.id);
    } else {
      setMessages([]);
      setLiveActivities([]);
      setLiveThinking(null);
    }
  }, [activeConversation?.id]);

  const loadMessages = async (convId: string) => {
    try {
      const list = await invokeCommand<Message[]>('get_messages', { conversationId: convId });
      setMessages(list);
    } catch (e) {
      console.error('Failed to load messages', e);
    }
  };

  // Listen to runtime host agent-event stream
  useEffect(() => {
    let unlisten: (() => void) | null = null;

    listenToEvent<{ task_id: string; event: AgentEvent }>('agent-event', (data) => {
      const { task_id, event } = data;

      if ('ModelThinking' in event) {
        setLiveThinking(event.ModelThinking.preview);
      } else if ('ToolCallStarted' in event) {
        const { tool_name, call_id, arguments: args } = event.ToolCallStarted;
        setLiveActivities((prev) => [
          ...prev,
          {
            id: call_id,
            toolName: tool_name,
            arguments: args,
            status: 'running',
          },
        ]);
      } else if ('ToolCallFinished' in event) {
        const { call_id, duration_ms, exit_code, is_error } = event.ToolCallFinished;
        setLiveActivities((prev) =>
          prev.map((act) =>
            act.id === call_id
              ? {
                  ...act,
                  status: is_error ? 'failed' : 'completed',
                  durationMs: duration_ms,
                  exitCode: exit_code,
                }
              : act
          )
        );
      } else if ('FinalAnswer' in event) {
        setLiveThinking(null);
        setIsGenerating(false);
        if (activeConversation) {
          loadMessages(activeConversation.id);
        }
      } else if ('Error' in event || 'TaskStatusChanged' in event) {
        if ('Error' in event) {
          setLiveThinking(null);
          setIsGenerating(false);
        }
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [activeConversation?.id]);

  // Scroll detection to disable auto-scroll when user scrolls up
  const handleScroll = useCallback(() => {
    const el = scrollContainerRef.current;
    if (!el) return;

    const distanceFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    if (distanceFromBottom > 120) {
      isAutoScrollEnabled.current = false;
      setShowScrollBottom(true);
    } else {
      isAutoScrollEnabled.current = true;
      setShowScrollBottom(false);
    }
  }, []);

  const scrollToBottom = (smooth = true) => {
    const el = scrollContainerRef.current;
    if (!el) return;
    el.scrollTo({
      top: el.scrollHeight,
      behavior: smooth ? 'smooth' : 'auto',
    });
    isAutoScrollEnabled.current = true;
    setShowScrollBottom(false);
  };

  useEffect(() => {
    if (isAutoScrollEnabled.current) {
      scrollToBottom(false);
    }
  }, [messages, liveActivities, liveThinking]);

  // Send message handler
  const handleSendMessage = async (prompt: string) => {
    if (!prompt.trim() || isGenerating) return;

    let convId = activeConversation?.id;

    if (!convId && onNewConversationNeeded) {
      convId = await onNewConversationNeeded(prompt);
    }

    if (!convId) return;

    // Optimistically append user message
    const optimisticUserMsg: Message = {
      id: String(Date.now()),
      conversation_id: convId,
      role: 'user',
      content: prompt,
      tool_calls: null,
      tool_call_id: null,
      token_count: Math.ceil(prompt.length / 4),
      created_at: new Date().toISOString(),
    };

    setMessages((prev) => [...prev, optimisticUserMsg]);
    setIsGenerating(true);
    setLiveActivities([]);
    setLiveThinking(t('Đang suy nghĩ...', 'Thinking...'));

    try {
      const taskId = await invokeCommand<string>('send_chat_message', {
        conversationId: convId,
        content: prompt,
      });
      setActiveTaskId(taskId);
    } catch (e) {
      console.error('Failed to send message', e);
      setIsGenerating(false);
      setLiveThinking(null);
    }
  };

  const handleStop = async () => {
    if (activeTaskId) {
      try {
        await invokeCommand('cancel_task', { taskId: activeTaskId });
      } catch (e) {
        console.error('Failed to cancel task', e);
      }
    }
    setIsGenerating(false);
    setLiveThinking(null);
  };

  return (
    <div className="flex-1 flex flex-col h-full bg-app overflow-hidden relative">
      {/* Messages Scroll Area */}
      <div
        ref={scrollContainerRef}
        onScroll={handleScroll}
        className="flex-1 overflow-y-auto overflow-x-hidden pt-4 pb-2"
      >
        {messages.length === 0 ? (
          // Clean Minimal Empty State
          <div className="h-full flex flex-col items-center justify-center px-4 py-16 text-center select-none max-w-lg mx-auto">
            <div className="w-12 h-12 rounded-2xl bg-surface border border-border flex items-center justify-center text-primary shadow-sm mb-4">
              <Sparkles className="w-6 h-6 text-accent" />
            </div>

            <h2 className="text-xl font-semibold text-primary mb-2 tracking-tight">
              {t('Tôi có thể giúp gì cho bạn?', 'How can I help you today?')}
            </h2>

            <p className="text-xs text-secondary leading-relaxed mb-6">
              {t(
                'Giao việc bằng ngôn ngữ tự nhiên: viết code, phân tích dữ liệu, thao tác web, hoặc tự động hóa quy trình.',
                'Ask anything in natural language: write code, inspect projects, automate desktop tasks, or run workflows.'
              )}
            </p>

            {/* Quick Suggestion Chips */}
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 w-full text-left">
              <button
                onClick={() =>
                  handleSendMessage(
                    t('Kiểm tra cấu trúc dự án và chạy build/test xem có lỗi gì không', 'Inspect project workspace and run tests')
                  )
                }
                className="flex items-center gap-2.5 p-3 rounded-xl bg-surface border border-border hover:border-accent/50 hover:bg-muted/40 transition-all text-xs group"
              >
                <Code2 className="w-4 h-4 text-accent group-hover:scale-105 transition-transform" />
                <span className="text-secondary group-hover:text-primary">
                  {t('Kiểm tra & build mã nguồn', 'Inspect & build code')}
                </span>
              </button>

              <button
                onClick={() =>
                  handleSendMessage(
                    t('Viết một script tự động hóa kiểm tra tài nguyên hệ thống', 'Write an automated script to inspect system resources')
                  )
                }
                className="flex items-center gap-2.5 p-3 rounded-xl bg-surface border border-border hover:border-accent/50 hover:bg-muted/40 transition-all text-xs group"
              >
                <Terminal className="w-4 h-4 text-amber-500 group-hover:scale-105 transition-transform" />
                <span className="text-secondary group-hover:text-primary">
                  {t('Thực thi lệnh terminal', 'Execute terminal tasks')}
                </span>
              </button>

              <button
                onClick={() =>
                  handleSendMessage(
                    t('Nghiên cứu tài liệu và tổng hợp các điểm quan trọng', 'Research documentation and synthesize key points')
                  )
                }
                className="flex items-center gap-2.5 p-3 rounded-xl bg-surface border border-border hover:border-accent/50 hover:bg-muted/40 transition-all text-xs group"
              >
                <Globe className="w-4 h-4 text-emerald-500 group-hover:scale-105 transition-transform" />
                <span className="text-secondary group-hover:text-primary">
                  {t('Nghiên cứu & tổng hợp', 'Research & summarize')}
                </span>
              </button>

              <button
                onClick={() =>
                  handleSendMessage(
                    t('Lập kế hoạch tối ưu hóa hiệu năng và kiến trúc', 'Plan performance and architecture optimizations')
                  )
                }
                className="flex items-center gap-2.5 p-3 rounded-xl bg-surface border border-border hover:border-accent/50 hover:bg-muted/40 transition-all text-xs group"
              >
                <Cpu className="w-4 h-4 text-purple-500 group-hover:scale-105 transition-transform" />
                <span className="text-secondary group-hover:text-primary">
                  {t('Lập kế hoạch tự động hóa', 'Plan autonomous tasks')}
                </span>
              </button>
            </div>
          </div>
        ) : (
          // Message stream
          <div className="space-y-1">
            {messages.map((msg, index) => {
              const isLast = index === messages.length - 1;
              return (
                <MessageItem
                  key={msg.id}
                  message={msg}
                  activities={isLast && msg.role === 'assistant' ? liveActivities : []}
                />
              );
            })}

            {/* In-chat live activities when assistant is working */}
            {isGenerating && liveActivities.length > 0 && messages[messages.length - 1]?.role === 'user' && (
              <div className="w-full max-w-3xl mx-auto px-4 py-2">
                <div className="space-y-1">
                  {liveActivities.map((act) => (
                    <ContextualActivity key={act.id} activity={act} />
                  ))}
                </div>
              </div>
            )}

            {/* Thinking preview indicator */}
            {isGenerating && liveThinking && (
              <div className="w-full max-w-3xl mx-auto px-4 py-2">
                <div className="flex items-center gap-2 text-xs text-muted font-sans select-none animate-pulse">
                  <Loader2 className="w-3.5 h-3.5 animate-spin text-accent" />
                  <span>{liveThinking}</span>
                </div>
              </div>
            )}

            {/* In-line Pending Approvals if any */}
            {pendingApprovals.length > 0 && (
              <div className="w-full max-w-3xl mx-auto px-4">
                {pendingApprovals.map((req) => (
                  <InlineApprovalCard
                    key={req.id}
                    request={req}
                    onResolve={onResolveApproval}
                  />
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      {/* Floating "Jump to latest" button */}
      {showScrollBottom && (
        <button
          onClick={() => scrollToBottom(true)}
          className="absolute bottom-24 right-6 p-2 rounded-full bg-surface border border-border text-secondary hover:text-primary shadow-md transition-all z-10 animate-fade-in"
          title={t('Cuộn xuống dưới cùng', 'Scroll to bottom')}
        >
          <ArrowDown className="w-4 h-4" />
        </button>
      )}

      {/* Bottom Composer */}
      <ChatComposer
        onSendMessage={handleSendMessage}
        onStopGeneration={handleStop}
        isGenerating={isGenerating}
        modelName={modelName}
      />
    </div>
  );
};
