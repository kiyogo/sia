import React from 'react';
import { AlertOctagon, ShieldAlert, CheckCircle2 } from 'lucide-react';

interface EmergencyStopBannerProps {
  isEmergencyStop: boolean;
  pendingApprovalsCount: number;
  onTriggerStop: () => void;
  onResetStop: () => void;
  onOpenApprovals: () => void;
}

export const EmergencyStopBanner: React.FC<EmergencyStopBannerProps> = ({
  isEmergencyStop,
  pendingApprovalsCount,
  onTriggerStop,
  onResetStop,
  onOpenApprovals,
}) => {
  return (
    <div className="flex items-center justify-between px-6 py-2.5 bg-slate-900 border-b border-slate-800 text-xs">
      <div className="flex items-center gap-4">
        {isEmergencyStop ? (
          <div className="flex items-center gap-2 text-rose-400 font-semibold animate-pulse">
            <AlertOctagon className="w-4 h-4" />
            <span>EMERGENCY STOP ACTIVE — All agent actions halted.</span>
          </div>
        ) : pendingApprovalsCount > 0 ? (
          <div
            onClick={onOpenApprovals}
            className="flex items-center gap-2 text-amber-400 font-medium cursor-pointer hover:underline"
          >
            <ShieldAlert className="w-4 h-4" />
            <span>{pendingApprovalsCount} dangerous operation(s) waiting for your human approval</span>
          </div>
        ) : (
          <div className="flex items-center gap-2 text-slate-400">
            <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            <span>Security policy enforced (Workspace sandbox active)</span>
          </div>
        )}
      </div>

      <div className="flex items-center gap-2">
        {isEmergencyStop ? (
          <button
            onClick={onResetStop}
            className="px-3 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 font-medium transition"
          >
            Resume Operations
          </button>
        ) : (
          <button
            onClick={onTriggerStop}
            className="px-3 py-1 rounded bg-rose-600/90 hover:bg-rose-600 text-white font-medium flex items-center gap-1.5 shadow-sm transition"
          >
            <AlertOctagon className="w-3.5 h-3.5" />
            Emergency Stop
          </button>
        )}
      </div>
    </div>
  );
};
