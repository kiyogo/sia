import React, { useState } from 'react';
import {
  Briefcase,
  BarChart3,
  Mail,
  CheckCircle,
  XCircle,
  Play,
  Clock,
  Sparkles,
} from 'lucide-react';
import { BusinessConnectorDto, EmailDraftDto, BusinessApprovalDto, ColumnSummaryDto } from '../types';

export const BusinessDashboard: React.FC = () => {
  const [activeSection, setActiveSection] = useState<'workflows' | 'data' | 'email' | 'approvals'>('workflows');

  // Business connectors
  const [connectors] = useState<BusinessConnectorDto[]>([
    {
      id: 'conn-1',
      provider: 'Gmail (Google Workspace)',
      auth_type: 'OAuth2',
      status: 'Connected',
      scopes: ['gmail.readonly', 'gmail.compose'],
      rate_limit_per_minute: 120,
      last_connected_at: '2026-10-21T02:00:00Z',
      created_at: '2026-10-20T00:00:00Z',
    },
    {
      id: 'conn-2',
      provider: 'Google Calendar',
      auth_type: 'OAuth2',
      status: 'Connected',
      scopes: ['calendar.events.readonly', 'calendar.events'],
      rate_limit_per_minute: 60,
      last_connected_at: '2026-10-21T02:00:00Z',
      created_at: '2026-10-20T00:00:00Z',
    },
    {
      id: 'conn-3',
      provider: 'Microsoft Graph / Outlook',
      auth_type: 'OAuth2',
      status: 'Disconnected',
      scopes: ['Mail.Read', 'Mail.Send'],
      rate_limit_per_minute: 100,
      created_at: '2026-10-20T00:00:00Z',
    },
  ]);

  // Email drafts
  const [drafts] = useState<EmailDraftDto[]>([
    {
      id: 'draft-101',
      connector_id: 'conn-1',
      to_recipients: ['vp_strategy@enterprise.com'],
      cc_recipients: ['lead@workspace.ai'],
      bcc_recipients: [],
      subject: 'Q3 Market Research Deliverable & Competitor Matrix',
      body: 'Dear VP of Strategy,\n\nOur autonomous research pipeline has synthesized the Q3 market data, verified key analyst findings, and resolved metric discrepancies.\n\nPlease find the cited DOCX and PDF deliverables attached.',
      status: 'PendingApproval',
      operation_id: 'op_send_q3_report_8829',
      created_at: '2026-10-21T03:15:00Z',
      updated_at: '2026-10-21T03:15:00Z',
    },
  ]);

  // Pending Approvals
  const [approvals, setApprovals] = useState<BusinessApprovalDto[]>([
    {
      id: 'appr-1',
      action_type: 'SendEmail',
      target_resource: 'vp_strategy@enterprise.com',
      payload_preview: 'Subject: Q3 Market Research Deliverable & Competitor Matrix (DOCX + PDF)',
      status: 'Pending',
      created_at: '2026-10-21T03:15:00Z',
    },
  ]);

  // Tabular Data Summary Mock
  const [columnSummary] = useState<ColumnSummaryDto>({
    column_name: 'QuarterlyRevenue',
    count: 24,
    null_count: 0,
    sum: 1245000,
    mean: 51875,
    median: 49500,
    min: 22000,
    max: 98000,
    std_dev: 18450.2,
  });

  const handleResolveApproval = (id: string, approved: boolean) => {
    setApprovals((prev) =>
      prev.map((a) => (a.id === id ? { ...a, status: approved ? 'Approved' : 'Rejected' } : a))
    );
  };

  return (
    <div className="flex-1 flex flex-col h-full bg-slate-950 text-slate-100 overflow-hidden font-sans">
      {/* Top Header */}
      <header className="px-6 py-4 bg-slate-900/60 border-b border-slate-800 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-indigo-600/20 border border-indigo-500/40 flex items-center justify-center text-indigo-400 shadow-md">
            <Briefcase className="w-5 h-5" />
          </div>
          <div>
            <h1 className="text-lg font-semibold text-slate-100">Business & Data Automation</h1>
            <p className="text-xs text-slate-400">Enterprise workflows, data analytics, email drafts & human approvals</p>
          </div>
        </div>

        {/* Section Navigation */}
        <div className="flex items-center gap-1 bg-slate-900 border border-slate-800 rounded-lg p-1">
          <button
            onClick={() => setActiveSection('workflows')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-all flex items-center gap-1.5 ${
              activeSection === 'workflows'
                ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/50'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Play className="w-3.5 h-3.5" />
            Workflow Templates
          </button>
          <button
            onClick={() => setActiveSection('data')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-all flex items-center gap-1.5 ${
              activeSection === 'data'
                ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/50'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <BarChart3 className="w-3.5 h-3.5" />
            Data Analytics
          </button>
          <button
            onClick={() => setActiveSection('email')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-all flex items-center gap-1.5 ${
              activeSection === 'email'
                ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/50'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Mail className="w-3.5 h-3.5" />
            Email & Calendar
          </button>
          <button
            onClick={() => setActiveSection('approvals')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-all flex items-center gap-1.5 ${
              activeSection === 'approvals'
                ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/50'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Clock className="w-3.5 h-3.5" />
            Approvals ({approvals.filter((a) => a.status === 'Pending').length})
          </button>
        </div>
      </header>

      {/* Main Content Area */}
      <div className="flex-1 overflow-y-auto p-6 space-y-6">
        {/* Section 1: Workflow Templates */}
        {activeSection === 'workflows' && (
          <div className="max-w-4xl mx-auto space-y-6">
            <h2 className="text-sm font-semibold text-slate-200 uppercase tracking-wider">
              Production Business Workflow Templates
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-3 hover:border-indigo-500/40 transition-colors">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                    <Sparkles className="w-4 h-4 text-indigo-400" />
                    Market Research Pipeline
                  </h3>
                  <span className="text-[11px] px-2 py-0.5 bg-slate-800 text-slate-300 rounded">3 Tasks</span>
                </div>
                <p className="text-xs text-slate-400">
                  Search authorized web/document sources → Extract evidence → Check contradictions → Generate cited DOCX/PDF report.
                </p>
                <button className="w-full py-2 bg-indigo-600 hover:bg-indigo-500 text-slate-950 font-semibold rounded-lg text-xs flex items-center justify-center gap-2">
                  <Play className="w-3.5 h-3.5" /> Launch Market Research
                </button>
              </div>

              <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-3 hover:border-indigo-500/40 transition-colors">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                    <BarChart3 className="w-4 h-4 text-emerald-400" />
                    Data Reporting Pipeline
                  </h3>
                  <span className="text-[11px] px-2 py-0.5 bg-slate-800 text-slate-300 rounded">3 Tasks</span>
                </div>
                <p className="text-xs text-slate-400">
                  Ingest CSV/XLSX → Deterministic statistical calculations → Generate SVG charts → Export executive report.
                </p>
                <button className="w-full py-2 bg-emerald-600 hover:bg-emerald-500 text-slate-950 font-semibold rounded-lg text-xs flex items-center justify-center gap-2">
                  <Play className="w-3.5 h-3.5" /> Launch Data Reporting
                </button>
              </div>

              <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-3 hover:border-indigo-500/40 transition-colors">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                    <Mail className="w-4 h-4 text-cyan-400" />
                    Email Assistance & Action Items
                  </h3>
                  <span className="text-[11px] px-2 py-0.5 bg-slate-800 text-slate-300 rounded">3 Tasks</span>
                </div>
                <p className="text-xs text-slate-400">
                  Search connected mailbox → Summarize threads → Compose draft reply → Require human approval before dispatch.
                </p>
                <button className="w-full py-2 bg-cyan-600 hover:bg-cyan-500 text-slate-950 font-semibold rounded-lg text-xs flex items-center justify-center gap-2">
                  <Play className="w-3.5 h-3.5" /> Launch Email Assistant
                </button>
              </div>

              <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-3 hover:border-indigo-500/40 transition-colors">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                    <Briefcase className="w-4 h-4 text-amber-400" />
                    Hybrid Pipeline (Data → Report → Email)
                  </h3>
                  <span className="text-[11px] px-2 py-0.5 bg-slate-800 text-slate-300 rounded">3 Tasks</span>
                </div>
                <p className="text-xs text-slate-400">
                  Analyze dataset → Compile executive DOCX/PDF → Draft notification email with attachments and request approval.
                </p>
                <button className="w-full py-2 bg-amber-600 hover:bg-amber-500 text-slate-950 font-semibold rounded-lg text-xs flex items-center justify-center gap-2">
                  <Play className="w-3.5 h-3.5" /> Launch Hybrid Pipeline
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Section 2: Data Analytics */}
        {activeSection === 'data' && (
          <div className="max-w-4xl mx-auto space-y-6">
            <div className="bg-slate-900 border border-slate-800 rounded-xl p-6 space-y-4">
              <h2 className="text-sm font-semibold text-slate-200 flex items-center gap-2">
                <BarChart3 className="w-4 h-4 text-emerald-400" />
                Deterministic Tabular Data Analytics (Dataset: workspace/financials_q3.csv)
              </h2>

              <div className="grid grid-cols-4 gap-4 py-4 border-y border-slate-800 text-center">
                <div>
                  <div className="text-lg font-bold text-slate-100">{columnSummary.count}</div>
                  <div className="text-xs text-slate-400">Data Points</div>
                </div>
                <div>
                  <div className="text-lg font-bold text-slate-100">${(columnSummary.sum / 1000).toFixed(1)}K</div>
                  <div className="text-xs text-slate-400">Total Sum</div>
                </div>
                <div>
                  <div className="text-lg font-bold text-slate-100">${(columnSummary.mean / 1000).toFixed(1)}K</div>
                  <div className="text-xs text-slate-400">Mean Average</div>
                </div>
                <div>
                  <div className="text-lg font-bold text-slate-100">${(columnSummary.std_dev / 1000).toFixed(1)}K</div>
                  <div className="text-xs text-slate-400">Std Deviation</div>
                </div>
              </div>

              {/* Chart Preview */}
              <div className="space-y-2">
                <h4 className="text-xs font-semibold text-slate-400 uppercase tracking-wider">Generated Visualization Artifact</h4>
                <div className="bg-slate-950 p-4 rounded-xl border border-slate-800 flex justify-center">
                  <svg viewBox="0 0 500 200" className="w-full max-w-lg h-48">
                    <rect x="40" y="80" width="60" height="90" fill="#89b4fa" rx="4" />
                    <rect x="140" y="50" width="60" height="120" fill="#a6e3a1" rx="4" />
                    <rect x="240" y="30" width="60" height="140" fill="#f9e2af" rx="4" />
                    <rect x="340" y="10" width="60" height="160" fill="#fab387" rx="4" />
                    <text x="70" y="190" fill="#a6adc8" fontSize="11" textAnchor="middle">Q1 ($45K)</text>
                    <text x="170" y="190" fill="#a6adc8" fontSize="11" textAnchor="middle">Q2 ($58K)</text>
                    <text x="270" y="190" fill="#a6adc8" fontSize="11" textAnchor="middle">Q3 ($72K)</text>
                    <text x="370" y="190" fill="#a6adc8" fontSize="11" textAnchor="middle">Q4 ($91K)</text>
                  </svg>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Section 3: Email & Calendar Hub */}
        {activeSection === 'email' && (
          <div className="max-w-4xl mx-auto space-y-6">
            <div className="space-y-4">
              <h2 className="text-sm font-semibold text-slate-200">Connected Enterprise Accounts</h2>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {connectors.map((c) => (
                  <div key={c.id} className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3">
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-semibold text-slate-200">{c.provider}</span>
                      <span
                        className={`text-[10px] px-2 py-0.5 rounded font-semibold ${
                          c.status === 'Connected'
                            ? 'bg-emerald-950 text-emerald-300 border border-emerald-800'
                            : 'bg-slate-800 text-slate-400'
                        }`}
                      >
                        {c.status}
                      </span>
                    </div>
                    <div className="text-xs text-slate-400 space-y-1">
                      <p>Auth: {c.auth_type}</p>
                      <p>Rate Limit: {c.rate_limit_per_minute} req/min</p>
                      <p className="font-mono text-slate-500">Scopes: {c.scopes.join(', ')}</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="space-y-4">
              <h2 className="text-sm font-semibold text-slate-200">Email Drafts Pending Dispatch</h2>
              {drafts.map((d) => (
                <div key={d.id} className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-3">
                  <div className="flex items-center justify-between">
                    <h3 className="text-sm font-semibold text-slate-100">{d.subject}</h3>
                    <span className="text-xs px-2.5 py-0.5 rounded bg-amber-950 text-amber-300 border border-amber-800 font-medium">
                      {d.status}
                    </span>
                  </div>
                  <p className="text-xs text-slate-400">To: {d.to_recipients.join(', ')}</p>
                  <div className="bg-slate-950 p-3 rounded-lg border border-slate-800 text-xs font-mono text-slate-300 whitespace-pre-line">
                    {d.body}
                  </div>
                  <div className="flex items-center justify-between text-xs text-slate-400 pt-2 border-t border-slate-800">
                    <span className="font-mono text-slate-500">Operation ID: {d.operation_id}</span>
                    <span className="text-amber-400 font-medium">Requires approval in queue</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Section 4: Human Approval Queue */}
        {activeSection === 'approvals' && (
          <div className="max-w-4xl mx-auto space-y-6">
            <h2 className="text-sm font-semibold text-slate-200">Human Approval Queue for External Side Effects</h2>
            {approvals.length === 0 ? (
              <div className="bg-slate-900 border border-slate-800 rounded-xl p-8 text-center text-slate-400 text-xs">
                No actions currently pending approval.
              </div>
            ) : (
              <div className="space-y-4">
                {approvals.map((appr) => (
                  <div key={appr.id} className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-4">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="px-2.5 py-0.5 bg-rose-950/80 border border-rose-500/40 text-rose-300 text-xs font-semibold rounded">
                          {appr.action_type}
                        </span>
                        <span className="text-xs text-slate-400">Target: {appr.target_resource}</span>
                      </div>
                      <span className="text-xs text-slate-500">{new Date(appr.created_at).toLocaleTimeString()}</span>
                    </div>

                    <div className="bg-slate-950 p-3 rounded-lg border border-slate-800 text-xs font-mono text-slate-300">
                      {appr.payload_preview}
                    </div>

                    {appr.status === 'Pending' ? (
                      <div className="flex items-center justify-end gap-3 pt-2">
                        <button
                          onClick={() => handleResolveApproval(appr.id, false)}
                          className="px-4 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 font-semibold rounded-lg text-xs flex items-center gap-1.5"
                        >
                          <XCircle className="w-3.5 h-3.5 text-rose-400" />
                          Reject
                        </button>
                        <button
                          onClick={() => handleResolveApproval(appr.id, true)}
                          className="px-4 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-slate-950 font-semibold rounded-lg text-xs flex items-center gap-1.5 shadow-md"
                        >
                          <CheckCircle className="w-3.5 h-3.5" />
                          Approve External Send
                        </button>
                      </div>
                    ) : (
                      <div className="text-right text-xs font-medium">
                        Status:{' '}
                        <span className={appr.status === 'Approved' ? 'text-emerald-400' : 'text-rose-400'}>
                          {appr.status}
                        </span>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
