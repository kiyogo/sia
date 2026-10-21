import React, { useState } from 'react';
import { ShieldAlert, Check, X, Terminal, FileEdit, Monitor } from 'lucide-react';
import { ApprovalRequest } from '../types';
import { useTheme } from '../context/ThemeContext';

interface InlineApprovalCardProps {
  request: ApprovalRequest;
  onResolve: (id: string, approved: boolean) => Promise<void>;
}

export const InlineApprovalCard: React.FC<InlineApprovalCardProps> = ({
  request,
  onResolve,
}) => {
  const { t } = useTheme();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleAction = async (approved: boolean) => {
    setIsSubmitting(true);
    try {
      await onResolve(request.id, approved);
    } catch (e) {
      console.error('Failed to resolve approval', e);
    } finally {
      setIsSubmitting(false);
    }
  };

  const getTierIcon = () => {
    switch (request.tier) {
      case 'ShellExecution':
        return <Terminal className="w-4 h-4 text-amber-500" />;
      case 'WorkspaceWrite':
      case 'DestructiveAction':
        return <FileEdit className="w-4 h-4 text-orange-500" />;
      case 'DesktopControl':
        return <Monitor className="w-4 h-4 text-purple-500" />;
      default:
        return <ShieldAlert className="w-4 h-4 text-amber-500" />;
    }
  };

  return (
    <div className="my-3 p-3.5 rounded-xl border border-amber-500/30 bg-amber-50/60 dark:bg-amber-950/20 text-xs shadow-sm">
      <div className="flex items-start gap-2.5">
        <div className="p-1.5 rounded-lg bg-amber-500/10 text-amber-500 flex-shrink-0 mt-0.5">
          {getTierIcon()}
        </div>

        <div className="flex-1 space-y-1">
          <div className="flex items-center gap-2">
            <span className="font-semibold text-zinc-900 dark:text-zinc-100">
              {t('Yêu cầu phê duyệt hành động', 'Permission Approval Required')}
            </span>
            <span className="px-1.5 py-0.5 rounded text-[10px] font-mono font-medium bg-amber-500/15 text-amber-700 dark:text-amber-300">
              {request.tier}
            </span>
          </div>

          <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">{request.description}</p>

          {/* Details if available */}
          {request.details && (
            <pre className="mt-1 p-2 rounded bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 text-[11px] font-mono text-zinc-700 dark:text-zinc-300 max-h-32 overflow-x-auto whitespace-pre-wrap">
              {typeof request.details === 'object'
                ? JSON.stringify(request.details, null, 2)
                : String(request.details)}
            </pre>
          )}

          {/* Actions */}
          <div className="flex items-center gap-2 pt-2">
            <button
              onClick={() => handleAction(true)}
              disabled={isSubmitting}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-700 text-white font-medium text-xs transition-colors shadow-sm disabled:opacity-50"
            >
              <Check className="w-3.5 h-3.5" />
              <span>{t('Chấp thuận', 'Approve')}</span>
            </button>

            <button
              onClick={() => handleAction(false)}
              disabled={isSubmitting}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-100 hover:bg-zinc-200 dark:bg-zinc-800 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 font-medium text-xs transition-colors disabled:opacity-50"
            >
              <X className="w-3.5 h-3.5" />
              <span>{t('Từ chối', 'Reject')}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
