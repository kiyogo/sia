import React, { useState, useEffect, useRef } from 'react';
import { Conversation, Message, AgentEvent } from '../types';
import { invokeCommand, listenToEvent } from '../lib/tauriBridge';
import {
  Send,
  Plus,
  Bot,
  User,
  Wrench,
  CheckCircle2,
  AlertCircle,
  Square,
  Sparkles,
  Clock,
  Coins,
} from 'lucide-react';

interface ChatProps {
  onTaskStarted?: (taskId: string) => void;
}

export const Chat: React.FC<ChatProps> = ({ onTaskStarted }) => {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [activeConvId, setActiveConvId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputPrompt, setInputPrompt] = useState('');
  const [isGenerating, setIsGenerating] = useState(false);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const [liveEvents, setLiveEvents] = useState<AgentEvent[]>([]);
  const [tokenStats, setTokenStats] = useState<{ in: number; out: number; total: number }>({
    in: 0,
    out: 0,
    total: 0,
  });

  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    loadConversations();
  }, []);

  useEffect(() => {
    if (activeConvId) {
      loadMessages(activeConvId);
    }
  }, [activeConvId]);

  useEffect(() => {
    scrollToBottom();
  }, [messages, liveEvents]);

  // Listen to streaming agent events from runtime host
  useEffect(() => {
    let unlistenFn: (() => void) | null = null;
    listenToEvent<{ task_id: string; event: AgentEvent }>('agent-event', (data) => {
      const evt = data.event;
      setLiveEvents((prev) => [...prev, evt]);

      if ('TokenUsageUpdated' in evt) {
        setTokenStats({
          in: evt.TokenUsageUpdated.prompt_tokens,
          out: evt.TokenUsageUpdated.completion_tokens,
          total: evt.TokenUsageUpdated.total_tokens,
        });
      }

      if ('FinalAnswer' in evt || 'TaskStatusChanged' in evt || 'Error' in evt) {
        if ('FinalAnswer' in evt) {
          // Refresh message history when finished
          if (activeConvId) {
            loadMessages(activeConvId);
          }
        }
        setIsGenerating(false);
      }
    }).then((fn) => {
      unlistenFn = fn;
    });

    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, [activeConvId]);

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

  const loadMessages = async (convId: string) => {
    try {
      const list = await invokeCommand<Message[]>('get_messages', { conversationId: convId });
      setMessages(list);
    } catch (e) {
      console.error('Failed to load messages', e);
    }
  };

  const handleNewConversation = async () => {
    const title = `Session ${new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
    try {
      const created = await invokeCommand<Conversation>('create_conversation', { title });
      setConversations([created, ...conversations]);
      setActiveConvId(created.id);
      setMessages([]);
      setLiveEvents([]);
    } catch (e) {
      console.error('Failed to create conversation', e);
    }
  };

  const handleSendMessage = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputPrompt.trim() || isGenerating) return;

    let convId = activeConvId;
    if (!convId) {
      const title = inputPrompt.trim().slice(0, 24);
      const created = await invokeCommand<Conversation>('create_conversation', { title });
      setConversations([created, ...conversations]);
      setActiveConvId(created.id);
      convId = created.id;
    }

    const currentText = inputPrompt.trim();
    setInputPrompt('');
    setIsGenerating(true);
    setLiveEvents([]);

    // Optimistically show user message
    const optimisticUserMsg: Message = {
      id: String(Date.now()),
      conversation_id: convId,
      role: 'user',
      content: currentText,
      tool_calls: null,
      tool_call_id: null,
      token_count: Math.ceil(currentText.length / 4),
      created_at: new Date().toISOString(),
    };
    setMessages((prev) => [...prev, optimisticUserMsg]);

    try {
      const taskId = await invokeCommand<string>('send_chat_message', {
        conversationId: convId,
        content: currentText,
      });
      setActiveTaskId(taskId);
      if (onTaskStarted) onTaskStarted(taskId);
    } catch (err: any) {
      console.error('Failed to send message', err);
      setIsGenerating(false);
      setLiveEvents((prev) => [
        ...prev,
        { Error: { message: err?.toString() || 'Execution failed' } },
      ]);
    }
  };

  const handleStopTask = async () => {
    if (activeTaskId) {
      try {
        await invokeCommand('cancel_task', { taskId: activeTaskId });
      } catch (e) {
        console.error('Failed to stop task', e);
      }
    }
    setIsGenerating(false);
  };

  return (
    <div className="flex-1 flex h-full overflow-hidden">
      {/* Conversation Sessions List */}
      <div className="w-64 border-r border-slate-800 bg-slate-950/70 flex flex-col justify-between">
        <div className="p-3 border-b border-slate-800">
          <button
            onClick={handleNewConversation}
            className="w-full flex items-center justify-center gap-2 px-3 py-2 text-xs font-semibold rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white shadow transition"
          >
            <Plus className="w-4 h-4" />
            <span>New Chat Session</span>
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {conversations.map((c) => {
            const isActive = c.id === activeConvId;
            return (
              <button
                key={c.id}
                onClick={() => {
                  setActiveConvId(c.id);
                  setLiveEvents([]);
                }}
                className={`w-full text-left px-3 py-2.5 rounded-lg text-xs transition truncate ${
                  isActive
                    ? 'bg-slate-800 text-slate-100 font-medium border border-slate-700'
                    : 'text-slate-400 hover:bg-slate-900 hover:text-slate-200'
                }`}
              >
                <div className="truncate font-medium">{c.title}</div>
                <div className="text-[10px] text-slate-400 mt-0.5">
                  {new Date(c.updated_at).toLocaleDateString()}
                </div>
              </button>
            );
          })}
        </div>

        {/* Token Usage Badge */}
        <div className="p-3 border-t border-slate-800 text-[11px] text-slate-400 flex items-center justify-between">
          <span className="flex items-center gap-1">
            <Coins className="w-3.5 h-3.5 text-amber-400" />
            Tokens
          </span>
          <span className="font-mono text-slate-300">
            {tokenStats.total > 0 ? tokenStats.total.toLocaleString() : '0'}
          </span>
        </div>
      </div>

      {/* Main Chat Stream Area */}
      <div className="flex-1 flex flex-col bg-slate-950 overflow-hidden">
        {/* Messages Scroll Area */}
        <div className="flex-1 overflow-y-auto p-6 space-y-4">
          {messages.length === 0 && liveEvents.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-center p-8 text-slate-500">
              <Bot className="w-12 h-12 text-indigo-400/50 mb-3" />
              <h3 className="text-base font-semibold text-slate-300">Universal Autonomous AI Agent</h3>
              <p className="text-xs text-slate-400 max-w-md mt-1">
                Ask a question, request terminal commands, explore workspace files, or control the Windows desktop.
              </p>
            </div>
          ) : (
            messages.map((m) => {
              const isUser = m.role === 'user';
              const isTool = m.role === 'tool';
              return (
                <div
                  key={m.id}
                  className={`flex gap-3 text-xs ${isUser ? 'justify-end' : 'justify-start'}`}
                >
                  {!isUser && (
                    <div className="w-7 h-7 rounded-lg bg-indigo-600/20 border border-indigo-500/30 flex items-center justify-center text-indigo-400 flex-shrink-0">
                      {isTool ? <Wrench className="w-3.5 h-3.5" /> : <Bot className="w-3.5 h-3.5" />}
                    </div>
                  )}

                  <div
                    className={`max-w-2xl rounded-xl p-3.5 leading-relaxed select-text ${
                      isUser
                        ? 'bg-indigo-600 text-white rounded-br-none shadow-sm'
                        : isTool
                        ? 'bg-slate-900 border border-slate-800 text-slate-300 font-mono text-[11px]'
                        : 'bg-slate-900/90 border border-slate-800 text-slate-200 rounded-bl-none'
                    }`}
                  >
                    <div className="whitespace-pre-wrap">{m.content}</div>
                  </div>

                  {isUser && (
                    <div className="w-7 h-7 rounded-lg bg-slate-800 flex items-center justify-center text-slate-300 flex-shrink-0">
                      <User className="w-3.5 h-3.5" />
                    </div>
                  )}
                </div>
              );
            })
          )}

          {/* Real-time Streaming Events & Tool Execution Cards */}
          {liveEvents.map((evt, idx) => {
            if ('ModelThinking' in evt) {
              return (
                <div key={idx} className="flex items-center gap-2 text-xs text-indigo-400 italic">
                  <Sparkles className="w-3.5 h-3.5 animate-pulse" />
                  <span>{evt.ModelThinking.preview}</span>
                </div>
              );
            }

            if ('ToolCallStarted' in evt) {
              return (
                <div
                  key={idx}
                  className="p-3 bg-slate-900/80 border border-slate-800 rounded-lg text-xs space-y-1.5"
                >
                  <div className="flex items-center gap-2 text-amber-400 font-semibold">
                    <Wrench className="w-3.5 h-3.5 animate-spin" />
                    <span>Calling Tool: {evt.ToolCallStarted.tool_name}</span>
                  </div>
                  <pre className="p-2 bg-slate-950 rounded border border-slate-800 font-mono text-[10px] text-slate-300 overflow-x-auto">
                    {evt.ToolCallStarted.arguments}
                  </pre>
                </div>
              );
            }

            if ('ToolCallFinished' in evt) {
              const res = evt.ToolCallFinished;
              return (
                <div
                  key={idx}
                  className={`p-3 rounded-lg border text-xs space-y-1.5 ${
                    res.is_error
                      ? 'bg-rose-950/20 border-rose-800/40 text-rose-300'
                      : 'bg-slate-900/60 border-slate-800 text-slate-300'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span className="flex items-center gap-1.5 font-medium">
                      {res.is_error ? (
                        <AlertCircle className="w-3.5 h-3.5 text-rose-400" />
                      ) : (
                        <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                      )}
                      {res.tool_name} result (exit: {res.exit_code ?? 0})
                    </span>
                    <span className="text-[10px] text-slate-400 flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      {res.duration_ms} ms
                    </span>
                  </div>
                  {res.truncated && (
                    <div className="text-[10px] text-amber-400 italic">
                      [Output truncated by Token Optimizer]
                    </div>
                  )}
                </div>
              );
            }

            if ('FinalAnswer' in evt) {
              return (
                <div
                  key={idx}
                  className="p-4 bg-slate-900 border border-slate-800 rounded-xl text-xs text-slate-200 select-text leading-relaxed whitespace-pre-wrap"
                >
                  {evt.FinalAnswer.content}
                </div>
              );
            }

            if ('Error' in evt) {
              return (
                <div
                  key={idx}
                  className="p-3 bg-rose-950/30 border border-rose-800 rounded-lg text-xs text-rose-300 flex items-center gap-2"
                >
                  <AlertCircle className="w-4 h-4 text-rose-400 flex-shrink-0" />
                  <span>{evt.Error.message}</span>
                </div>
              );
            }

            return null;
          })}

          <div ref={messagesEndRef} />
        </div>

        {/* Input Bar */}
        <div className="p-4 border-t border-slate-800 bg-slate-950/80 backdrop-blur">
          <form onSubmit={handleSendMessage} className="flex gap-2">
            <input
              type="text"
              value={inputPrompt}
              onChange={(e) => setInputPrompt(e.target.value)}
              placeholder={
                isGenerating
                  ? 'Agent is actively executing steps...'
                  : 'Instruct AI (e.g., check cargo version, create file, run build...)'
              }
              disabled={isGenerating}
              className="flex-1 bg-slate-900 border border-slate-800 rounded-xl px-4 py-2.5 text-xs text-slate-100 placeholder-slate-500 focus:outline-none focus:border-indigo-500/80 transition"
            />

            {isGenerating ? (
              <button
                type="button"
                onClick={handleStopTask}
                className="px-4 py-2.5 rounded-xl bg-rose-600 hover:bg-rose-500 text-white font-medium text-xs flex items-center gap-1.5 transition shadow"
              >
                <Square className="w-3.5 h-3.5" />
                <span>Stop</span>
              </button>
            ) : (
              <button
                type="submit"
                disabled={!inputPrompt.trim()}
                className="px-4 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 disabled:hover:bg-indigo-600 text-white font-medium text-xs flex items-center gap-1.5 transition shadow"
              >
                <Send className="w-3.5 h-3.5" />
                <span>Execute</span>
              </button>
            )}
          </form>
        </div>
      </div>
    </div>
  );
};
