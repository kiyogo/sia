import React from 'react';
import { ApprovalRequest } from '../types';
import { invokeCommand } from '../lib/tauriBridge';
import { ShieldAlert, Check, X } from 'lucide-react';

interface ApprovalDialogProps {
  approvals: ApprovalRequest[];
  onResolved: () => void;
  onClose: () => void;
}

export const ApprovalDialog: React.FC<ApprovalDialogProps> = ({
  approvals,
  onResolved,
  onClose,
}) => {
  if (approvals.length === 0) return null;

  const handleAction = async (id: string, approved: boolean) => {
    try {
      await invokeCommand('resolve_approval', { id, approved });
      onResolved();
    } catch (e) {
      console.error('Failed to resolve approval', e);
    }
  };

  return (
    <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
      <div className="bg-slate-900 border border-slate-700 rounded-2xl max-w-lg w-full p-6 shadow-2xl space-y-4">
        <div className="flex items-center gap-3 text-amber-400">
          <div className="w-10 h-10 rounded-xl bg-amber-500/10 border border-amber-500/30 flex items-center justify-center">
            <ShieldAlert className="w-5 h-5" />
          </div>
          <div>
            <h3 className="font-bold text-sm text-slate-100">Human Approval Required</h3>
            <p className="text-xs text-slate-400">
              The AI Agent is requesting permission to execute a privileged or destructive operation.
            </p>
          </div>
        </div>

        <div className="max-h-80 overflow-y-auto space-y-3">
          {approvals.map((req) => (
            <div
              key={req.id}
              className="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-3 text-xs"
            >
              <div className="flex items-center justify-between">
                <span className="px-2 py-0.5 rounded bg-amber-500/10 text-amber-300 font-semibold border border-amber-500/20 uppercase tracking-wide text-[10px]">
                  {req.tier}
                </span>
                <span className="text-[10px] text-slate-400 font-mono">
                  {new Date(req.requested_at).toLocaleTimeString()}
                </span>
              </div>

              <div className="font-mono text-slate-200 bg-slate-900/60 p-2.5 rounded border border-slate-800 break-all select-text">
                {req.description}
              </div>

              <div className="flex items-center justify-end gap-2 pt-1">
                <button
                  onClick={() => handleAction(req.id, false)}
                  className="px-3.5 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 font-medium text-xs flex items-center gap-1 transition"
                >
                  <X className="w-3.5 h-3.5" />
                  <span>Reject</span>
                </button>
                <button
                  onClick={() => handleAction(req.id, true)}
                  className="px-3.5 py-1.5 rounded-lg bg-amber-600 hover:bg-amber-500 text-white font-medium text-xs flex items-center gap-1 transition shadow"
                >
                  <Check className="w-3.5 h-3.5" />
                  <span>Authorize Operation</span>
                </button>
              </div>
            </div>
          ))}
        </div>

        <div className="flex justify-end pt-2 border-t border-slate-800">
          <button
            onClick={onClose}
            className="text-xs text-slate-400 hover:text-slate-200 transition"
          >
            Review Later
          </button>
        </div>
      </div>
    </div>
  );
};
