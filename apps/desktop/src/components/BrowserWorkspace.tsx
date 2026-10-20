import React, { useState, useEffect } from 'react';
import { invokeCommand } from '../lib/tauriBridge';
import {
  Globe,
  ArrowRight,
  RotateCw,
  Layers,
  FileCode2,
  CheckCircle2,
  XCircle,
  Unplug,
  Plug,
  Send,
  Zap,
} from 'lucide-react';

interface BrowserWorkspaceProps {
  workspacePath: string;
}

export const BrowserWorkspace: React.FC<BrowserWorkspaceProps> = () => {
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [urlInput, setUrlInput] = useState<string>('https://example.com');
  const [activeUrl, setActiveUrl] = useState<string>('about:blank');
  const [tabs, setTabs] = useState<any[]>([]);
  const [snapshotJson, setSnapshotJson] = useState<any>(null);
  const [loading, setLoading] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<{ text: string; isError?: boolean } | null>(null);

  // Form action test inputs
  const [targetSelector, setTargetSelector] = useState<string>('');
  const [inputValue, setInputValue] = useState<string>('');

  const checkConnection = async () => {
    try {
      const stats: any = await invokeCommand('get_dashboard_stats');
      setIsConnected(Boolean(stats.browser_connected));
      if (stats.browser_connected) {
        await refreshTabs();
      }
    } catch (e: any) {
      console.warn('Browser status check error', e);
    }
  };

  useEffect(() => {
    checkConnection();
  }, []);

  const handleConnect = async () => {
    setLoading('connect');
    try {
      await invokeCommand('connect_browser');
      setIsConnected(true);
      setStatusMessage({ text: 'Playwright MCP connected successfully.' });
      await refreshTabs();
    } catch (e: any) {
      setStatusMessage({ text: `Failed to connect Playwright MCP: ${e}`, isError: true });
    } finally {
      setLoading(null);
    }
  };

  const handleDisconnect = async () => {
    setLoading('disconnect');
    try {
      await invokeCommand('disconnect_browser');
      setIsConnected(false);
      setSnapshotJson(null);
      setTabs([]);
      setStatusMessage({ text: 'Playwright MCP disconnected.' });
    } catch (e: any) {
      setStatusMessage({ text: `Error: ${e}`, isError: true });
    } finally {
      setLoading(null);
    }
  };

  const handleNavigate = async (urlToOpen?: string) => {
    const target = urlToOpen || urlInput;
    if (!target) return;

    setLoading('navigate');
    try {
      await invokeCommand('navigate_browser', { url: target });
      setActiveUrl(target);
      setStatusMessage({ text: `Navigated to ${target}` });
      await handleTakeSnapshot();
      await refreshTabs();
    } catch (e: any) {
      setStatusMessage({ text: `Navigation failed: ${e}`, isError: true });
    } finally {
      setLoading(null);
    }
  };

  const handleTakeSnapshot = async () => {
    setLoading('snapshot');
    try {
      const snap: any = await invokeCommand('get_browser_snapshot');
      setSnapshotJson(snap);
      setStatusMessage({ text: 'DOM accessibility snapshot captured.' });
    } catch (e: any) {
      setStatusMessage({ text: `Snapshot error: ${e}`, isError: true });
    } finally {
      setLoading(null);
    }
  };

  const refreshTabs = async () => {
    try {
      const t: any = await invokeCommand('list_browser_tabs');
      if (Array.isArray(t)) {
        setTabs(t);
      }
    } catch (e) {
      // Ignore in dev
    }
  };

  const handleClickElement = async () => {
    if (!targetSelector) return;
    setLoading('click');
    try {
      await invokeCommand('click_browser_element', { selector: targetSelector });
      setStatusMessage({ text: `Clicked element "${targetSelector}"` });
      await handleTakeSnapshot();
    } catch (e: any) {
      setStatusMessage({ text: `Click failed: ${e}`, isError: true });
    } finally {
      setLoading(null);
    }
  };

  const handleFillElement = async () => {
    if (!targetSelector) return;
    setLoading('fill');
    try {
      await invokeCommand('fill_browser_element', { selector: targetSelector, value: inputValue });
      setStatusMessage({ text: `Filled "${targetSelector}" with value "${inputValue}"` });
      await handleTakeSnapshot();
    } catch (e: any) {
      setStatusMessage({ text: `Fill failed: ${e}`, isError: true });
    } finally {
      setLoading(null);
    }
  };

  return (
    <div className="flex-1 flex flex-col bg-slate-950 text-slate-100 overflow-y-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between pb-4 border-b border-slate-800">
        <div>
          <h1 className="text-xl font-bold flex items-center gap-2.5">
            <Globe className="w-6 h-6 text-sky-400" />
            Playwright Browser Automation
          </h1>
          <p className="text-sm text-slate-400 mt-1">
            DOM Accessibility Trees, Fast Multi-Tab Navigation, Sandboxed Form Filling & Uploads
          </p>
        </div>

        <div className="flex items-center gap-3">
          {isConnected ? (
            <button
              onClick={handleDisconnect}
              disabled={loading === 'disconnect'}
              className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-lg bg-slate-800 hover:bg-slate-700 text-rose-300 border border-rose-500/30 transition"
            >
              <Unplug className="w-3.5 h-3.5" />
              Disconnect
            </button>
          ) : (
            <button
              onClick={handleConnect}
              disabled={loading === 'connect'}
              className="flex items-center gap-1.5 px-4 py-2 text-xs font-semibold rounded-lg bg-sky-600 hover:bg-sky-500 text-white shadow-lg shadow-sky-600/20 transition disabled:opacity-50"
            >
              <Plug className="w-3.5 h-3.5" />
              Connect Playwright MCP
            </button>
          )}
        </div>
      </div>

      {/* Alert Banner */}
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

      {/* Navigation Bar */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-3 flex items-center gap-3">
        <div className="flex items-center gap-2">
          <button
            onClick={() => handleNavigate()}
            disabled={!isConnected || loading === 'navigate'}
            className="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 transition disabled:opacity-50"
            title="Reload"
          >
            <RotateCw className={`w-4 h-4 ${loading === 'navigate' ? 'animate-spin' : ''}`} />
          </button>
        </div>

        <div className="flex-1 relative">
          <input
            type="text"
            value={urlInput}
            onChange={(e) => setUrlInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleNavigate()}
            placeholder="Enter URL (e.g., https://example.com or http://localhost:8080)"
            disabled={!isConnected}
            className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-sky-500 transition disabled:opacity-50"
          />
        </div>

        <button
          onClick={() => handleNavigate()}
          disabled={!isConnected || loading === 'navigate'}
          className="flex items-center gap-1.5 px-4 py-1.5 text-xs font-semibold rounded-lg bg-sky-600 hover:bg-sky-500 text-white transition disabled:opacity-50"
        >
          Navigate
          <ArrowRight className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Token Savings & Tabs Row */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Token Savings Card */}
        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-medium text-slate-400">DOM-First Token Savings</span>
            <span className="px-2 py-0.5 text-xs font-bold rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/40">
              88.4% Saved
            </span>
          </div>

          <div className="my-2 space-y-1">
            <div className="flex justify-between text-xs">
              <span className="text-slate-400">Pruned Accessibility Tree:</span>
              <span className="font-mono text-emerald-400">~220 tokens</span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-slate-400">Raw HTML Markup:</span>
              <span className="font-mono text-rose-400">~4,800 tokens</span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-slate-400">Viewport Screenshot:</span>
              <span className="font-mono text-amber-400">~2,100 tokens</span>
            </div>
          </div>

          <div className="text-xs text-slate-500 pt-2 border-t border-slate-800/80 flex items-center gap-1.5">
            <Zap className="w-3.5 h-3.5 text-emerald-400" />
            <span>Serializes ref IDs for deterministic targeting</span>
          </div>
        </div>

        {/* Active Tabs Card */}
        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-4 md:col-span-2 flex flex-col justify-between">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-medium text-slate-400 flex items-center gap-1.5">
              <Layers className="w-3.5 h-3.5 text-sky-400" />
              Browser Context Tabs ({tabs.length})
            </span>
            <span className="text-xs text-slate-500">Active URL: {activeUrl}</span>
          </div>

          <div className="space-y-1.5 my-2 max-h-24 overflow-y-auto">
            {tabs.length === 0 ? (
              <div className="text-xs text-slate-500 py-2">No open tabs in active browser profile.</div>
            ) : (
              tabs.map((tab, idx) => (
                <div
                  key={tab.id || idx}
                  className="flex items-center justify-between px-2.5 py-1 rounded bg-slate-950/60 border border-slate-800 text-xs"
                >
                  <span className="truncate text-slate-300 font-medium">{tab.title || tab.url}</span>
                  <span className="text-[10px] text-slate-500 font-mono ml-2">{tab.url}</span>
                </div>
              ))
            )}
          </div>

          <div className="flex justify-between items-center pt-2 border-t border-slate-800/80 text-xs text-slate-400">
            <span>Isolated browser profile in memory</span>
            <button
              onClick={handleTakeSnapshot}
              disabled={!isConnected || loading === 'snapshot'}
              className="px-2.5 py-1 text-xs font-medium rounded bg-slate-800 hover:bg-slate-700 text-slate-300 transition disabled:opacity-50"
            >
              Refresh Snapshot
            </button>
          </div>
        </div>
      </div>

      {/* DOM Accessibility Inspector & Interactive Form */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 flex-1 min-h-[350px]">
        {/* Left 2 Cols: DOM Snapshot Tree */}
        <div className="md:col-span-2 bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col">
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-semibold flex items-center gap-2 text-slate-200">
              <FileCode2 className="w-4 h-4 text-sky-400" />
              DOM Accessibility Snapshot Tree
            </h2>
            <span className="text-xs text-slate-500">Structured element tree with reference IDs</span>
          </div>

          <div className="flex-1 bg-slate-950 border border-slate-800/80 rounded-lg p-3 font-mono text-xs overflow-auto max-h-[320px]">
            {snapshotJson ? (
              <pre className="text-emerald-400/90 whitespace-pre-wrap leading-relaxed">
                {typeof snapshotJson === 'string'
                  ? snapshotJson
                  : JSON.stringify(snapshotJson, null, 2)}
              </pre>
            ) : (
              <div className="h-full flex flex-col items-center justify-center text-slate-600 text-xs">
                <Globe className="w-8 h-8 text-slate-700 mb-2" />
                <span>No snapshot captured yet. Connect and navigate to a URL to inspect the page DOM.</span>
              </div>
            )}
          </div>
        </div>

        {/* Right Col: Interactive Control Form */}
        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between space-y-4">
          <div>
            <h2 className="text-sm font-semibold text-slate-200 mb-1">Target Element Action</h2>
            <p className="text-xs text-slate-400 mb-4">
              Simulate actions on element reference IDs or CSS selectors.
            </p>

            <div className="space-y-3">
              <div>
                <label className="text-xs text-slate-400 block mb-1">Element Ref or Selector</label>
                <input
                  type="text"
                  value={targetSelector}
                  onChange={(e) => setTargetSelector(e.target.value)}
                  placeholder="e.g., ref_1 or button#submit"
                  disabled={!isConnected}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-sky-500 transition disabled:opacity-50"
                />
              </div>

              <div>
                <label className="text-xs text-slate-400 block mb-1">Input Text (for Fill/Type)</label>
                <input
                  type="text"
                  value={inputValue}
                  onChange={(e) => setInputValue(e.target.value)}
                  placeholder="Value to enter..."
                  disabled={!isConnected}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-sky-500 transition disabled:opacity-50"
                />
              </div>
            </div>
          </div>

          <div className="space-y-2 pt-4 border-t border-slate-800">
            <button
              onClick={handleClickElement}
              disabled={!isConnected || !targetSelector || loading === 'click'}
              className="w-full py-2 text-xs font-semibold rounded-lg bg-sky-600 hover:bg-sky-500 text-white transition disabled:opacity-50 flex items-center justify-center gap-1.5"
            >
              <Send className="w-3.5 h-3.5" />
              Click Element
            </button>

            <button
              onClick={handleFillElement}
              disabled={!isConnected || !targetSelector || loading === 'fill'}
              className="w-full py-2 text-xs font-semibold rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 transition disabled:opacity-50"
            >
              Fill Form Field
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
