import React, { useState } from 'react';
import { Copy, Check, Sparkles, Brain, ChevronDown, ChevronRight } from 'lucide-react';
import { Message } from '../types';
import { CodeBlock } from './CodeBlock';
import { ContextualActivity, ToolActivityItem } from './ContextualActivity';
import { useTheme } from '../context/ThemeContext';

interface MessageItemProps {
  message: Message;
  activities?: ToolActivityItem[];
}

export const MessageItem: React.FC<MessageItemProps> = ({ message, activities = [] }) => {
  const { t } = useTheme();
  const [copied, setCopied] = useState(false);
  const [isThinkingExpanded, setIsThinkingExpanded] = useState(false);
  const isUser = message.role === 'user';

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(cleanContentForCopy(message.content));
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      console.error('Failed to copy message', e);
    }
  };

  const formattedTime = React.useMemo(() => {
    if (!message.created_at) return '';
    try {
      return new Date(message.created_at).toLocaleTimeString([], {
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return '';
    }
  }, [message.created_at]);

  // Extract <think>...</think> if present in assistant response
  const { thinkingText, mainContent } = React.useMemo(() => {
    if (isUser || !message.content) {
      return { thinkingText: null, mainContent: message.content || '' };
    }

    const thinkMatch = message.content.match(/<think>([\s\S]*?)<\/think>/);
    if (thinkMatch) {
      const thinking = thinkMatch[1].trim();
      const rest = message.content.replace(/<think>[\s\S]*?<\/think>/, '').trim();
      return { thinkingText: thinking, mainContent: rest };
    }

    return { thinkingText: null, mainContent: message.content };
  }, [isUser, message.content]);

  return (
    <div
      className={`group w-full max-w-3xl mx-auto px-4 py-3 transition-colors ${
        isUser ? 'flex justify-end' : 'flex justify-start'
      }`}
    >
      <div
        className={`relative max-w-[88%] sm:max-w-[84%] rounded-2xl ${
          isUser
            ? 'bg-zinc-100 dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 px-4 py-2.5 rounded-br-sm border border-zinc-200 dark:border-zinc-700 shadow-sm'
            : 'bg-transparent text-zinc-800 dark:text-zinc-200 px-1 py-1 w-full'
        }`}
      >
        {/* Assistant Header */}
        {!isUser && (
          <div className="flex items-center justify-between mb-2 select-none text-xs text-zinc-400 dark:text-zinc-500">
            <div className="flex items-center gap-1.5">
              <div className="w-5 h-5 rounded-full bg-blue-500/10 text-blue-500 flex items-center justify-center">
                <Sparkles className="w-3 h-3" />
              </div>
              <span className="font-semibold text-zinc-800 dark:text-zinc-200 text-xs">SIA</span>
              {formattedTime && <span className="text-[11px] text-zinc-400 dark:text-zinc-500">· {formattedTime}</span>}
            </div>

            <button
              onClick={handleCopy}
              className="opacity-0 group-hover:opacity-100 p-1 text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 transition-opacity"
              title={t('Sao chép tin nhắn', 'Copy message')}
            >
              {copied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
            </button>
          </div>
        )}

        {/* Thinking / Reasoning Accordion if assistant has thoughts */}
        {!isUser && thinkingText && (
          <div className="mb-3 rounded-xl border border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-900/60 overflow-hidden text-xs">
            <button
              onClick={() => setIsThinkingExpanded(!isThinkingExpanded)}
              className="w-full flex items-center justify-between px-3 py-2 text-left text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800/50 transition-colors select-none font-medium"
            >
              <div className="flex items-center gap-2">
                <Brain className="w-3.5 h-3.5 text-blue-500 flex-shrink-0" />
                <span>{t('Quá trình suy nghĩ của AI', 'AI Thought Process')}</span>
              </div>
              {isThinkingExpanded ? (
                <ChevronDown className="w-3.5 h-3.5" />
              ) : (
                <ChevronRight className="w-3.5 h-3.5" />
              )}
            </button>

            {isThinkingExpanded && (
              <div className="px-3 pb-3 pt-1 border-t border-zinc-200 dark:border-zinc-800 text-[11px] text-zinc-600 dark:text-zinc-400 leading-relaxed font-mono whitespace-pre-wrap">
                {thinkingText}
              </div>
            )}
          </div>
        )}

        {/* Contextual activities associated with this assistant message */}
        {!isUser && activities.length > 0 && (
          <div className="mb-2 space-y-1">
            {activities.map((act) => (
              <ContextualActivity key={act.id} activity={act} />
            ))}
          </div>
        )}

        {/* Message Content */}
        <div className="text-sm leading-relaxed break-words space-y-2">
          {renderFormattedContent(mainContent)}
        </div>

        {/* User Time */}
        {isUser && formattedTime && (
          <div className="text-[10px] text-zinc-400 dark:text-zinc-500 text-right mt-1 select-none">
            {formattedTime}
          </div>
        )}
      </div>
    </div>
  );
};

function cleanContentForCopy(content: string): string {
  return content.replace(/<think>[\s\S]*?<\/think>/g, '').trim();
}

function renderFormattedContent(content: string) {
  if (!content) return null;

  const codeBlockRegex = /```([a-zA-Z0-9_-]*)\n([\s\S]*?)```/g;
  const parts: React.ReactNode[] = [];
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = codeBlockRegex.exec(content)) !== null) {
    const textBefore = content.substring(lastIndex, match.index);
    if (textBefore) {
      parts.push(
        <div key={`text-${lastIndex}`} className="space-y-1 whitespace-pre-wrap">
          {renderInlineMarkdown(textBefore)}
        </div>
      );
    }

    const language = match[1] || 'text';
    const code = match[2];
    parts.push(
      <CodeBlock key={`code-${match.index}`} language={language} code={code.trimEnd()} />
    );

    lastIndex = match.index + match[0].length;
  }

  const remaining = content.substring(lastIndex);
  if (remaining) {
    parts.push(
      <div key={`text-${lastIndex}`} className="space-y-1 whitespace-pre-wrap">
        {renderInlineMarkdown(remaining)}
      </div>
    );
  }

  return <>{parts}</>;
}

