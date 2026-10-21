import React, { useState } from 'react';
import {
  Terminal,
  FileCode,
  Globe,
  Monitor,
  CheckCircle2,
  XCircle,
  Loader2,
  ChevronDown,
  ChevronRight,
  Code2,
  ListTodo,
  Search,
} from 'lucide-react';
import { useTheme } from '../context/ThemeContext';

export interface ToolActivityItem {
  id: string;
  toolName: string;
  arguments?: string;
  status: 'running' | 'completed' | 'failed';
  output?: string;
  durationMs?: number;
  exitCode?: number | null;
}

interface ContextualActivityProps {
  activity: ToolActivityItem;
}

export const ContextualActivity: React.FC<ContextualActivityProps> = ({ activity }) => {
  const { t } = useTheme();
  const [isExpanded, setIsExpanded] = useState(false);

  const parsedArgs = React.useMemo(() => {
    if (!activity.arguments) return null;
    try {
      return JSON.parse(activity.arguments);
    } catch {
      return activity.arguments;
    }
  }, [activity.arguments]);

  // Antigravity & OpenCode style compact badge formatting
  const { actionVerb, targetText, icon } = React.useMemo(() => {
    const name = activity.toolName.toLowerCase();

    if (name.includes('todo')) {
      const todos = parsedArgs?.todos || [];
      const completed = Array.isArray(todos) ? todos.filter((t: any) => t.status === 'completed').length : 0;
      const total = Array.isArray(todos) ? todos.length : 0;
      return {
        actionVerb: 'Planned',
        targetText: total > 0 ? `todos (${completed}/${total} done)` : 'working plan',
        icon: <ListTodo className="w-3.5 h-3.5 text-blue-500 flex-shrink-0" />,
      };
    }

    if (name.includes('glob') || name.includes('grep') || name.includes('search')) {
      const pattern = parsedArgs?.pattern || '';
      return {
        actionVerb: 'Searched',
        targetText: pattern ? `pattern "${pattern}"` : 'codebase',
        icon: <Search className="w-3.5 h-3.5 text-indigo-500 flex-shrink-0" />,
      };
    }

    if (name.includes('terminal') || name.includes('shell') || name.includes('bash') || name.includes('cmd') || name.includes('powershell')) {
      const cmd = parsedArgs?.command || (typeof parsedArgs === 'string' ? parsedArgs : '');
      return {
        actionVerb: 'Ran',
        targetText: cmd ? `$ ${String(cmd).slice(0, 60)}` : 'terminal command',
        icon: <Terminal className="w-3.5 h-3.5 text-amber-500 flex-shrink-0" />,
      };
    }

    if (name.includes('read') || name.includes('cat') || name.includes('open')) {
      const path = parsedArgs?.path || parsedArgs?.file_path || '';
      const lines = parsedArgs?.start_line && parsedArgs?.end_line 
        ? `#L${parsedArgs.start_line}-${parsedArgs.end_line}` 
        : '';
      return {
        actionVerb: 'Analyzed',
        targetText: `${path || 'file'} ${lines}`.trim(),
        icon: <Code2 className="w-3.5 h-3.5 text-blue-500 flex-shrink-0" />,
      };
    }

    if (name.includes('write') || name.includes('edit') || name.includes('patch')) {
      const path = parsedArgs?.path || parsedArgs?.file_path || '';
      return {
        actionVerb: 'Edited',
        targetText: path || 'file',
        icon: <FileCode className="w-3.5 h-3.5 text-emerald-500 flex-shrink-0" />,
      };
    }

    if (name.includes('browser') || name.includes('navigate') || name.includes('click')) {
      const url = parsedArgs?.url || '';
      return {
        actionVerb: 'Browsed',
        targetText: url ? String(url).slice(0, 50) : 'web page',
        icon: <Globe className="w-3.5 h-3.5 text-indigo-500 flex-shrink-0" />,
      };
    }

    if (name.includes('desktop') || name.includes('window')) {
      return {
        actionVerb: 'Interacted with',
        targetText: activity.toolName,
        icon: <Monitor className="w-3.5 h-3.5 text-purple-500 flex-shrink-0" />,
      };
    }

    return {
      actionVerb: 'Executed',
      targetText: activity.toolName,
      icon: <Code2 className="w-3.5 h-3.5 text-zinc-500 flex-shrink-0" />,
    };
  }, [activity.toolName, parsedArgs]);

  return (
    <div className="my-1.5 rounded-lg border border-zinc-200/80 dark:border-zinc-800/80 bg-white dark:bg-[#121215] overflow-hidden text-xs transition-all shadow-sm">
      {/* Sleek Minimal Activity Pill */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between px-2.5 py-1.5 text-left hover:bg-zinc-50 dark:hover:bg-zinc-800/40 transition-colors"
      >
        <div className="flex items-center gap-2 overflow-hidden">
          {activity.status === 'running' && (
            <Loader2 className="w-3 h-3 text-blue-500 animate-spin flex-shrink-0" />
          )}
          {activity.status === 'completed' && (
            <CheckCircle2 className="w-3 h-3 text-emerald-500 flex-shrink-0" />
          )}
          {activity.status === 'failed' && (
            <XCircle className="w-3 h-3 text-red-500 flex-shrink-0" />
          )}

          <div className="flex items-center gap-1.5 overflow-hidden text-[11px] font-mono">
            {icon}
            <span className="text-zinc-500 dark:text-zinc-400 font-sans font-medium">{actionVerb}</span>
            <span className="px-1 py-0.5 rounded bg-zinc-100 dark:bg-zinc-800 text-blue-600 dark:text-blue-400 font-semibold text-[10px] select-none">
              &lt;/&gt;
            </span>
            <span className="text-zinc-800 dark:text-zinc-200 font-medium truncate max-w-[280px] sm:max-w-[420px]">
              {targetText}
            </span>
          </div>
        </div>

        <div className="flex items-center gap-2 flex-shrink-0 text-zinc-400 dark:text-zinc-500">
          {activity.durationMs !== undefined && (
            <span className="text-[10px] font-mono">{activity.durationMs}ms</span>
          )}
          {isExpanded ? (
            <ChevronDown className="w-3 h-3" />
          ) : (
            <ChevronRight className="w-3 h-3" />
          )}
        </div>
      </button>

      {/* Expanded Details */}
      {isExpanded && (
        <div className="px-3 pb-2.5 pt-1.5 border-t border-zinc-100 dark:border-zinc-800/60 bg-zinc-50/50 dark:bg-zinc-900/30 space-y-2">
          {activity.arguments && (
            <div>
              <div className="text-[10px] uppercase font-semibold text-zinc-400 dark:text-zinc-500 mb-1">
                {t('Tham số', 'Parameters')}
              </div>
              <pre className="p-2 rounded bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 font-mono text-[11px] text-zinc-700 dark:text-zinc-300 overflow-x-auto leading-relaxed whitespace-pre-wrap">
                {typeof parsedArgs === 'object'
                  ? JSON.stringify(parsedArgs, null, 2)
                  : String(activity.arguments)}
              </pre>
            </div>
          )}

          {activity.output && (
            <div>
              <div className="text-[10px] uppercase font-semibold text-zinc-400 dark:text-zinc-500 mb-1 flex items-center justify-between">
                <span>{t('Kết quả', 'Result')}</span>
                {activity.exitCode !== undefined && (
                  <span className="font-mono text-[10px]">
                    exit: {activity.exitCode ?? 0}
                  </span>
                )}
              </div>
              <pre className="p-2 rounded bg-[#0d1117] text-[#c9d1d9] font-mono text-[11px] overflow-x-auto max-h-48 leading-relaxed whitespace-pre-wrap">
                {activity.output}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
