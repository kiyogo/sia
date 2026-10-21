import React, { useState } from 'react';
import { Copy, Check } from 'lucide-react';
import { useTheme } from '../context/ThemeContext';

interface CodeBlockProps {
  language?: string;
  code: string;
}

export const CodeBlock: React.FC<CodeBlockProps> = ({ language, code }) => {
  const { t } = useTheme();
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      console.error('Failed to copy code', e);
    }
  };

  const lineCount = code.split('\n').length;

  return (
    <div className="my-3 rounded-xl overflow-hidden border border-border bg-[#0d1117] text-[#c9d1d9] text-xs font-mono">
      {/* Code Header */}
      <div className="flex items-center justify-between px-3.5 py-1.5 bg-[#161b22] border-b border-[#30363d] select-none text-[11px]">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-[#8b949e] uppercase tracking-wider">
            {language || 'text'}
          </span>
          <span className="text-[#484f58]">·</span>
          <span className="text-[#8b949e]">{lineCount} lines</span>
        </div>

        <button
          onClick={handleCopy}
          className="flex items-center gap-1.5 px-2 py-0.5 rounded text-[#8b949e] hover:text-[#f0f6fc] hover:bg-[#21262d] transition-colors"
          title={t('Sao chép mã', 'Copy code')}
        >
          {copied ? (
            <>
              <Check className="w-3.5 h-3.5 text-emerald-400" />
              <span className="text-emerald-400 font-sans text-[11px]">{t('Đã sao chép', 'Copied')}</span>
            </>
          ) : (
            <>
              <Copy className="w-3.5 h-3.5" />
              <span className="font-sans text-[11px]">{t('Sao chép', 'Copy')}</span>
            </>
          )}
        </button>
      </div>

      {/* Code Body */}
      <pre className="p-3.5 overflow-x-auto leading-relaxed">
        <code>{code}</code>
      </pre>
    </div>
  );
};
