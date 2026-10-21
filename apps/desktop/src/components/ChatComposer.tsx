import React, { useState, useRef, useEffect } from 'react';
import { ArrowUp, Square, Paperclip, Sparkles, X, FileText } from 'lucide-react';
import { useTheme } from '../context/ThemeContext';

interface ChatComposerProps {
  onSendMessage: (content: string, attachments?: File[]) => void;
  onStopGeneration?: () => void;
  isGenerating: boolean;
  modelName?: string;
  disabled?: boolean;
}

export const ChatComposer: React.FC<ChatComposerProps> = ({
  onSendMessage,
  onStopGeneration,
  isGenerating,
  modelName = 'Universal AI',
  disabled = false,
}) => {
  const { t } = useTheme();
  const [text, setText] = useState('');
  const [isComposing, setIsComposing] = useState(false);
  const [attachedFiles, setAttachedFiles] = useState<{ name: string; size: number }[]>([]);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    textarea.style.height = 'auto';
    const newHeight = Math.min(Math.max(textarea.scrollHeight, 40), 200);
    textarea.style.height = `${newHeight}px`;
  }, [text]);

  const handleSubmit = () => {
    if ((!text.trim() && attachedFiles.length === 0) || isGenerating || disabled) return;

    let fullPrompt = text.trim();
    if (attachedFiles.length > 0) {
      const fileNotice = `\n[Attached files: ${attachedFiles.map((f) => f.name).join(', ')}]`;
      fullPrompt += fileNotice;
    }

    onSendMessage(fullPrompt);
    setText('');
    setAttachedFiles([]);

    if (textareaRef.current) {
      textareaRef.current.style.height = '40px';
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (isComposing || e.nativeEvent.isComposing) {
      return;
    }

    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      const files = Array.from(e.target.files).map((f) => ({
        name: f.name,
        size: f.size,
      }));
      setAttachedFiles((prev) => [...prev, ...files]);
    }
  };

  const removeAttachment = (index: number) => {
    setAttachedFiles((prev) => prev.filter((_, i) => i !== index));
  };

  return (
    <div className="w-full max-w-3xl mx-auto px-4 pb-4 pt-1">
      {/* File attachments badge preview */}
      {attachedFiles.length > 0 && (
        <div className="flex flex-wrap gap-2 mb-2">
          {attachedFiles.map((file, idx) => (
            <div
              key={idx}
              className="flex items-center gap-1.5 px-2.5 py-1 text-xs rounded-md bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300 border border-zinc-200 dark:border-zinc-700"
            >
              <FileText className="w-3.5 h-3.5 text-zinc-400" />
              <span className="truncate max-w-[150px]">{file.name}</span>
              <button
                onClick={() => removeAttachment(idx)}
                className="hover:text-red-500 transition-colors"
              >
                <X className="w-3 h-3" />
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Main Composer Box */}
      <div className="relative flex flex-col rounded-2xl bg-white dark:bg-[#151518] border border-zinc-200 dark:border-zinc-800 shadow-sm focus-within:border-blue-500/70 focus-within:ring-2 focus-within:ring-blue-500/10 transition-all duration-150">
        <textarea
          ref={textareaRef}
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={handleKeyDown}
          onCompositionStart={() => setIsComposing(true)}
          onCompositionEnd={() => setIsComposing(false)}
          placeholder={t('Nhập tin nhắn hoặc giao nhiệm vụ cho AI...', 'Ask a question or assign an autonomous task...')}
          rows={1}
          disabled={disabled}
          className="w-full px-4 pt-3 pb-2 bg-transparent text-zinc-900 dark:text-zinc-100 placeholder-zinc-400 dark:placeholder-zinc-500 resize-none focus:outline-none text-sm leading-relaxed"
          style={{ minHeight: '42px', maxHeight: '200px' }}
        />

        {/* Bottom toolbar */}
        <div className="flex items-center justify-between px-3 pb-2 pt-1">
          {/* Left tools: Attach file & model badge */}
          <div className="flex items-center gap-2">
            <input
              ref={fileInputRef}
              type="file"
              multiple
              onChange={handleFileChange}
              className="hidden"
            />
            <button
              type="button"
              onClick={() => fileInputRef.current?.click()}
              title={t('Đính kèm tệp', 'Attach file')}
              className="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
            >
              <Paperclip className="w-4 h-4" />
            </button>

            <div className="flex items-center gap-1.5 px-2.5 py-0.5 text-[11px] font-medium rounded-full bg-zinc-100 dark:bg-zinc-800/80 text-zinc-600 dark:text-zinc-400 border border-zinc-200/80 dark:border-zinc-700/60 select-none">
              <Sparkles className="w-3 h-3 text-blue-500" />
              <span className="truncate max-w-[140px]">{modelName}</span>
            </div>
          </div>

          {/* Right tool: Send or Stop */}
          <div>
            {isGenerating ? (
              <button
                type="button"
                onClick={onStopGeneration}
                title={t('Dừng tạo phản hồi', 'Stop generation')}
                className="p-2 rounded-xl bg-red-500 text-white hover:bg-red-600 transition-colors shadow-sm"
              >
                <Square className="w-4 h-4 fill-current" />
              </button>
            ) : (
              <button
                type="button"
                onClick={handleSubmit}
                disabled={!text.trim() && attachedFiles.length === 0}
                title={t('Gửi tin nhắn (Enter)', 'Send message (Enter)')}
                className={`p-2 rounded-xl transition-all shadow-sm ${
                  text.trim() || attachedFiles.length > 0
                    ? 'bg-zinc-900 text-white dark:bg-zinc-100 dark:text-zinc-900 hover:opacity-90'
                    : 'bg-zinc-100 dark:bg-zinc-800 text-zinc-400 dark:text-zinc-600 cursor-not-allowed'
                }`}
              >
                <ArrowUp className="w-4 h-4 stroke-[2.5]" />
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Subtle Hint */}
      <div className="flex items-center justify-between mt-1 px-2 text-[11px] text-zinc-400 dark:text-zinc-500 select-none">
        <span>
          {t('Nhấn Enter để gửi · Shift+Enter để xuống dòng', 'Press Enter to send · Shift+Enter for new line')}
        </span>
      </div>
    </div>
  );
};