function renderInlineMarkdown(text: string): React.ReactNode {
  const lines = text.split('\n');

  return (
    <>
      {lines.map((line, i) => {
        if (line.trim().startsWith('- ') || line.trim().startsWith('* ')) {
          return (
            <div key={i} className="flex items-start gap-2 ml-2 my-0.5">
              <span className="text-blue-500 select-none mt-1 leading-none">•</span>
              <span className="flex-1">{formatInlineSpans(line.trim().substring(2))}</span>
            </div>
          );
        }

        if (line.startsWith('### ')) {
          return (
            <h4 key={i} className="text-sm font-semibold text-zinc-900 dark:text-zinc-100 mt-2 mb-1">
              {formatInlineSpans(line.substring(4))}
            </h4>
          );
        }
        if (line.startsWith('## ')) {
          return (
            <h3 key={i} className="text-base font-semibold text-zinc-900 dark:text-zinc-100 mt-3 mb-1">
              {formatInlineSpans(line.substring(3))}
            </h3>
          );
        }
        if (line.startsWith('# ')) {
          return (
            <h2 key={i} className="text-lg font-bold text-zinc-900 dark:text-zinc-100 mt-4 mb-2">
              {formatInlineSpans(line.substring(2))}
            </h2>
          );
        }

        if (!line.trim()) {
          return <div key={i} className="h-1.5" />;
        }

        return <p key={i} className="my-0.5">{formatInlineSpans(line)}</p>;
      })}
    </>
  );
}

function formatInlineSpans(text: string): React.ReactNode {
  const regex = /(`[^`]+`|\*\*[^*]+\*\*)/g;
  const parts = text.split(regex);

  return parts.map((part, index) => {
    if (part.startsWith('`') && part.endsWith('`') && part.length >= 2) {
      return (
        <code
          key={index}
          className="px-1.5 py-0.5 mx-0.5 text-xs font-mono rounded bg-zinc-100 dark:bg-zinc-800 text-blue-600 dark:text-blue-400 border border-zinc-200 dark:border-zinc-700"
        >
          {part.slice(1, -1)}
        </code>
      );
    }
    if (part.startsWith('**') && part.endsWith('**') && part.length >= 4) {
      return (
        <strong key={index} className="font-semibold text-zinc-900 dark:text-zinc-100">
          {part.slice(2, -2)}
        </strong>
      );
    }
    return part;
  });
}
