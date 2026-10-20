import React, { useState } from 'react';
import { ProcessOutput } from '../types';
import { invokeCommand } from '../lib/tauriBridge';
import {
  Terminal as TerminalIcon,
  Play,
  RotateCcw,
  Clock,
  CheckCircle2,
  AlertCircle,
} from 'lucide-react';

interface HistoryItem {
  command: string;
  shell: string;
  output: ProcessOutput;
  timestamp: string;
}

export const TerminalView: React.FC = () => {
  const [shell, setShell] = useState<'powershell' | 'cmd'>('powershell');
  const [commandInput, setCommandInput] = useState('');
  const [isExecuting, setIsExecuting] = useState(false);
  const [history, setHistory] = useState<HistoryItem[]>([]);

  const handleExecute = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!commandInput.trim() || isExecuting) return;

    const cmd = commandInput.trim();
    setCommandInput('');
    setIsExecuting(true);

    try {
      const output = await invokeCommand<ProcessOutput>('execute_terminal_command', {
        command: cmd,
        shell,
      });

      setHistory((prev) => [
        ...prev,
        {
          command: cmd,
          shell,
          output,
          timestamp: new Date().toLocaleTimeString(),
        },
      ]);
    } catch (err: any) {
      console.error('Failed to execute terminal command', err);
      setHistory((prev) => [
        ...prev,
        {
          command: cmd,
          shell,
          output: {
            stdout: '',
            stderr: err?.toString() || 'Execution error',
            exit_code: 1,
            duration_ms: 0,
            truncated: false,
          },
          timestamp: new Date().toLocaleTimeString(),
        },
      ]);
    } finally {
      setIsExecuting(false);
    }
  };

  const handleClear = () => {
    setHistory([]);
  };

  return (
    <div className="flex-1 flex flex-col h-full bg-slate-950 font-mono text-xs">
      {/* Top Header Bar */}
      <div className="p-3 border-b border-slate-800 bg-slate-950/80 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <TerminalIcon className="w-4 h-4 text-indigo-400" />
          <span className="font-semibold text-slate-200">Native Windows Terminal</span>

          {/* Shell Selector */}
          <div className="flex bg-slate-900 border border-slate-800 rounded-lg p-0.5">
            <button
              onClick={() => setShell('powershell')}
              className={`px-2.5 py-1 rounded text-[11px] font-semibold transition ${
                shell === 'powershell'
                  ? 'bg-indigo-600 text-white shadow'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              PowerShell
            </button>
            <button
              onClick={() => setShell('cmd')}
              className={`px-2.5 py-1 rounded text-[11px] font-semibold transition ${
                shell === 'cmd'
                  ? 'bg-indigo-600 text-white shadow'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              CMD
            </button>
          </div>
        </div>

        <button
          onClick={handleClear}
          className="text-xs text-slate-400 hover:text-slate-200 flex items-center gap-1 transition"
        >
          <RotateCcw className="w-3.5 h-3.5" />
          <span>Clear</span>
        </button>
      </div>

      {/* Terminal History Console */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4 select-text">
        {history.length === 0 ? (
          <div className="h-full flex flex-col items-center justify-center text-slate-500 space-y-1">
            <TerminalIcon className="w-8 h-8 text-slate-700" />
            <span>Ready. Enter commands to execute directly on the system.</span>
            <span className="text-[10px] text-slate-400">
              (e.g., git --version, cargo --version, dir)
            </span>
          </div>
        ) : (
          history.map((h, idx) => {
            const isError = h.output.exit_code !== 0;
            return (
              <div key={idx} className="space-y-1.5 border-b border-slate-900 pb-3">
                {/* Prompt command line */}
                <div className="flex items-center justify-between text-slate-400 text-[11px]">
                  <div className="flex items-center gap-2">
                    <span className="text-indigo-400 font-bold">{h.shell}&gt;</span>
                    <span className="text-slate-200 font-semibold">{h.command}</span>
                  </div>
                  <div className="flex items-center gap-3 text-[10px]">
                    <span className="flex items-center gap-1 text-slate-400">
                      <Clock className="w-3 h-3" />
                      {h.output.duration_ms} ms
                    </span>
                    <span
                      className={`flex items-center gap-1 font-semibold ${
                        isError ? 'text-rose-400' : 'text-emerald-400'
                      }`}
                    >
                      {isError ? (
                        <AlertCircle className="w-3 h-3" />
                      ) : (
                        <CheckCircle2 className="w-3 h-3" />
                      )}
                      exit {h.output.exit_code ?? 0}
                    </span>
                  </div>
                </div>

                {/* Stdout */}
                {h.output.stdout && (
                  <pre className="p-3 bg-slate-900/60 rounded-lg text-slate-300 leading-relaxed overflow-x-auto whitespace-pre-wrap">
                    {h.output.stdout}
                  </pre>
                )}

                {/* Stderr */}
                {h.output.stderr && (
                  <pre className="p-3 bg-rose-950/20 border border-rose-900/30 rounded-lg text-rose-300 leading-relaxed overflow-x-auto whitespace-pre-wrap">
                    {h.output.stderr}
                  </pre>
                )}
              </div>
            );
          })
        )}
      </div>

      {/* Command Input Prompt Bar */}
      <div className="p-3 border-t border-slate-800 bg-slate-950">
        <form onSubmit={handleExecute} className="flex gap-2">
          <div className="flex items-center px-3 bg-slate-900 border border-slate-800 rounded-lg text-indigo-400 font-bold text-xs select-none">
            {shell}&gt;
          </div>
          <input
            type="text"
            value={commandInput}
            onChange={(e) => setCommandInput(e.target.value)}
            placeholder={isExecuting ? 'Running command...' : 'Type command and press Enter...'}
            disabled={isExecuting}
            className="flex-1 bg-slate-900 border border-slate-800 rounded-lg px-4 py-2 text-xs text-slate-100 placeholder-slate-500 focus:outline-none focus:border-indigo-500/80 font-mono transition"
          />
          <button
            type="submit"
            disabled={isExecuting || !commandInput.trim()}
            className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white font-medium text-xs flex items-center gap-1.5 transition shadow"
          >
            <Play className="w-3.5 h-3.5" />
            <span>Run</span>
          </button>
        </form>
      </div>
    </div>
  );
};
