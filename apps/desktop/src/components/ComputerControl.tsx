import React, { useState, useEffect } from 'react';
import { invokeCommand } from '../lib/tauriBridge';
import { WindowInfo, OwnershipStatusReport } from '../types';
import {
  Monitor,
  ShieldAlert,
  Hand,
  Clock,
  Eye,
  EyeOff,
  RefreshCw,
  AppWindow,
  Lock,
  Unlock,
  CheckCircle2,
  XCircle,
  Zap,
} from 'lucide-react';

interface ComputerControlProps {
  workspacePath: string;
}

export const ComputerControl: React.FC<ComputerControlProps> = () => {
  const [ownership, setOwnership] = useState<OwnershipStatusReport | null>(null);
  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [lockStatus, setLockStatus] = useState<string>('Unlocked');
  const [previewEnabled, setPreviewEnabled] = useState<boolean>(false);
  const [screenshotData, setScreenshotData] = useState<string | null>(null);
  const [loadingAction, setLoadingAction] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<{ text: string; isError?: boolean } | null>(null);

  const fetchStatus = async () => {
    try {
      const own = await invokeCommand<OwnershipStatusReport>('get_ownership_status');
      setOwnership(own);

      const wins = await invokeCommand<WindowInfo[]>('list_desktop_windows');
      setWindows(wins);

      const lock = await invokeCommand<string>('get_desktop_lock_status');
      setLockStatus(lock);
    } catch (e: any) {
      console.warn('Error fetching computer control status', e);
    }
  };

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 3000);
    return () => clearInterval(interval);
  }, []);

  const handleAcquireLease = async () => {
    setLoadingAction('lease');
    try {
      await invokeCommand('acquire_desktop_lease', { agentName: 'DesktopAgent', timeoutSecs: 20 });
      setStatusMessage({ text: 'Acquired 20-second Desktop Ownership lease.' });
      await fetchStatus();
    } catch (e: any) {
      setStatusMessage({ text: `Failed to acquire lease: ${e}`, isError: true });
    } finally {
      setLoadingAction(null);
    }
  };

  const handleUserTakeover = async () => {
    setLoadingAction('takeover');
    try {
      await invokeCommand('user_takeover_desktop');
      setStatusMessage({ text: 'User takeover active: AI input simulation paused.' });
      await fetchStatus();
    } catch (e: any) {
      setStatusMessage({ text: `Failed user takeover: ${e}`, isError: true });
    } finally {
      setLoadingAction(null);
    }
  };

  const handleReleaseUserControl = async () => {
    try {
      await invokeCommand('release_user_control_desktop');
      setStatusMessage({ text: 'User released manual control override.' });
      await fetchStatus();
    } catch (e: any) {
      setStatusMessage({ text: `Failed: ${e}`, isError: true });
    }
  };

  const handleEmergencyStop = async () => {
    try {
      await invokeCommand('trigger_emergency_stop');
      setStatusMessage({ text: 'EMERGENCY STOP TRIGGERED: Desktop control revoked.', isError: true });
      await fetchStatus();
    } catch (e: any) {
      setStatusMessage({ text: `Error: ${e}`, isError: true });
    }
  };

  const handleResetEmergencyStop = async () => {
    try {
      await invokeCommand('reset_emergency_stop');
      setStatusMessage({ text: 'Emergency stop cleared.' });
      await fetchStatus();
    } catch (e: any) {
      setStatusMessage({ text: `Error: ${e}`, isError: true });
    }
  };

  const handleSwitchWindow = async (title: string) => {
    setLoadingAction(`switch-${title}`);
    try {
      const ok = await invokeCommand<boolean>('switch_desktop_window', { title });
      if (ok) {
        setStatusMessage({ text: `Switched focus to window: "${title}"` });
      } else {
        setStatusMessage({ text: `Window "${title}" not found or activation returned false`, isError: true });
      }
      await fetchStatus();
    } catch (e: any) {
      setStatusMessage({ text: `Switch window error: ${e}`, isError: true });
    } finally {
      setLoadingAction(null);
    }
  };

  const handleCaptureScreenshot = async () => {
    setLoadingAction('capture');
    try {
      const b64 = await invokeCommand<string>('take_desktop_screenshot');
      setScreenshotData(b64);
      setStatusMessage({ text: 'Desktop screen captured successfully.' });
    } catch (e: any) {
      setStatusMessage({ text: `Screenshot error: ${e}`, isError: true });
    } finally {
      setLoadingAction(null);
    }
  };

  return (
    <div className="flex-1 flex flex-col bg-slate-950 text-slate-100 overflow-y-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between pb-4 border-b border-slate-800">
        <div>
          <h1 className="text-xl font-bold flex items-center gap-2.5">
            <Monitor className="w-6 h-6 text-indigo-400" />
            Windows Computer Control
          </h1>
          <p className="text-sm text-slate-400 mt-1">
            Windows-MCP UI Automation, Desktop Ownership Leases, Window Management & Verified Input
          </p>
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={fetchStatus}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 transition"
          >
            <RefreshCw className="w-3.5 h-3.5" />
            Refresh
          </button>

          {ownership?.is_emergency_stopped ? (
            <button
              onClick={handleResetEmergencyStop}
              className="flex items-center gap-1.5 px-4 py-2 text-xs font-semibold rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white shadow-lg transition"
            >
              Reset Emergency Stop
            </button>
          ) : (
            <button
              onClick={handleEmergencyStop}
              className="flex items-center gap-1.5 px-4 py-2 text-xs font-semibold rounded-lg bg-rose-600 hover:bg-rose-500 text-white shadow-lg shadow-rose-600/20 transition"
            >
              <ShieldAlert className="w-4 h-4" />
              Emergency Stop
            </button>
          )}
        </div>
      </div>

      {/* Status Alert Banner */}
      {statusMessage && (
        <div
          className={`px-4 py-2.5 rounded-lg text-sm flex items-center justify-between ${
            statusMessage.isError
              ? 'bg-rose-500/10 border border-rose-500/30 text-rose-300'
              : 'bg-emerald-500/10 border border-emerald-500/30 text-emerald-300'
          }`}
        >
          <div className="flex items-center gap-2">
            {statusMessage.isError ? (
              <XCircle className="w-4 h-4 text-rose-400 flex-shrink-0" />
            ) : (
              <CheckCircle2 className="w-4 h-4 text-emerald-400 flex-shrink-0" />
            )}
            <span>{statusMessage.text}</span>
          </div>
          <button onClick={() => setStatusMessage(null)} className="text-xs opacity-70 hover:opacity-100">
            Dismiss
          </button>
        </div>
      )}

      {/* Grid: Ownership Card & Lock State */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Ownership Card */}
        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-medium text-slate-400">Desktop Ownership Controller</span>
            <span
              className={`px-2 py-0.5 text-xs font-bold rounded-full ${
                ownership?.current_controller === 'Agent'
                  ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40'
                  : ownership?.current_controller === 'User'
                  ? 'bg-blue-500/20 text-blue-300 border border-blue-500/40'
                  : 'bg-slate-800 text-slate-400'
              }`}
            >
              {ownership?.current_controller || 'None'}
            </span>
          </div>

          <div className="my-2">
            <div className="text-2xl font-bold text-slate-100 flex items-center gap-2">
              <Clock className="w-5 h-5 text-indigo-400" />
              {ownership?.remaining_secs ?? 0}s
            </div>
            <p className="text-xs text-slate-500 mt-1">Remaining AI input control lease duration</p>
          </div>

          <div className="flex gap-2 pt-2 border-t border-slate-800/80">
            <button
              onClick={handleAcquireLease}
              disabled={loadingAction === 'lease' || ownership?.current_controller === 'User'}
              className="flex-1 py-1.5 text-xs font-medium rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white disabled:opacity-50 transition"
            >
              Acquire Lease
            </button>
            {ownership?.current_controller === 'User' ? (
              <button
                onClick={handleReleaseUserControl}
                className="flex-1 py-1.5 text-xs font-medium rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 transition"
              >
                Release User
              </button>
            ) : (
              <button
                onClick={handleUserTakeover}
                className="flex-1 py-1.5 text-xs font-medium rounded-lg bg-amber-600/80 hover:bg-amber-600 text-white flex items-center justify-center gap-1 transition"
              >
                <Hand className="w-3.5 h-3.5" />
                Takeover
              </button>
            )}
          </div>
        </div>

        {/* Lock State Card */}
        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-medium text-slate-400">Desktop Session Lock</span>
            {lockStatus === 'Locked' ? (
              <span className="px-2 py-0.5 text-xs font-bold rounded-full bg-rose-500/20 text-rose-300 border border-rose-500/40 flex items-center gap-1">
                <Lock className="w-3 h-3" /> Locked
              </span>
            ) : (
              <span className="px-2 py-0.5 text-xs font-bold rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 flex items-center gap-1">
                <Unlock className="w-3 h-3" /> Unlocked
              </span>
            )}
          </div>

          <div className="my-2">
            <div className="text-lg font-semibold text-slate-200">
              {lockStatus === 'Locked' ? 'Workstation Locked' : 'Interactive Session'}
            </div>
            <p className="text-xs text-slate-500 mt-1">
              {lockStatus === 'Locked'
                ? 'GUI automation is safely suspended while workstation is locked.'
                : 'Desktop is available for UI Automation and window manipulation.'}
            </p>
          </div>

          <div className="text-xs text-slate-400 pt-2 border-t border-slate-800/80 flex items-center gap-1.5">
            <Zap className="w-3.5 h-3.5 text-amber-400" />
            <span>Non-GUI tasks continue unaffected in background</span>
          </div>
        </div>

        {/* Token Savings Benchmark */}
        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-medium text-slate-400">Token Optimization Metric</span>
            <span className="px-2 py-0.5 text-xs font-bold rounded-full bg-indigo-500/20 text-indigo-300 border border-indigo-500/40">
              UIA First
            </span>
          </div>

          <div className="my-2 space-y-1">
            <div className="flex justify-between text-xs">
              <span className="text-slate-400">UIA Accessibility Tree:</span>
              <span className="font-mono text-emerald-400">~180 tokens</span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-slate-400">Full Screen Screenshot:</span>
              <span className="font-mono text-rose-400">~2,400 tokens</span>
            </div>
            <div className="flex justify-between text-xs font-bold pt-1 border-t border-slate-800">
              <span className="text-slate-300">Efficiency Gain:</span>
              <span className="text-indigo-400">92.5% Token Savings</span>
            </div>
          </div>

          <div className="text-xs text-slate-500 pt-2 border-t border-slate-800/80">
            Prefers accessibility tree coordinates over visual pixels.
          </div>
        </div>
      </div>

      {/* Open Windows Section */}
      <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-4 space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-semibold flex items-center gap-2 text-slate-200">
            <AppWindow className="w-4 h-4 text-indigo-400" />
            Open Top-Level Windows ({windows.length})
          </h2>
          <span className="text-xs text-slate-500">Click Bring to Focus to switch active window</span>
        </div>

        {windows.length === 0 ? (
          <div className="text-xs text-slate-500 py-4 text-center">No open application windows discovered.</div>
        ) : (
          <div className="space-y-1.5 max-h-56 overflow-y-auto pr-1">
            {windows.map((w) => (
              <div
                key={w.id}
                className="flex items-center justify-between px-3 py-2 rounded-lg bg-slate-950/60 border border-slate-800 hover:border-slate-700 transition"
              >
                <div className="flex items-center gap-3 overflow-hidden">
                  <span className="w-2 h-2 rounded-full bg-emerald-400 flex-shrink-0" />
                  <div className="overflow-hidden">
                    <div className="text-xs font-medium text-slate-200 truncate">{w.title}</div>
                    <div className="text-[10px] text-slate-500 font-mono">
                      Process: {w.process_name} (PID: {w.id})
                    </div>
                  </div>
                </div>

                <button
                  onClick={() => handleSwitchWindow(w.title)}
                  disabled={loadingAction === `switch-${w.title}`}
                  className="px-2.5 py-1 text-xs font-medium rounded bg-indigo-600/30 hover:bg-indigo-600/50 text-indigo-300 border border-indigo-500/30 transition disabled:opacity-50"
                >
                  Bring to Focus
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Live Screen Preview Section */}
      <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-4 space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-sm font-semibold flex items-center gap-2 text-slate-200">
              <Eye className="w-4 h-4 text-indigo-400" />
              Live Screen Preview (Privacy Conscious)
            </h2>
            <p className="text-xs text-slate-500 mt-0.5">
              Only captures on-demand to protect privacy and conserve local resources.
            </p>
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={handleCaptureScreenshot}
              disabled={loadingAction === 'capture'}
              className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white transition disabled:opacity-50"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${loadingAction === 'capture' ? 'animate-spin' : ''}`} />
              Capture Now
            </button>

            <button
              onClick={() => setPreviewEnabled(!previewEnabled)}
              className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 transition"
            >
              {previewEnabled ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
              {previewEnabled ? 'Hide Canvas' : 'Show Canvas'}
            </button>
          </div>
        </div>

        {previewEnabled && (
          <div className="border border-slate-800 rounded-lg p-2 bg-slate-950/80 flex items-center justify-center min-h-[220px]">
            {screenshotData ? (
              <img
                src={`data:image/png;base64,${screenshotData}`}
                alt="Desktop Live Preview"
                className="max-h-[380px] rounded border border-slate-800 object-contain shadow-md"
              />
            ) : (
              <div className="text-xs text-slate-500 flex flex-col items-center gap-2">
                <Monitor className="w-8 h-8 text-slate-600" />
                <span>No preview captured yet. Click "Capture Now" to inspect current display.</span>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
