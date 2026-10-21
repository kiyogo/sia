import React, { useState } from 'react';
import { Copy, Check, User, Bot, Sparkles } from 'lucide-react';
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
  const isUser = message.role === 'user';

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(message.content);
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

  return (
    <div
      className={`group w-full max-w-3xl mx-auto px-4 py-3 transition-colors ${
        isUser ? 'flex justify-end' : 'flex justify-start'
      }`}
    >
      <div
        className={`relative max-w-[85%] sm:max-w-[82%] rounded-2xl ${
          isUser
            ? 'bg-muted/70 text-primary px-4 py-2.5 rounded-br-sm border border-border/40'
            : 'bg-transparent text-primary px-1 py-1 w-full'
        }`}
      >
        {/* Assistant Header */}
        {!isUser && (
          <div className="flex items-center justify-between mb-1.5 select-none text-xs text-muted">
            <div className="flex items-center gap-1.5">
              <div className="w-5 h-5 rounded-full bg-accent/10 text-accent flex items-center justify-center">
                <Sparkles className="w-3 h-3" />
              </div>
              <span className="font-medium text-secondary text-xs">SIA</span>
              {formattedTime && <span className="text-[11px] text-muted">· {formattedTime}</span>}
            </div>

            <button
              onClick={handleCopy}
              className="opacity-0 group-hover:opacity-100 p-1 text-muted hover:text-primary transition-opacity"
              title={t('Sao chép tin nhắn', 'Copy message')}
            >
              {copied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
            </button>
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
        <div className="text-sm leading-relaxed break-words">
          {renderFormattedContent(message.content)}
        </div>

        {/* User Time */}
        {isUser && formattedTime && (
          <div className="text-[10px] text-muted text-right mt-1 select-none">
            {formattedTime}
          </div>
        )}
      </div>
    </div>
  );
};

// Render markdown blocks: fenced code blocks and formatted paragraphs
function renderFormattedContent(content: string) {
  if (!content) return null;

  // Split content by code blocks ```lang ... ```
  const codeBlockRegex = /```([a-zA-Z0-9_-]*)\n([\s\S]*?)```/g;
  const parts: React.ReactNode[] = [];
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = codeBlockRegex.exec(content)) !== null) {
    const textBefore = content.substring(lastIndex, match.index);
    if (textBefore) {
      parts.push(
        <div key={`text-${lastIndex}`} className="space-y-2 whitespace-pre-wrap">
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
      <div key={`text-${lastIndex}`} className="space-y-2 whitespace-pre-wrap">
        {renderInlineMarkdown(remaining)}
      </div>
    );
  }

  return <>{parts}</>;
}

// Render bold **text**, inline `code`, bullet points
function renderInlineMarkdown(text: string): React.ReactNode {
  const lines = text.split('\n');

  return (
    <>
      {lines.map((line, i) => {
        // Bullet list
        if (line.trim().startsWith('- ') || line.trim().startsWith('* ')) {
          return (
            <div key={i} className="flex items-start gap-2 ml-2 my-0.5">
              <span className="text-accent select-none mt-1 leading-none">•</span>
              <span className="flex-1">{formatInlineSpans(line.trim().substring(2))}</span>
            </div>
          );
        }

        // Heading ##
        if (line.startsWith('### ')) {
          return (
            <h4 key={i} className="text-sm font-semibold text-primary mt-2 mb-1">
              {formatInlineSpans(line.substring(4))}
            </h4>
          );
        }
        if (line.startsWith('## ')) {
          return (
            <h3 key={i} className="text-base font-semibold text-primary mt-3 mb-1">
              {formatInlineSpans(line.substring(3))}
            </h3>
          );
        }
        if (line.startsWith('# ')) {
          return (
            <h2 key={i} className="text-lg font-bold text-primary mt-4 mb-2">
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
  // Replace inline `code` and **bold**
  const regex = /(`[^`]+`|\*\*[^*]+\*\*)/g;
  const parts = text.split(regex);

  return parts.map((part, index) => {
    if (part.startsWith('`') && part.endsWith('`') && part.length >= 2) {
      return (
        <code
          key={index}
          className="px-1.5 py-0.5 mx-0.5 text-xs font-mono rounded bg-muted text-accent border border-border/50"
        >
          {part.slice(1, -1)}
        </code>
      );
    }
    if (part.startsWith('**') && part.endsWith('**') && part.length >= 4) {
      return (
        <strong key={index} className="font-semibold text-primary">
          {part.slice(2, -2)}
        </strong>
      );
    }
    return part;
  });
}
