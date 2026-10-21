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
  Folder,
  RefreshCw,
  AlertTriangle,
} from 'lucide-react';
import { RuntimeConfig, PermissionMode } from '../types';
import { invokeCommand } from '../lib/tauriBridge';
import { useTheme, Theme, Language } from '../context/ThemeContext';

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
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [serviceStatus, setServiceStatus] = useState<string>('Unknown');

  useEffect(() => {
    if (isOpen) {
      loadConfig();
      loadServiceStatus();
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
    try {
      const isOk = await invokeCommand<boolean>('test_provider_connection');
      setTestStatus(isOk ? 'success' : 'failed');
    } catch {
      setTestStatus('failed');
    }
    setTimeout(() => setTestStatus('idle'), 3000);
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm select-none animate-fade-in">
      <div className="w-full max-w-lg rounded-2xl bg-surface border border-border shadow-2xl flex flex-col overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-3.5 border-b border-border bg-surface">
          <h3 className="text-sm font-semibold text-primary">{t('Cài đặt', 'Settings')}</h3>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-muted hover:text-primary hover:bg-muted transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="flex items-center px-5 border-b border-border-subtle bg-muted/20">
          <button
            onClick={() => setActiveTab('appearance')}
            className={`flex items-center gap-2 py-2.5 px-3 text-xs font-medium border-b-2 transition-all ${
              activeTab === 'appearance'
                ? 'border-accent text-primary'
                : 'border-transparent text-secondary hover:text-primary'
            }`}
          >
            <Palette className="w-3.5 h-3.5" />
            <span>{t('Giao diện', 'Appearance')}</span>
          </button>

          <button
            onClick={() => setActiveTab('provider')}
            className={`flex items-center gap-2 py-2.5 px-3 text-xs font-medium border-b-2 transition-all ${
              activeTab === 'provider'
                ? 'border-accent text-primary'
                : 'border-transparent text-secondary hover:text-primary'
            }`}
          >
            <Cpu className="w-3.5 h-3.5" />
            <span>{t('Mô hình AI', 'AI Provider')}</span>
          </button>

          <button
            onClick={() => setActiveTab('security')}
            className={`flex items-center gap-2 py-2.5 px-3 text-xs font-medium border-b-2 transition-all ${
              activeTab === 'security'
                ? 'border-accent text-primary'
                : 'border-transparent text-secondary hover:text-primary'
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
                <label className="block font-medium text-primary mb-2">
                  {t('Chủ đề hiển thị', 'Theme')}
                </label>
                <div className="grid grid-cols-3 gap-2">
                  <button
                    onClick={() => setTheme('light')}
                    className={`flex items-center justify-center gap-2 p-2.5 rounded-xl border text-xs font-medium transition-all ${
                      theme === 'light'
                        ? 'border-accent bg-accent/10 text-primary'
                        : 'border-border bg-surface text-secondary hover:bg-muted'
                    }`}
                  >
                    <Sun className="w-4 h-4 text-amber-500" />
                    <span>{t('Sáng', 'Light')}</span>
                  </button>

                  <button
                    onClick={() => setTheme('dark')}
                    className={`flex items-center justify-center gap-2 p-2.5 rounded-xl border text-xs font-medium transition-all ${
                      theme === 'dark'
                        ? 'border-accent bg-accent/10 text-primary'
                        : 'border-border bg-surface text-secondary hover:bg-muted'
                    }`}
                  >
                    <Moon className="w-4 h-4 text-blue-400" />
                    <span>{t('Tối', 'Dark')}</span>
                  </button>

                  <button
                    onClick={() => setTheme('system')}
                    className={`flex items-center justify-center gap-2 p-2.5 rounded-xl border text-xs font-medium transition-all ${
                      theme === 'system'
                        ? 'border-accent bg-accent/10 text-primary'
                        : 'border-border bg-surface text-secondary hover:bg-muted'
                    }`}
                  >
                    <Laptop className="w-4 h-4 text-muted" />
                    <span>{t('Tự động', 'System')}</span>
                  </button>
                </div>
              </div>

              {/* Language Selection */}
              <div>
                <label className="block font-medium text-primary mb-2">
                  {t('Ngôn ngữ', 'Language')}
                </label>
                <div className="grid grid-cols-2 gap-2">
                  <button
                    onClick={() => setLanguage('vi')}
                    className={`flex items-center justify-center gap-2 p-2.5 rounded-xl border text-xs font-medium transition-all ${
                      language === 'vi'
                        ? 'border-accent bg-accent/10 text-primary'
                        : 'border-border bg-surface text-secondary hover:bg-muted'
                    }`}
                  >
                    <span>Tiếng Việt 🇻🇳</span>
                  </button>

                  <button
                    onClick={() => setLanguage('en')}
                    className={`flex items-center justify-center gap-2 p-2.5 rounded-xl border text-xs font-medium transition-all ${
                      language === 'en'
                        ? 'border-accent bg-accent/10 text-primary'
                        : 'border-border bg-surface text-secondary hover:bg-muted'
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
                <label className="block font-medium text-primary mb-1">
                  {t('Địa chỉ API Provider', 'Provider Endpoint')}
                </label>
                <input
                  type="text"
                  value={config.provider_url}
                  onChange={(e) => setConfig({ ...config, provider_url: e.target.value })}
                  placeholder="https://api.openai.com/v1"
                  className="w-full px-3 py-2 bg-muted/50 border border-border rounded-lg text-primary text-xs focus:outline-none focus:border-accent"
                />
              </div>

              <div>
                <label className="block font-medium text-primary mb-1">
                  {t('Tên mô hình', 'Model Name')}
                </label>
                <input
                  type="text"
                  value={config.model_name}
                  onChange={(e) => setConfig({ ...config, model_name: e.target.value })}
                  placeholder="gpt-4o"
                  className="w-full px-3 py-2 bg-muted/50 border border-border rounded-lg text-primary text-xs focus:outline-none focus:border-accent"
                />
              </div>

              <div>
                <label className="block font-medium text-primary mb-1">
                  {t('Khóa API (API Key)', 'API Key')}
                </label>
                <div className="relative">
                  <input
                    type={showApiKey ? 'text' : 'password'}
                    value={config.api_key}
                    onChange={(e) => setConfig({ ...config, api_key: e.target.value })}
                    placeholder="sk-..."
                    className="w-full pl-3 pr-9 py-2 bg-muted/50 border border-border rounded-lg text-primary text-xs focus:outline-none focus:border-accent font-mono"
                  />
                  <button
                    type="button"
                    onClick={() => setShowApiKey(!showApiKey)}
                    className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted hover:text-primary"
                  >
                    {showApiKey ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                  </button>
                </div>
              </div>

              <div className="pt-2 flex items-center justify-between">
                <button
                  type="button"
                  onClick={handleTestConnection}
                  disabled={testStatus === 'testing'}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-border bg-surface text-secondary hover:text-primary hover:bg-muted text-xs transition-colors"
                >
                  {testStatus === 'testing' && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
                  {testStatus === 'success' && <Check className="w-3.5 h-3.5 text-emerald-500" />}
                  {testStatus === 'failed' && <AlertTriangle className="w-3.5 h-3.5 text-red-500" />}
                  <span>
                    {testStatus === 'testing'
                      ? t('Đang kiểm tra...', 'Testing...')
                      : testStatus === 'success'
                      ? t('Kết nối thành công!', 'Connected!')
                      : testStatus === 'failed'
                      ? t('Kết nối thất bại', 'Connection Failed')
                      : t('Kiểm tra kết nối', 'Test Connection')}
                  </span>
                </button>
              </div>
            </div>
          )}

          {/* TAB 3: SYSTEM & SECURITY */}
          {activeTab === 'security' && config && (
            <div className="space-y-3.5">
              <div>
                <label className="block font-medium text-primary mb-1">
                  {t('Thư mục Workspace làm việc', 'Workspace Directory')}
                </label>
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={config.workspace_path}
                    onChange={(e) => setConfig({ ...config, workspace_path: e.target.value })}
                    className="flex-1 px-3 py-2 bg-muted/50 border border-border rounded-lg text-primary text-xs focus:outline-none focus:border-accent font-mono"
                  />
                </div>
              </div>

              <div>
                <label className="block font-medium text-primary mb-1">
                  {t('Chế độ phân quyền (Permission Mode)', 'Permission Mode')}
                </label>
                <div className="grid grid-cols-3 gap-2">
                  {(['Strict', 'WorkspaceAuto', 'FullAuto'] as PermissionMode[]).map((mode) => (
                    <button
                      key={mode}
                      onClick={() => setConfig({ ...config, permission_mode: mode })}
                      className={`p-2 rounded-xl border text-xs font-medium text-center transition-all ${
                        config.permission_mode === mode
                          ? 'border-accent bg-accent/10 text-primary'
                          : 'border-border bg-surface text-secondary hover:bg-muted'
                      }`}
                    >
                      {mode}
                    </button>
                  ))}
                </div>
              </div>

              <div className="p-3 rounded-xl bg-muted/40 border border-border-subtle flex items-center justify-between">
                <div>
                  <div className="font-medium text-primary">
                    {t('Dịch vụ chạy ngầm 24/7', '24/7 Background Runtime')}
                  </div>
                  <div className="text-[11px] text-muted">
                    {t('Trạng thái Windows Service', 'Windows Service State')}: {serviceStatus}
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-5 py-3 border-t border-border bg-surface">
          <div>
            {saveSuccess && (
              <span className="text-xs text-emerald-500 flex items-center gap-1 font-medium">
                <Check className="w-3.5 h-3.5" />
                {t('Đã lưu thành công!', 'Saved successfully!')}
              </span>
            )}
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={onClose}
              className="px-3 py-1.5 rounded-lg border border-border text-secondary hover:text-primary hover:bg-muted text-xs transition-colors"
            >
              {t('Đóng', 'Close')}
            </button>

            {activeTab !== 'appearance' && (
              <button
                onClick={handleSave}
                disabled={isSaving}
                className="px-3.5 py-1.5 rounded-lg bg-primary text-surface hover:opacity-90 font-medium text-xs transition-opacity shadow-sm disabled:opacity-50"
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
