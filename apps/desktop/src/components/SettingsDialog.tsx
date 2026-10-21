import React, { useState, useEffect } from 'react';
import {
  X,
  Palette,
  Cpu,
  Shield,
  Sun,
  Moon,
  Laptop,
  Check,
  Loader2,
  Eye,
  EyeOff,
  AlertTriangle,
} from 'lucide-react';
import { RuntimeConfig, PermissionMode } from '../types';
import { invokeCommand } from '../lib/tauriBridge';
import { useTheme } from '../context/ThemeContext';

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfigUpdated?: () => void;
}

type SettingsTab = 'appearance' | 'provider' | 'security';

export const SettingsDialog: React.FC<SettingsDialogProps> = ({
  isOpen,
  onClose,
  onConfigUpdated,
}) => {
  const { theme, setTheme, language, setLanguage, t } = useTheme();
  const [activeTab, setActiveTab] = useState<SettingsTab>('appearance');

  // Runtime config state
  const [config, setConfig] = useState<RuntimeConfig | null>(null);
  const [showApiKey, setShowApiKey] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [testStatus, setTestStatus] = useState<'idle' | 'testing' | 'success' | 'failed'>('idle');
  const [testMessage, setTestMessage] = useState<string>('');
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [serviceStatus, setServiceStatus] = useState<string>('Unknown');

  useEffect(() => {
    if (isOpen) {
      loadConfig();
      loadServiceStatus();
      setTestStatus('idle');
      setTestMessage('');
    }
  }, [isOpen]);

  const loadConfig = async () => {
    try {
      const cfg = await invokeCommand<RuntimeConfig>('get_runtime_config');
      setConfig(cfg);
    } catch (e) {
      console.error('Failed to load runtime config', e);
    }
  };

  const loadServiceStatus = async () => {
    try {
      const status = await invokeCommand<string>('get_service_status');
      setServiceStatus(status);
    } catch {
      setServiceStatus('Standby');
    }
  };

  const handleSave = async () => {
    if (!config) return;
    setIsSaving(true);
    setSaveSuccess(false);
    try {
      await invokeCommand('save_runtime_config', { config });
      setSaveSuccess(true);
      if (onConfigUpdated) onConfigUpdated();
      setTimeout(() => setSaveSuccess(false), 2500);
    } catch (e) {
      console.error('Failed to save config', e);
    } finally {
      setIsSaving(false);
    }
  };

  const handleTestConnection = async () => {
    setTestStatus('testing');
    setTestMessage('');
    try {
      // First save active config so test checks the current inputs
      if (config) {
        await invokeCommand('save_runtime_config', { config });
      }
      const isOk = await invokeCommand<boolean>('test_provider_connection');
      if (isOk) {
        setTestStatus('success');
        setTestMessage(t('Kết nối thành công (200 OK)!', 'Successfully connected (200 OK)!'));
      } else {
        setTestStatus('failed');
        setTestMessage(
          t(
            'Không thể kết nối. Vui lòng kiểm tra xem server local hoặc proxy đã bật chưa.',
            'Connection failed. Please ensure the local server or proxy is running.'
          )
        );
      }
    } catch (err: any) {
      setTestStatus('failed');
      setTestMessage(typeof err === 'string' ? err : err?.message || 'Connection error');
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm select-none animate-fade-in">
      <div className="w-full max-w-lg rounded-2xl bg-white dark:bg-[#141417] border border-zinc-200 dark:border-zinc-800 shadow-2xl flex flex-col overflow-hidden text-zinc-900 dark:text-zinc-100">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-3.5 border-b border-zinc-200 dark:border-zinc-800">
          <h3 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">{t('Cài đặt', 'Settings')}</h3>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="flex items-center px-5 border-b border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-900/40">
          <button
            onClick={() => setActiveTab('appearance')}
            className={`flex items-center gap-2 py-2.5 px-3 text-xs font-medium border-b-2 transition-all ${
              activeTab === 'appearance'
                ? 'border-blue-500 text-blue-600 dark:text-blue-400 font-semibold'
                : 'border-transparent text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200'
            }`}
          >
            <Palette className="w-3.5 h-3.5" />
            <span>{t('Giao diện', 'Appearance')}</span>
          </button>

          <button
            onClick={() => setActiveTab('provider')}
            className={`flex items-center gap-2 py-2.5 px-3 text-xs font-medium border-b-2 transition-all ${
              activeTab === 'provider'
                ? 'border-blue-500 text-blue-600 dark:text-blue-400 font-semibold'
                : 'border-transparent text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200'
            }`}
          >
            <Cpu className="w-3.5 h-3.5" />
            <span>{t('Mô hình AI', 'AI Provider')}</span>
          </button>

          <button
            onClick={() => setActiveTab('security')}
            className={`flex items-center gap-2 py-2.5 px-3 text-xs font-medium border-b-2 transition-all ${
              activeTab === 'security'
                ? 'border-blue-500 text-blue-600 dark:text-blue-400 font-semibold'
                : 'border-transparent text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200'
            }`}
          >
            <Shield className="w-3.5 h-3.5" />
            <span>{t('Hệ thống & Quyền', 'System & Security')}</span>
          </button>
        </div>

        {/* Tab Content */}
        <div className="p-5 overflow-y-auto max-h-[60vh] space-y-5 text-xs">
          {/* TAB 1: APPEARANCE */}
          {activeTab === 'appearance' && (
            <div className="space-y-4">
              {/* Theme Selection */}
              <div>
                <label className="block font-medium text-zinc-800 dark:text-zinc-200 mb-2">
                  {t('Chủ đề hiển thị', 'Theme')}
                </label>
                <div className="grid grid-cols-3 gap-2">
                  <button
                    onClick={() => setTheme('light')}
                    className={`flex items-center justify-center gap-2 p-2.5 rounded-xl border text-xs font-medium transition-all ${
                      theme === 'light'
                        ? 'border-blue-500 bg-blue-50 text-blue-600 dark:bg-blue-950/40 dark:text-blue-400 font-semibold shadow-sm'
                        : 'border-zinc-200 dark:border-zinc-800 bg-white dark:bg-[#121215] text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800/50'
                    }`}
                  >
                    <Sun className="w-4 h-4 text-amber-500" />
                    <span>{t('Sáng', 'Light')}</span>
                  </button>

                  <button
                    onClick={() => setTheme('dark')}
                    className={`flex items-center justify-center gap-2 p-2.5 rounded-xl border text-xs font-medium transition-all ${
                      theme === 'dark'
                        ? 'border-blue-500 bg-blue-50 text-blue-600 dark:bg-blue-950/40 dark:text-blue-400 font-semibold shadow-sm'
                        : 'border-zinc-200 dark:border-zinc-800 bg-white dark:bg-[#121215] text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800/50'
                    }`}
                  >
                    <Moon className="w-4 h-4 text-blue-400" />
                    <span>{t('Tối', 'Dark')}</span>
                  </button>

                  <button
                    onClick={() => setTheme('system')}
                    className={`flex items-center justify-center gap-2 p-2.5 rounded-xl border text-xs font-medium transition-all ${
                      theme === 'system'
                        ? 'border-blue-500 bg-blue-50 text-blue-600 dark:bg-blue-950/40 dark:text-blue-400 font-semibold shadow-sm'
                        : 'border-zinc-200 dark:border-zinc-800 bg-white dark:bg-[#121215] text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800/50'
                    }`}
                  >
                    <Laptop className="w-4 h-4 text-zinc-500" />
                    <span>{t('Tự động', 'System')}</span>
                  </button>
                </div>
              </div>

              {/* Language Selection */}
              <div>
                <label className="block font-medium text-zinc-800 dark:text-zinc-200 mb-2">
                  {t('Ngôn ngữ', 'Language')}
                </label>
                <div className="grid grid-cols-2 gap-2">
                  <button
                    onClick={() => setLanguage('vi')}
                    className={`flex items-center justify-center gap-2 p-2.5 rounded-xl border text-xs font-medium transition-all ${
                      language === 'vi'
                        ? 'border-blue-500 bg-blue-50 text-blue-600 dark:bg-blue-950/40 dark:text-blue-400 font-semibold shadow-sm'
                        : 'border-zinc-200 dark:border-zinc-800 bg-white dark:bg-[#121215] text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800/50'
                    }`}
                  >
                    <span>Tiếng Việt 🇻🇳</span>
                  </button>

                  <button
                    onClick={() => setLanguage('en')}
                    className={`flex items-center justify-center gap-2 p-2.5 rounded-xl border text-xs font-medium transition-all ${
                      language === 'en'
                        ? 'border-blue-500 bg-blue-50 text-blue-600 dark:bg-blue-950/40 dark:text-blue-400 font-semibold shadow-sm'
                        : 'border-zinc-200 dark:border-zinc-800 bg-white dark:bg-[#121215] text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800/50'
                    }`}
                  >
                    <span>English 🇺🇸</span>
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* TAB 2: AI PROVIDER */}
          {activeTab === 'provider' && config && (
            <div className="space-y-3.5">
              <div>
                <label className="block font-medium text-zinc-800 dark:text-zinc-200 mb-1">
                  {t('Địa chỉ API Provider', 'Provider Endpoint')}
                </label>
                <input
                  type="text"
                  value={config.provider_url}
                  onChange={(e) => setConfig({ ...config, provider_url: e.target.value })}
                  placeholder="http://localhost:6660/api hoặc https://api.openai.com/v1"
                  className="w-full px-3 py-2 bg-zinc-50 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded-lg text-zinc-900 dark:text-zinc-100 text-xs focus:outline-none focus:border-blue-500 font-mono"
                />
                <div className="text-[11px] text-zinc-400 dark:text-zinc-500 mt-1">
                  {t('Hỗ trợ OpenAI, Gemini proxy, LM Studio, Ollama, v.v.', 'Supports OpenAI, Gemini proxy, LM Studio, Ollama, etc.')}
                </div>
              </div>

              <div>
                <label className="block font-medium text-zinc-800 dark:text-zinc-200 mb-1">
                  {t('Tên mô hình', 'Model Name')}
                </label>
                <input
                  type="text"
                  value={config.model_name}
                  onChange={(e) => setConfig({ ...config, model_name: e.target.value })}
                  placeholder="3.5 Flash, gpt-4o, gemini-1.5-flash..."
                  className="w-full px-3 py-2 bg-zinc-50 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded-lg text-zinc-900 dark:text-zinc-100 text-xs focus:outline-none focus:border-blue-500"
                />
              </div>

              <div>
                <label className="block font-medium text-zinc-800 dark:text-zinc-200 mb-1">
                  {t('Khóa API (API Key)', 'API Key')}
                </label>
                <div className="relative">
                  <input
                    type={showApiKey ? 'text' : 'password'}
                    value={config.api_key}
                    onChange={(e) => setConfig({ ...config, api_key: e.target.value })}
                    placeholder="sk-... (để trống nếu proxy local không yêu cầu)"
                    className="w-full pl-3 pr-9 py-2 bg-zinc-50 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded-lg text-zinc-900 dark:text-zinc-100 text-xs focus:outline-none focus:border-blue-500 font-mono"
                  />
                  <button
                    type="button"
                    onClick={() => setShowApiKey(!showApiKey)}
                    className="absolute right-2.5 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200"
                  >
                    {showApiKey ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                  </button>
                </div>
              </div>

              {/* Test Connection Button & Message */}
              <div className="pt-2 space-y-2">
                <button
                  type="button"
                  onClick={handleTestConnection}
                  disabled={testStatus === 'testing'}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-800 dark:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-700 text-xs transition-colors shadow-sm disabled:opacity-50"
                >
                  {testStatus === 'testing' && <Loader2 className="w-3.5 h-3.5 animate-spin text-blue-500" />}
                  {testStatus === 'success' && <Check className="w-3.5 h-3.5 text-emerald-500" />}
                  {testStatus === 'failed' && <AlertTriangle className="w-3.5 h-3.5 text-red-500" />}
                  <span>
                    {testStatus === 'testing'
                      ? t('Đang kiểm tra kết nối...', 'Testing connection...')
                      : t('Kiểm tra kết nối', 'Test Connection')}
                  </span>
                </button>

                {testMessage && (
                  <div
                    className={`p-2.5 rounded-xl text-[11px] leading-relaxed border ${
                      testStatus === 'success'
                        ? 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950/30 dark:text-emerald-300 dark:border-emerald-800'
                        : 'bg-red-50 text-red-700 border-red-200 dark:bg-red-950/30 dark:text-red-300 dark:border-red-800'
                    }`}
                  >
                    {testMessage}
                  </div>
                )}
              </div>
            </div>
          )}

          {/* TAB 3: SYSTEM & SECURITY */}
          {activeTab === 'security' && config && (
            <div className="space-y-3.5">
              <div>
                <label className="block font-medium text-zinc-800 dark:text-zinc-200 mb-1">
                  {t('Thư mục Workspace làm việc', 'Workspace Directory')}
                </label>
                <input
                  type="text"
                  value={config.workspace_path}
                  onChange={(e) => setConfig({ ...config, workspace_path: e.target.value })}
                  className="w-full px-3 py-2 bg-zinc-50 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded-lg text-zinc-900 dark:text-zinc-100 text-xs focus:outline-none focus:border-blue-500 font-mono"
                />
              </div>

              <div>
                <label className="block font-medium text-zinc-800 dark:text-zinc-200 mb-1">
                  {t('Chế độ phân quyền (Permission Mode)', 'Permission Mode')}
                </label>
                <div className="grid grid-cols-3 gap-2">
                  {(['Strict', 'WorkspaceAuto', 'FullAuto'] as PermissionMode[]).map((mode) => (
                    <button
                      key={mode}
                      onClick={() => setConfig({ ...config, permission_mode: mode })}
                      className={`p-2 rounded-xl border text-xs font-medium text-center transition-all ${
                        config.permission_mode === mode
                          ? 'border-blue-500 bg-blue-50 text-blue-600 dark:bg-blue-950/40 dark:text-blue-400 font-semibold shadow-sm'
                          : 'border-zinc-200 dark:border-zinc-800 bg-white dark:bg-[#121215] text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800/50'
                      }`}
                    >
                      {mode}
                    </button>
                  ))}
                </div>
              </div>

              <div className="p-3 rounded-xl bg-zinc-50 dark:bg-zinc-900/60 border border-zinc-200 dark:border-zinc-800 flex items-center justify-between">
                <div>
                  <div className="font-medium text-zinc-800 dark:text-zinc-200">
                    {t('Dịch vụ chạy ngầm 24/7', '24/7 Background Runtime')}
                  </div>
                  <div className="text-[11px] text-zinc-500 dark:text-zinc-400">
                    {t('Trạng thái Windows Service', 'Windows Service State')}: {serviceStatus}
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-5 py-3 border-t border-zinc-200 dark:border-zinc-800 bg-white dark:bg-[#141417]">
          <div>
            {saveSuccess && (
              <span className="text-xs text-emerald-600 dark:text-emerald-400 flex items-center gap-1 font-medium">
                <Check className="w-3.5 h-3.5" />
                {t('Đã lưu thành công!', 'Saved successfully!')}
              </span>
            )}
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={onClose}
              className="px-3 py-1.5 rounded-lg border border-zinc-200 dark:border-zinc-700 text-zinc-700 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 text-xs transition-colors"
            >
              {t('Đóng', 'Close')}
            </button>

            {activeTab !== 'appearance' && (
              <button
                onClick={handleSave}
                disabled={isSaving}
                className="px-3.5 py-1.5 rounded-lg bg-zinc-900 text-white dark:bg-zinc-100 dark:text-zinc-900 hover:opacity-90 font-medium text-xs transition-opacity shadow-sm disabled:opacity-50"
              >
                {isSaving ? t('Đang lưu...', 'Saving...') : t('Lưu thay đổi', 'Save Changes')}
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
