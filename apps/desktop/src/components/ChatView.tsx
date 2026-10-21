import React, { useState, useEffect, useRef, useCallback } from 'react';
import {
  ArrowDown,
  Sparkles,
  Code2,
  Terminal,
  Globe,
  Cpu,
  Loader2,
  Brain,
  AlertCircle,
  Settings2,
} from 'lucide-react';
import { Conversation, Message, ApprovalRequest, AgentEvent } from '../types';
import { invokeCommand, listenToEvent } from '../lib/tauriBridge';
import { MessageItem } from './MessageItem';
import { ChatComposer } from './ChatComposer';
import { ContextualActivity, ToolActivityItem } from './ContextualActivity';
import { InlineApprovalCard } from './InlineApprovalCard';
import { useTheme } from '../context/ThemeContext';

interface ChatViewProps {
  activeConversation: Conversation | null;
  onNewConversationNeeded?: (firstPrompt: string) => Promise<string>;
  modelName?: string;
  pendingApprovals: ApprovalRequest[];
  onResolveApproval: (id: string, approved: boolean) => Promise<void>;
  onOpenSettings?: () => void;
}

export const ChatView: React.FC<ChatViewProps> = ({
  activeConversation,
  onNewConversationNeeded,
  modelName = 'Universal AI',
  pendingApprovals,
  onResolveApproval,
  onOpenSettings,
}) => {
  const { t } = useTheme();
  const [messages, setMessages] = useState<Message[]>([]);
  const [isGenerating, setIsGenerating] = useState(false);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const [liveActivities, setLiveActivities] = useState<ToolActivityItem[]>([]);
  const [liveThinking, setLiveThinking] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [showScrollBottom, setShowScrollBottom] = useState(false);
  const [thinkingElapsed, setThinkingElapsed] = useState(0);

  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const isAutoScrollEnabled = useRef(true);

  // Timer for thinking animation
  useEffect(() => {
    let timer: any = null;
    if (isGenerating) {
      setThinkingElapsed(0);
      timer = setInterval(() => {
        setThinkingElapsed((prev) => prev + 1);
      }, 1000);
    } else {
      setThinkingElapsed(0);
    }
    return () => {
      if (timer) clearInterval(timer);
    };
  }, [isGenerating]);

  // Load message history when active conversation changes
  useEffect(() => {
    if (activeConversation) {
      loadMessages(activeConversation.id);
    } else {
      setMessages([]);
      setLiveActivities([]);
      setLiveThinking(null);
      setErrorMessage(null);
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
      const { event } = data;

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
        setErrorMessage(null);
        const asstMsg: Message = {
          id: String(Date.now()),
          conversation_id: activeConversation?.id || '',
          role: 'assistant',
          content: event.FinalAnswer.content,
          tool_calls: null,
          tool_call_id: null,
          token_count: Math.ceil(event.FinalAnswer.content.length / 4),
          created_at: new Date().toISOString(),
        };
        setMessages((prev) => {
          if (prev.some((m) => m.content === asstMsg.content && m.role === 'assistant')) {
            return prev;
          }
          return [...prev, asstMsg];
        });
        if (activeConversation) {
          loadMessages(activeConversation.id);
        }
      } else if ('Error' in event) {
        setLiveThinking(null);
        setIsGenerating(false);
        setErrorMessage(event.Error.message);
        const errMsg: Message = {
          id: String(Date.now()),
          conversation_id: activeConversation?.id || '',
          role: 'assistant',
          content: `⚠️ **Lỗi phản hồi từ AI**: ${event.Error.message}\n\n*Vui lòng mở Cài đặt (Settings) để kiểm tra lại Endpoint và API Key.*`,
          tool_calls: null,
          tool_call_id: null,
          token_count: 20,
          created_at: new Date().toISOString(),
        };
        setMessages((prev) => [...prev, errMsg]);
      } else if ('TaskStatusChanged' in event) {
        if (event.TaskStatusChanged.status === 'Cancelled' || event.TaskStatusChanged.status === 'Failed') {
          setIsGenerating(false);
          setLiveThinking(null);
        }
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [activeConversation?.id]);

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
  }, [messages, liveActivities, liveThinking, errorMessage]);

  const handleSendMessage = async (prompt: string) => {
    if (!prompt.trim() || isGenerating) return;

    let convId = activeConversation?.id;

    if (!convId && onNewConversationNeeded) {
      convId = await onNewConversationNeeded(prompt);
    }

    if (!convId) return;

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
    setLiveThinking(t('Đang phân tích yêu cầu...', 'Analyzing request...'));
    setErrorMessage(null);

    try {
      const taskId = await invokeCommand<string>('send_chat_message', {
        conversationId: convId,
        content: prompt,
      });
      setActiveTaskId(taskId);
    } catch (e: any) {
      console.error('Failed to send message', e);
      setIsGenerating(false);
      setLiveThinking(null);
      setErrorMessage(typeof e === 'string' ? e : e?.message || 'Failed to start AI generation task');
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
    <div className="flex-1 flex flex-col h-full bg-zinc-50 dark:bg-[#09090b] text-zinc-900 dark:text-zinc-100 overflow-hidden relative transition-colors">
      {/* Messages Scroll Area */}
      <div
        ref={scrollContainerRef}
        onScroll={handleScroll}
        className="flex-1 overflow-y-auto overflow-x-hidden pt-4 pb-2"
      >
        {messages.length === 0 ? (
          /* Clean Minimal Empty State */
          <div className="h-full flex flex-col items-center justify-center px-4 py-16 text-center select-none max-w-lg mx-auto">
            <div className="w-12 h-12 rounded-2xl bg-white dark:bg-[#141417] border border-zinc-200 dark:border-zinc-800 flex items-center justify-center text-zinc-900 dark:text-zinc-100 shadow-sm mb-4">
              <Sparkles className="w-6 h-6 text-blue-500" />
            </div>

            <h2 className="text-xl font-semibold text-zinc-900 dark:text-zinc-100 mb-2 tracking-tight">
              {t('Tôi có thể giúp gì cho bạn?', 'How can I help you today?')}
            </h2>

            <p className="text-xs text-zinc-500 dark:text-zinc-400 leading-relaxed mb-6">
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
                className="flex items-center gap-2.5 p-3 rounded-xl bg-white dark:bg-[#141417] border border-zinc-200 dark:border-zinc-800 hover:border-blue-500/50 hover:bg-zinc-100/60 dark:hover:bg-zinc-800/50 transition-all text-xs group"
              >
                <Code2 className="w-4 h-4 text-blue-500 group-hover:scale-105 transition-transform" />
                <span className="text-zinc-600 dark:text-zinc-300 group-hover:text-zinc-900 dark:group-hover:text-zinc-100">
                  {t('Kiểm tra & build mã nguồn', 'Inspect & build code')}
                </span>
              </button>

              <button
                onClick={() =>
                  handleSendMessage(
                    t('Viết một script tự động hóa kiểm tra tài nguyên hệ thống', 'Write an automated script to inspect system resources')
                  )
                }
                className="flex items-center gap-2.5 p-3 rounded-xl bg-white dark:bg-[#141417] border border-zinc-200 dark:border-zinc-800 hover:border-blue-500/50 hover:bg-zinc-100/60 dark:hover:bg-zinc-800/50 transition-all text-xs group"
              >
                <Terminal className="w-4 h-4 text-amber-500 group-hover:scale-105 transition-transform" />
                <span className="text-zinc-600 dark:text-zinc-300 group-hover:text-zinc-900 dark:group-hover:text-zinc-100">
                  {t('Thực thi lệnh terminal', 'Execute terminal tasks')}
                </span>
              </button>

              <button
                onClick={() =>
                  handleSendMessage(
                    t('Nghiên cứu tài liệu và tổng hợp các điểm quan trọng', 'Research documentation and synthesize key points')
                  )
                }
                className="flex items-center gap-2.5 p-3 rounded-xl bg-white dark:bg-[#141417] border border-zinc-200 dark:border-zinc-800 hover:border-blue-500/50 hover:bg-zinc-100/60 dark:hover:bg-zinc-800/50 transition-all text-xs group"
              >
                <Globe className="w-4 h-4 text-emerald-500 group-hover:scale-105 transition-transform" />
                <span className="text-zinc-600 dark:text-zinc-300 group-hover:text-zinc-900 dark:group-hover:text-zinc-100">
                  {t('Nghiên cứu & tổng hợp', 'Research & summarize')}
                </span>
              </button>

              <button
                onClick={() =>
                  handleSendMessage(
                    t('Lập kế hoạch tối ưu hóa hiệu năng và kiến trúc', 'Plan performance and architecture optimizations')
                  )
                }
                className="flex items-center gap-2.5 p-3 rounded-xl bg-white dark:bg-[#141417] border border-zinc-200 dark:border-zinc-800 hover:border-blue-500/50 hover:bg-zinc-100/60 dark:hover:bg-zinc-800/50 transition-all text-xs group"
              >
                <Cpu className="w-4 h-4 text-purple-500 group-hover:scale-105 transition-transform" />
                <span className="text-zinc-600 dark:text-zinc-300 group-hover:text-zinc-900 dark:group-hover:text-zinc-100">
                  {t('Lập kế hoạch tự động hóa', 'Plan autonomous tasks')}
                </span>
              </button>
            </div>
          </div>
        ) : (
          /* Message stream */
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

            {/* Prominent Thinking & Reasoning Card when AI is generating (OpenCode / Antigravity Style) */}
            {isGenerating && (
              <div className="w-full max-w-3xl mx-auto px-4 py-2">
                <div className="p-3 rounded-xl border border-blue-500/30 bg-blue-50/50 dark:bg-blue-950/20 text-xs shadow-sm space-y-2">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className="w-5 h-5 rounded-full bg-blue-500/20 text-blue-600 dark:text-blue-400 flex items-center justify-center animate-pulse">
                        <Brain className="w-3.5 h-3.5" />
                      </div>
                      <span className="font-semibold text-zinc-900 dark:text-zinc-100 font-mono text-[12px]">
                        Thinking for {thinkingElapsed}s
                      </span>
                    </div>

                    <div className="flex items-center gap-2 text-[11px] font-mono text-zinc-500 dark:text-zinc-400">
                      <span className="animate-pulse text-blue-500 font-medium">streaming</span>
                    </div>
                  </div>

                  {/* Live Thinking Preview Text */}
                  <div className="p-2.5 rounded-lg bg-white/90 dark:bg-[#121215]/90 border border-zinc-200/60 dark:border-zinc-800/60 text-[11px] font-mono text-zinc-700 dark:text-zinc-300 leading-relaxed max-h-40 overflow-y-auto whitespace-pre-wrap">
                    <div className="flex items-start gap-2">
                      <Loader2 className="w-3.5 h-3.5 animate-spin text-blue-500 flex-shrink-0 mt-0.5" />
                      <div className="space-y-1">
                        <p>{liveThinking || t('Đang kết nối tới mô hình AI và nạp công cụ workspace...', 'Connecting to AI model & loading workspace tools...')}</p>
                        {thinkingElapsed >= 25 && (
                          <p className="text-[10px] text-zinc-400 italic">
                            {t('Đang suy nghĩ chuyên sâu hoặc proxy local đang chuyển tiếp request...', 'Deep thinking in progress or local proxy is forwarding upstream request...')}
                          </p>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* Error Message Card if generation fails */}
            {errorMessage && (
              <div className="w-full max-w-3xl mx-auto px-4 py-2">
                <div className="p-3.5 rounded-2xl border border-red-500/30 bg-red-50/70 dark:bg-red-950/20 text-xs shadow-sm space-y-2">
                  <div className="flex items-start gap-2.5">
                    <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" />
                    <div className="flex-1 space-y-1">
                      <div className="font-semibold text-red-700 dark:text-red-300">
                        {t('Không thể nhận phản hồi từ AI', 'AI Response Failed')}
                      </div>
                      <p className="text-zinc-600 dark:text-zinc-400 font-mono text-[11px] leading-relaxed break-all">
                        {errorMessage}
                      </p>
                    </div>
                  </div>

                  {onOpenSettings && (
                    <div className="pt-1 flex justify-end">
                      <button
                        onClick={onOpenSettings}
                        className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 text-zinc-800 dark:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-700 text-xs font-medium transition-colors"
                      >
                        <Settings2 className="w-3.5 h-3.5 text-blue-500" />
                        <span>{t('Kiểm tra cấu hình AI (Settings)', 'Check AI Settings')}</span>
                      </button>
                    </div>
                  )}
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
          className="absolute bottom-24 right-6 p-2 rounded-full bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 text-zinc-600 dark:text-zinc-300 hover:text-zinc-900 dark:hover:text-zinc-100 shadow-md transition-all z-10 animate-fade-in"
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
