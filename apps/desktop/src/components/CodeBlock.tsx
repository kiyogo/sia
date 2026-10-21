import React, { useState } from 'react';
import { Copy, Check, Play, Loader2, ChevronDown, ChevronUp, Terminal, AlertCircle } from 'lucide-react';
import { useTheme } from '../context/ThemeContext';
import { invokeCommand } from '../lib/tauriBridge';

interface CodeBlockProps {
  language?: string;
  code: string;
}

interface ExecutionOutput {
  stdout: string;
  stderr: string;
  exit_code: number | null;
  duration_ms: number;
  truncated?: boolean;
}

export const CodeBlock: React.FC<CodeBlockProps> = ({ language, code }) => {
  const { t } = useTheme();
  const [copied, setCopied] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [runResult, setRunResult] = useState<ExecutionOutput | null>(null);
  const [showOutput, setShowOutput] = useState(true);
  const [runError, setRunError] = useState<string | null>(null);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      console.error('Failed to copy code', e);
    }
  };

  const normalizedLang = (language || '').toLowerCase().trim();
  const isExecutable =
    normalizedLang === 'bash' ||
    normalizedLang === 'sh' ||
    normalizedLang === 'shell' ||
    normalizedLang === 'powershell' ||
    normalizedLang === 'ps1' ||
    normalizedLang === 'cmd' ||
    normalizedLang === 'bat' ||
    normalizedLang === 'terminal' ||
    normalizedLang === 'python' ||
    normalizedLang === 'py';

  const handleRun = async () => {
    if (isRunning) return;
    setIsRunning(true);
    setRunError(null);
    setShowOutput(true);

    try {
      const shellType =
        normalizedLang === 'cmd' || normalizedLang === 'bat'
          ? 'cmd'
          : normalizedLang === 'bash' || normalizedLang === 'sh'
          ? 'bash'
          : 'powershell';

      const res = await invokeCommand<ExecutionOutput>('execute_terminal_command', {
        command: code,
        shell: shellType,
      });

      setRunResult(res);
    } catch (err: any) {
      setRunError(err?.message || String(err));
    } finally {
      setIsRunning(false);
    }
  };

  const lineCount = code.split('\n').length;

  return (
    <div className="my-3 rounded-xl overflow-hidden border border-border bg-[#0d1117] text-[#c9d1d9] text-xs font-mono shadow-sm">
      {/* Code Header */}
      <div className="flex items-center justify-between px-3.5 py-1.5 bg-[#161b22] border-b border-[#30363d] select-none text-[11px]">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-[#8b949e] uppercase tracking-wider">
            {language || 'text'}
          </span>
          <span className="text-[#484f58]">·</span>
          <span className="text-[#8b949e]">{lineCount} lines</span>
        </div>

        <div className="flex items-center gap-2">
          {/* Direct One-Click Run Button */}
          {isExecutable && (
            <button
              onClick={handleRun}
              disabled={isRunning}
              className={`flex items-center gap-1.5 px-2.5 py-0.5 rounded transition-all font-sans text-[11px] font-medium ${
                isRunning
                  ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30'
                  : 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 hover:bg-emerald-500/25 hover:text-emerald-300'
              }`}
              title={t('Chạy lệnh ngay lập tức', 'Execute command directly')}
            >
              {isRunning ? (
                <>
                  <Loader2 className="w-3 h-3 animate-spin text-amber-400" />
                  <span>{t('Đang chạy...', 'Running...')}</span>
                </>
              ) : (
                <>
                  <Play className="w-3 h-3 text-emerald-400 fill-emerald-400/20" />
                  <span>{t('Chạy lệnh', 'Run')}</span>
                </>
              )}
            </button>
          )}

          {/* Copy Button */}
          <button
            onClick={handleCopy}
            className="flex items-center gap-1.5 px-2 py-0.5 rounded text-[#8b949e] hover:text-[#f0f6fc] hover:bg-[#21262d] transition-colors"
            title={t('Sao chép mã', 'Copy code')}
          >
            {copied ? (
              <>
                <Check className="w-3.5 h-3.5 text-emerald-400" />
                <span className="text-emerald-400 font-sans text-[11px]">
                  {t('Đã sao chép', 'Copied')}
                </span>
              </>
            ) : (
              <>
                <Copy className="w-3.5 h-3.5" />
                <span className="font-sans text-[11px]">{t('Sao chép', 'Copy')}</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Code Body */}
      <pre className="p-3.5 overflow-x-auto leading-relaxed">
        <code>{code}</code>
      </pre>

      {/* Inline Execution Output Drawer */}
      {(runResult !== null || runError !== null) && (
        <div className="border-t border-[#30363d] bg-[#090d13]">
          <div
            onClick={() => setShowOutput(!showOutput)}
            className="flex items-center justify-between px-3.5 py-1 bg-[#13171f] cursor-pointer hover:bg-[#1a202c] transition-colors select-none text-[11px]"
          >
            <div className="flex items-center gap-2">
              <Terminal className="w-3.5 h-3.5 text-zinc-400" />
              <span className="font-semibold text-zinc-300 font-sans">
                {t('Kết quả thực thi', 'Terminal Output')}
              </span>
              {runResult && (
                <span
                  className={`px-1.5 py-0.2 rounded text-[10px] font-mono ${
                    runResult.exit_code === 0
                      ? 'bg-emerald-500/20 text-emerald-400'
                      : 'bg-red-500/20 text-red-400'
                  }`}
                >
                  Exit {runResult.exit_code ?? 'None'} ({runResult.duration_ms}ms)
                </span>
              )}
              {runError && (
                <span className="px-1.5 py-0.2 rounded text-[10px] bg-red-500/20 text-red-400 font-sans flex items-center gap-1">
                  <AlertCircle className="w-3 h-3" /> Error
                </span>
              )}
            </div>

            <button className="text-zinc-400 hover:text-zinc-200">
              {showOutput ? (
                <ChevronUp className="w-3.5 h-3.5" />
              ) : (
                <ChevronDown className="w-3.5 h-3.5" />
              )}
            </button>
          </div>

          {showOutput && (
            <div className="p-3 max-h-60 overflow-y-auto font-mono text-[11px] leading-relaxed border-t border-[#21262d]/60 bg-[#06090f]">
              {runError && <div className="text-red-400 whitespace-pre-wrap">{runError}</div>}
              {runResult?.stdout && (
                <div className="text-zinc-300 whitespace-pre-wrap">{runResult.stdout}</div>
              )}
              {runResult?.stderr && (
                <div className="text-amber-400/90 whitespace-pre-wrap mt-1">
                  {runResult.stderr}
                </div>
              )}
              {!runError && !runResult?.stdout && !runResult?.stderr && (
                <div className="text-zinc-500 italic font-sans">
                  {t('Lệnh kết thúc mà không có output.', 'Command completed with no output.')}
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
