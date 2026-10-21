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
  Cpu,
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

  // Parse arguments if JSON
  const parsedArgs = React.useMemo(() => {
    if (!activity.arguments) return null;
    try {
      return JSON.parse(activity.arguments);
    } catch {
      return activity.arguments;
    }
  }, [activity.arguments]);

  // Derive human-readable tool title & icon
  const { icon, title, subtitle } = React.useMemo(() => {
    const name = activity.toolName.toLowerCase();

    if (name.includes('terminal') || name.includes('bash') || name.includes('cmd') || name.includes('powershell')) {
      const cmd = parsedArgs?.command || (typeof parsedArgs === 'string' ? parsedArgs : '');
      return {
        icon: <Terminal className="w-3.5 h-3.5 text-amber-500" />,
        title: t('Thực thi lệnh Terminal', 'Terminal Command'),
        subtitle: cmd ? String(cmd).slice(0, 50) : '',
      };
    }

    if (name.includes('file') || name.includes('read') || name.includes('write')) {
      const path = parsedArgs?.path || parsedArgs?.file_path || '';
      const isWrite = name.includes('write');
      return {
        icon: <FileCode className="w-3.5 h-3.5 text-blue-500" />,
        title: isWrite ? t('Ghi tệp mã nguồn', 'Write File') : t('Đọc tệp mã nguồn', 'Read File'),
        subtitle: path ? String(path) : '',
      };
    }

    if (name.includes('browser') || name.includes('navigate') || name.includes('click')) {
      const url = parsedArgs?.url || '';
      return {
        icon: <Globe className="w-3.5 h-3.5 text-emerald-500" />,
        title: t('Thao tác trình duyệt web', 'Browser Action'),
        subtitle: url ? String(url) : '',
      };
    }

    if (name.includes('desktop') || name.includes('window') || name.includes('screenshot')) {
      return {
        icon: <Monitor className="w-3.5 h-3.5 text-purple-500" />,
        title: t('Điều khiển máy tính desktop', 'Desktop Control'),
        subtitle: activity.toolName,
      };
    }

    return {
      icon: <Cpu className="w-3.5 h-3.5 text-accent" />,
      title: activity.toolName,
      subtitle: '',
    };
  }, [activity.toolName, parsedArgs, t]);

  return (
    <div className="my-2 rounded-lg border border-border bg-surface/80 overflow-hidden text-xs transition-all shadow-sm">
      {/* Activity Summary Bar */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between px-3 py-2 text-left hover:bg-muted/50 transition-colors"
      >
        <div className="flex items-center gap-2 overflow-hidden">
          {activity.status === 'running' && (
            <Loader2 className="w-3.5 h-3.5 text-accent animate-spin flex-shrink-0" />
          )}
          {activity.status === 'completed' && (
            <CheckCircle2 className="w-3.5 h-3.5 text-emerald-500 flex-shrink-0" />
          )}
          {activity.status === 'failed' && (
            <XCircle className="w-3.5 h-3.5 text-red-500 flex-shrink-0" />
          )}

          <div className="flex items-center gap-1.5 overflow-hidden">
            {icon}
            <span className="font-medium text-primary whitespace-nowrap">{title}</span>
            {subtitle && (
              <span className="text-secondary font-mono truncate max-w-[280px] sm:max-w-[380px]">
                ({subtitle})
              </span>
            )}
          </div>
        </div>

        <div className="flex items-center gap-2 flex-shrink-0 text-muted">
          {activity.durationMs !== undefined && (
            <span className="text-[10px] font-mono">{activity.durationMs}ms</span>
          )}
          {isExpanded ? (
            <ChevronDown className="w-3.5 h-3.5" />
          ) : (
            <ChevronRight className="w-3.5 h-3.5" />
          )}
        </div>
      </button>

      {/* Expanded Details */}
      {isExpanded && (
        <div className="px-3 pb-3 pt-1 border-t border-border-subtle bg-muted/20 space-y-2">
          {/* Arguments */}
          {activity.arguments && (
            <div>
              <div className="text-[10px] uppercase font-semibold text-muted mb-1">
                {t('Tham số đầu vào', 'Parameters')}
              </div>
              <pre className="p-2 rounded bg-muted/60 font-mono text-[11px] text-secondary overflow-x-auto leading-relaxed whitespace-pre-wrap">
                {typeof parsedArgs === 'object'
                  ? JSON.stringify(parsedArgs, null, 2)
                  : String(activity.arguments)}
              </pre>
            </div>
          )}

          {/* Output / Result */}
          {activity.output && (
            <div>
              <div className="text-[10px] uppercase font-semibold text-muted mb-1 flex items-center justify-between">
                <span>{t('Kết quả đầu ra', 'Execution Output')}</span>
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
