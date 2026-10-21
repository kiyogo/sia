import React, { useState, useEffect } from 'react';
import { PermissionMode, PrerequisiteStatus, RuntimeConfig } from '../types';
import { invokeCommand } from '../lib/tauriBridge';
import {
  Settings as SettingsIcon,
  Save,
  CheckCircle2,
  AlertCircle,
  Eye,
  EyeOff,
  Radio,
  Layers,
  Shield,
  Sliders,
  Languages,
} from 'lucide-react';

interface SettingsProps {
  language?: 'vi' | 'en';
  onLanguageChange?: (lang: 'vi' | 'en') => void;
}

export const Settings: React.FC<SettingsProps> = ({
  language: propLanguage,
  onLanguageChange: propOnLanguageChange,
}) => {
  const [localLanguage, setLocalLanguage] = useState<'vi' | 'en'>(() => {
    return (localStorage.getItem('app_language') as 'vi' | 'en') || 'vi';
  });

  const language = propLanguage || localLanguage;
  const setLanguage = (lang: 'vi' | 'en') => {
    localStorage.setItem('app_language', lang);
    setLocalLanguage(lang);
    if (propOnLanguageChange) {
      propOnLanguageChange(lang);
    }
  };

  const isVi = language === 'vi';

  const [config, setConfig] = useState<RuntimeConfig | null>(null);
  const [showApiKey, setShowApiKey] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [isTesting, setIsTesting] = useState(false);

  const [prereqs, setPrereqs] = useState<PrerequisiteStatus[]>([]);
  const [mcpConnected, setMcpConnected] = useState(false);
  const [mcpConnecting, setMcpConnecting] = useState(false);
  const [mcpMessage, setMcpMessage] = useState<string | null>(null);

  useEffect(() => {
    loadSettings();
    checkPrereqs();
  }, []);

  const loadSettings = async () => {
    try {
      const cfg = await invokeCommand<RuntimeConfig>('get_runtime_config');
      setConfig(cfg);
    } catch (e) {
      console.error('Failed to load settings', e);
    }
  };

  const checkPrereqs = async () => {
    try {
      const p = await invokeCommand<PrerequisiteStatus[]>('get_system_prerequisites');
      setPrereqs(p);
      const [connected] = await invokeCommand<[boolean, boolean, string]>('check_mcp_status');
      setMcpConnected(connected);
    } catch (e) {
      console.error('Failed to check prerequisites', e);
    }
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!config) return;
    setIsSaving(true);
    try {
      await invokeCommand('save_runtime_config', { config });
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 2000);
    } catch (e) {
      console.error('Failed to save settings', e);
    } finally {
      setIsSaving(false);
    }
  };

  const handleTestConnection = async () => {
    setIsTesting(true);
    setTestResult(null);
    try {
      const ok = await invokeCommand<boolean>('test_provider_connection');
      if (ok) {
        setTestResult(
          isVi
            ? 'Thành công: Đã kết nối và xác thực tới nhà cung cấp mô hình AI.'
            : 'Success: Connection to AI provider verified.'
        );
      } else {
        setTestResult(
          isVi
            ? 'Thông báo: Đã kết nối tới máy chủ nhưng xác thực không thành công (mã khác 200). Vui lòng kiểm tra lại API Key.'
            : 'Notice: Connected to endpoint but authentication returned non-200. Check API Key.'
        );
      }
    } catch (err: any) {
      setTestResult(
        isVi
          ? `Thất bại: Không thể kết nối (${err?.toString() || 'Lỗi không xác định'})`
          : `Failed: ${err?.toString() || 'Connection failed'}`
      );
    } finally {
      setIsTesting(false);
    }
  };

  const handleToggleMcp = async () => {
    setMcpConnecting(true);
    setMcpMessage(null);
    try {
      if (mcpConnected) {
        await invokeCommand('disconnect_mcp');
        setMcpConnected(false);
        setMcpMessage(isVi ? 'Đã ngắt kết nối Windows-MCP.' : 'Windows-MCP disconnected.');
      } else {
        const tools = await invokeCommand<any[]>('connect_mcp');
        setMcpConnected(true);
        setMcpMessage(
          isVi
            ? `Đã kết nối thành công Windows-MCP với ${tools.length} công cụ điều khiển hệ thống.`
            : `Windows-MCP connected with ${tools.length} desktop tools.`
        );
      }
    } catch (err: any) {
      setMcpMessage(
        isVi
          ? `Lỗi Windows-MCP: ${err?.toString()}`
          : `Windows-MCP Error: ${err?.toString()}`
      );
    } finally {
      setMcpConnecting(false);
    }
  };

  if (!config) {
    return (
      <div className="p-8 text-xs text-slate-500">
        {isVi ? 'Đang tải cài đặt hệ thống...' : 'Loading settings...'}
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto p-8 space-y-8 bg-slate-950">
      {/* Top Header */}
      <div className="flex items-center justify-between pb-4 border-b border-slate-800">
        <div>
          <h2 className="text-xl font-bold text-slate-100 flex items-center gap-2">
            <SettingsIcon className="w-5 h-5 text-indigo-400" />
            <span>{isVi ? 'Cài Đặt Không Gian Làm Việc' : 'Workspace Settings'}</span>
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            {isVi
              ? 'Cấu hình nhà cung cấp mô hình AI, ngân sách token, phân quyền an toàn và tích hợp Windows-MCP.'
              : 'Configure AI provider endpoint, token budgets, safety permissions, and Windows-MCP.'}
          </p>
        </div>

        <button
          onClick={handleSave}
          disabled={isSaving}
          className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-xs flex items-center gap-1.5 transition shadow"
        >
          <Save className="w-3.5 h-3.5" />
          <span>
            {isSaving
              ? isVi
                ? 'Đang lưu...'
                : 'Saving...'
              : saveSuccess
              ? isVi
                ? 'Đã lưu thành công!'
                : 'Saved!'
              : isVi
              ? 'Lưu thay đổi'
              : 'Save Changes'}
          </span>
        </button>
      </div>

      <form onSubmit={handleSave} className="space-y-6 max-w-4xl">
        {/* Section 0: Ngôn ngữ / Language Selector */}
        <div className="p-5 rounded-xl bg-slate-900/60 border border-slate-800 space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-slate-200 flex items-center gap-2">
              <Languages className="w-4 h-4 text-sky-400" />
              <span>{isVi ? 'Ngôn Ngữ Giao Diện' : 'Interface Language'}</span>
            </h3>
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={() => setLanguage('vi')}
                className={`px-3 py-1.5 rounded-lg text-xs font-medium transition flex items-center gap-1.5 ${
                  isVi
                    ? 'bg-indigo-600 text-white shadow'
                    : 'bg-slate-800 text-slate-400 hover:text-slate-200 border border-slate-700'
                }`}
              >
                <span>🇻🇳</span>
                <span>Tiếng Việt</span>
              </button>
              <button
                type="button"
                onClick={() => setLanguage('en')}
                className={`px-3 py-1.5 rounded-lg text-xs font-medium transition flex items-center gap-1.5 ${
                  !isVi
                    ? 'bg-indigo-600 text-white shadow'
                    : 'bg-slate-800 text-slate-400 hover:text-slate-200 border border-slate-700'
                }`}
              >
                <span>🇺🇸</span>
                <span>English</span>
              </button>
            </div>
          </div>
          <p className="text-[11px] text-slate-400">
            {isVi
              ? 'Lựa chọn hiển thị giao diện tiếng Việt hoặc tiếng Anh cho toàn bộ cài đặt và bảng điều khiển.'
              : 'Switch interface language between Vietnamese and English for workspace settings and dashboards.'}
          </p>
        </div>

        {/* Section 1: AI Provider */}
        <div className="p-5 rounded-xl bg-slate-900/60 border border-slate-800 space-y-4">
          <h3 className="text-sm font-semibold text-slate-200 flex items-center gap-2">
            <Radio className="w-4 h-4 text-emerald-400" />
            <span>{isVi ? 'Nhà Cung Cấp Mô Hình AI' : 'AI Model Provider'}</span>
          </h3>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1">
                {isVi ? 'Địa chỉ Base URL của nhà cung cấp' : 'Provider Base URL'}
              </label>
              <input
                type="text"
                value={config.provider_url}
                onChange={(e) => setConfig({ ...config, provider_url: e.target.value })}
                placeholder="https://api.openai.com/v1 hoặc http://localhost:11434/v1"
                className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-xs text-slate-200 focus:outline-none focus:border-indigo-500 font-mono"
              />
            </div>

            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1">
                {isVi ? 'Tên mô hình (Model Name)' : 'Model Name'}
              </label>
              <input
                type="text"
                value={config.model_name}
                onChange={(e) => setConfig({ ...config, model_name: e.target.value })}
                placeholder="gpt-4o, deepseek-chat, llama3.2, claude-3-5..."
                className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-xs text-slate-200 focus:outline-none focus:border-indigo-500 font-mono"
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-400 mb-1">
              {isVi ? 'Khóa bảo mật API Key' : 'API Key'}
            </label>
            <div className="relative">
              <input
                type={showApiKey ? 'text' : 'password'}
                value={config.api_key}
                onChange={(e) => setConfig({ ...config, api_key: e.target.value })}
                placeholder="sk-..."
                className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-xs text-slate-200 focus:outline-none focus:border-indigo-500 pr-10 font-mono"
              />
              <button
                type="button"
                onClick={() => setShowApiKey(!showApiKey)}
                className="absolute right-3 top-2.5 text-slate-400 hover:text-slate-200"
              >
                {showApiKey ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
              </button>
            </div>
            <p className="text-[10px] text-slate-400 mt-1">
              {isVi
                ? 'Khóa API được lưu an toàn trong cơ sở dữ liệu SQLite máy bạn, không bao giờ gửi ra ngoài trái phép.'
                : 'Keys are stored securely in local database and are never logged or exported.'}
            </p>
          </div>

          <div className="flex items-center gap-3 pt-2">
            <button
              type="button"
              onClick={handleTestConnection}
              disabled={isTesting}
              className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium border border-slate-700 transition"
            >
              {isTesting
                ? isVi
                  ? 'Đang kiểm tra kết nối...'
                  : 'Testing connection...'
                : isVi
                ? 'Kiểm tra kết nối'
                : 'Test Connection'}
            </button>
            {testResult && (
              <span
                className={`text-xs ${
                  testResult.includes('Thành công') || testResult.startsWith('Success')
                    ? 'text-emerald-400'
                    : 'text-amber-400'
                }`}
              >
                {testResult}
              </span>
            )}
          </div>
        </div>

        {/* Section 2: Security & Permissions */}
        <div className="p-5 rounded-xl bg-slate-900/60 border border-slate-800 space-y-4">
          <h3 className="text-sm font-semibold text-slate-200 flex items-center gap-2">
            <Shield className="w-4 h-4 text-amber-400" />
            <span>{isVi ? 'Chế Độ Bảo Mật & Sandbox' : 'Security & Sandbox Mode'}</span>
          </h3>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            {[
              {
                mode: 'Strict',
                label: isVi ? 'Bảo Mật Nghiêm Ngặt' : 'Strict Security',
                desc: isVi
                  ? 'Yêu cầu người dùng duyệt thủ công cho mọi thao tác ghi file, thực thi lệnh shell và điều khiển máy tính.'
                  : 'Requires human approval for all file writes, shell commands, and desktop control.',
              },
              {
                mode: 'WorkspaceAuto',
                label: isVi ? 'Tự Động Trong Workspace (Khuyên dùng)' : 'Workspace Scoped (Recommended)',
                desc: isVi
                  ? 'Tự động cho phép đọc/ghi an toàn trong thư mục workspace. Các lệnh nhạy cảm ngoài workspace vẫn yêu cầu duyệt.'
                  : 'Auto-approves safe reads/writes in workspace. Dangerous ops require approval.',
              },
              {
                mode: 'FullAuto',
                label: isVi ? 'Tự Động Toàn Phần (Autonomous)' : 'Autonomous Mode',
                desc: isVi
                  ? 'Tự động hóa hoàn toàn các thao tác workspace và shell an toàn. Các hành động phá hủy hệ thống bị chặn tuyệt đối.'
                  : 'Auto-approves workspace edits and safe shell execution. Destructive actions blocked.',
              },
            ].map((p) => {
              const isSelected = config.permission_mode === p.mode;
              return (
                <div
                  key={p.mode}
                  onClick={() => setConfig({ ...config, permission_mode: p.mode as PermissionMode })}
                  className={`p-3.5 rounded-xl border cursor-pointer transition ${
                    isSelected
                      ? 'bg-indigo-600/10 border-indigo-500/50 text-indigo-300'
                      : 'bg-slate-950 border-slate-800 text-slate-400 hover:text-slate-300'
                  }`}
                >
                  <div className="font-semibold text-xs text-slate-200 mb-1">{p.label}</div>
                  <div className="text-[11px] leading-relaxed">{p.desc}</div>
                </div>
              );
            })}
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-400 mb-1">
              {isVi ? 'Đường dẫn thư mục Workspace hoạt động' : 'Active Workspace Root Path'}
            </label>
            <input
              type="text"
              value={config.workspace_path}
              onChange={(e) => setConfig({ ...config, workspace_path: e.target.value })}
              className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
            />
          </div>
        </div>

        {/* Section 3: Token Optimization Engine */}
        <div className="p-5 rounded-xl bg-slate-900/60 border border-slate-800 space-y-4">
          <h3 className="text-sm font-semibold text-slate-200 flex items-center gap-2">
            <Sliders className="w-4 h-4 text-indigo-400" />
            <span>{isVi ? 'Tối Ưu Hóa Token & Ngân Sách Ngữ Cảnh' : 'Token Optimization & Context Budget'}</span>
          </h3>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1">
                {isVi ? 'Số vòng lặp tối đa của Agent' : 'Max Agent Iteration Loops'}
              </label>
              <input
                type="number"
                min={1}
                max={100}
                value={config.max_loops}
                onChange={(e) => setConfig({ ...config, max_loops: Number(e.target.value) })}
                className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
              />
              <p className="text-[10px] text-slate-400 mt-1">
                {isVi
                  ? 'Giới hạn dừng cứng nhằm ngăn chặn vòng lặp vô hạn hoặc tiêu tốn token bất thường.'
                  : 'Hard stop limit preventing runaway loops or infinite token drain.'}
              </p>
            </div>

            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1">
                {isVi ? 'Số dòng kết quả công cụ tối đa (Cắt ngắn)' : 'Max Tool Output Lines (Truncation)'}
              </label>
              <input
                type="number"
                min={20}
                max={1000}
                value={config.budget_config.max_tool_output_lines}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    budget_config: {
                      ...config.budget_config,
                      max_tool_output_lines: Number(e.target.value),
                    },
                  })
                }
                className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
              />
              <p className="text-[10px] text-slate-400 mt-1">
                {isVi
                  ? 'Kết quả vượt quá số dòng này sẽ được tự động cắt bớt kèm chỉ dẫn phân trang để tối ưu chi phí.'
                  : 'Outputs longer than this are truncated with offset pagination tips.'}
              </p>
            </div>
          </div>
        </div>

        {/* Section 4: Windows-MCP Control Integration */}
        <div className="p-5 rounded-xl bg-slate-900/60 border border-slate-800 space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-slate-200 flex items-center gap-2">
              <Layers className="w-4 h-4 text-blue-400" />
              <span>{isVi ? 'Điều Khiển Máy Tính Cục Bộ Windows-MCP' : 'Windows-MCP Native Desktop Control'}</span>
            </h3>
            <button
              type="button"
              onClick={handleToggleMcp}
              disabled={mcpConnecting}
              className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition ${
                mcpConnected
                  ? 'bg-rose-600/80 hover:bg-rose-600 text-white'
                  : 'bg-blue-600 hover:bg-blue-500 text-white'
              }`}
            >
              {mcpConnecting
                ? isVi
                  ? 'Đang xử lý...'
                  : 'Processing...'
                : mcpConnected
                ? isVi
                  ? 'Ngắt kết nối Windows-MCP'
                  : 'Disconnect Windows-MCP'
                : isVi
                ? 'Kết nối Windows-MCP'
                : 'Connect Windows-MCP'}
            </button>
          </div>

          {/* Prerequisite status checklist */}
          <div className="space-y-2 pt-2">
            <span className="text-xs font-medium text-slate-400">
              {isVi ? 'Chẩn đoán điều kiện tiên quyết hệ thống:' : 'System Prerequisites Diagnostic:'}
            </span>
            {prereqs.map((pr, idx) => (
              <div
                key={idx}
                className="flex items-center justify-between p-2.5 rounded-lg bg-slate-950 border border-slate-800 text-xs"
              >
                <div className="flex items-center gap-2">
                  {pr.installed ? (
                    <CheckCircle2 className="w-4 h-4 text-emerald-400 flex-shrink-0" />
                  ) : (
                    <AlertCircle className="w-4 h-4 text-amber-400 flex-shrink-0" />
                  )}
                  <span className="font-semibold text-slate-200">{pr.name}</span>
                  <span className="text-slate-400 font-mono text-[11px]">
                    {pr.version || `(${pr.required_version})`}
                  </span>
                </div>
                <span className="text-[11px] text-slate-400 max-w-sm truncate text-right">
                  {pr.install_hint}
                </span>
              </div>
            ))}
          </div>

          {mcpMessage && (
            <div className="p-3 bg-slate-950 rounded-lg border border-slate-800 text-xs text-slate-300">
              {mcpMessage}
            </div>
          )}
        </div>
      </form>
    </div>
  );
};
