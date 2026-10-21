import React, { useState } from 'react';
import {
  Search,
  BookOpen,
  FileText,
  AlertTriangle,
  CheckCircle2,
  ExternalLink,
  ShieldCheck,
  FileCheck,
  Sparkles,
} from 'lucide-react';
import { ResearchSourceDto, EvidenceItemDto, DocumentMetadataDto } from '../types';

export const ResearchWorkspace: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'search' | 'sources' | 'evidence' | 'parser' | 'reports'>('search');
  const [searchQuery, setSearchQuery] = useState('Enterprise autonomous AI agent market adoption');
  const [isSearching, setIsSearching] = useState(false);

  // Mock / live sources state
  const [sources] = useState<ResearchSourceDto[]>([
    {
      id: 'src-1',
      url_or_path: 'https://gartner.example.com/ai-market-2026',
      title: 'Gartner Predicts Enterprise AI Market to Double by 2028',
      source_type: 'Web',
      retrieved_at: '2026-10-21T02:00:00Z',
      published_date: '2026-03-15',
      content_hash: 'sha256:7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069',
      verification_status: 'Verified',
      raw_snippet: 'Enterprise AI market adoption is expanding at 28% CAGR across Fortune 500 companies.',
    },
    {
      id: 'src-2',
      url_or_path: 'https://idc.example.com/market-spend',
      title: 'IDC Worldwide AI Spending Guide 2026',
      source_type: 'Web',
      retrieved_at: '2026-10-21T02:15:00Z',
      published_date: '2026-02-20',
      content_hash: 'sha256:1a823c6f8a84618e00e47087f978189c44d183eeab0b938c264264771746f332',
      verification_status: 'Verified',
      raw_snippet: 'Global spending on artificial intelligence software and infrastructure will exceed $300B by year end.',
    },
    {
      id: 'src-3',
      url_or_path: 'https://competitor.example.com/press-release',
      title: 'Competitor Press Statement: Enterprise Valuations',
      source_type: 'Web',
      retrieved_at: '2026-10-21T02:30:00Z',
      published_date: '2026-04-01',
      content_hash: 'sha256:9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
      verification_status: 'Contradicted',
      raw_snippet: 'Competitor claims the enterprise AI market size is already over $600B.',
    },
  ]);

  const [evidence] = useState<EvidenceItemDto[]>([
    {
      id: 'ev-1',
      source_id: 'src-1',
      claim_text: 'Enterprise AI adoption is expanding at 28% CAGR',
      evidence_type: 'Fact',
      page_or_section: 'Section 2.1 p.4',
      confidence_score: 0.96,
      verification_status: 'Verified',
      created_at: '2026-10-21T02:05:00Z',
    },
    {
      id: 'ev-2',
      source_id: 'src-2',
      claim_text: 'Worldwide enterprise software AI spending exceeds $300B',
      evidence_type: 'Fact',
      page_or_section: 'Executive Summary p.1',
      confidence_score: 0.92,
      verification_status: 'Verified',
      created_at: '2026-10-21T02:18:00Z',
    },
    {
      id: 'ev-3',
      source_id: 'src-3',
      claim_text: 'Enterprise AI market already reached $600B in Q1',
      evidence_type: 'Claim',
      page_or_section: 'Paragraph 3',
      confidence_score: 0.61,
      verification_status: 'Contradicted',
      created_at: '2026-10-21T02:32:00Z',
    },
  ]);

  const [parsedDoc] = useState<DocumentMetadataDto>({
    id: 'doc-1',
    file_path: 'workspace/market_analysis.pdf',
    format: 'PDF',
    file_size_bytes: 524288,
    title: 'Enterprise AI Market Outlook 2026',
    page_count: 12,
    word_count: 3840,
    table_count: 4,
    content_hash: 'sha256:a1b2c3d4...',
    parsed_summary: 'Structured document containing 12 pages, 4 tables, and 18 section headings.',
    created_at: '2026-10-21T03:00:00Z',
  });

  const handleSearch = () => {
    setIsSearching(true);
    setTimeout(() => {
      setIsSearching(false);
    }, 600);
  };

  return (
    <div className="flex-1 flex flex-col h-full bg-slate-950 text-slate-100 overflow-hidden font-sans">
      {/* Top Header */}
      <header className="px-6 py-4 bg-slate-900/60 border-b border-slate-800 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-cyan-600/20 border border-cyan-500/40 flex items-center justify-center text-cyan-400 shadow-md">
            <BookOpen className="w-5 h-5" />
          </div>
          <div>
            <h1 className="text-lg font-semibold text-slate-100">Research & Document Intelligence</h1>
            <p className="text-xs text-slate-400">Verifiable source provenance, structured parsing, and citation reports</p>
          </div>
        </div>

        {/* Tab Controls */}
        <div className="flex items-center gap-1 bg-slate-900 border border-slate-800 rounded-lg p-1">
          <button
            onClick={() => setActiveTab('search')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-all flex items-center gap-1.5 ${
              activeTab === 'search'
                ? 'bg-cyan-600/30 text-cyan-300 border border-cyan-500/50'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Search className="w-3.5 h-3.5" />
            Live Search
          </button>
          <button
            onClick={() => setActiveTab('sources')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-all flex items-center gap-1.5 ${
              activeTab === 'sources'
                ? 'bg-cyan-600/30 text-cyan-300 border border-cyan-500/50'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <ShieldCheck className="w-3.5 h-3.5" />
            Sources ({sources.length})
          </button>
          <button
            onClick={() => setActiveTab('evidence')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-all flex items-center gap-1.5 ${
              activeTab === 'evidence'
                ? 'bg-cyan-600/30 text-cyan-300 border border-cyan-500/50'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <CheckCircle2 className="w-3.5 h-3.5" />
            Evidence ({evidence.length})
          </button>
          <button
            onClick={() => setActiveTab('parser')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-all flex items-center gap-1.5 ${
              activeTab === 'parser'
                ? 'bg-cyan-600/30 text-cyan-300 border border-cyan-500/50'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <FileText className="w-3.5 h-3.5" />
            Document Parser
          </button>
          <button
            onClick={() => setActiveTab('reports')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-all flex items-center gap-1.5 ${
              activeTab === 'reports'
                ? 'bg-cyan-600/30 text-cyan-300 border border-cyan-500/50'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <FileCheck className="w-3.5 h-3.5" />
            Cited Reports
          </button>
        </div>
      </header>

      {/* Main Content Area */}
      <div className="flex-1 overflow-y-auto p-6 space-y-6">
        {/* Tab 1: Live Search */}
        {activeTab === 'search' && (
          <div className="space-y-6 max-w-4xl mx-auto">
            <div className="bg-slate-900 border border-slate-800 rounded-xl p-6 space-y-4 shadow-sm">
              <h2 className="text-sm font-semibold text-slate-200 flex items-center gap-2">
                <Search className="w-4 h-4 text-cyan-400" />
                Autonomous Deep Research Query
              </h2>
              <div className="flex gap-3">
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Enter research topic, question, or technology comparison..."
                  className="flex-1 bg-slate-950 border border-slate-700/80 rounded-lg px-4 py-2.5 text-sm text-slate-200 focus:outline-none focus:border-cyan-500"
                />
                <button
                  onClick={handleSearch}
                  disabled={isSearching}
                  className="px-5 py-2.5 bg-cyan-600 hover:bg-cyan-500 text-slate-950 font-semibold rounded-lg text-sm flex items-center gap-2 shadow-md disabled:opacity-50"
                >
                  <Sparkles className="w-4 h-4" />
                  {isSearching ? 'Researching...' : 'Execute Deep Search'}
                </button>
              </div>
              <p className="text-xs text-slate-400">
                Queries authorized search providers, extracts evidence snippets with timestamped citations, and cross-examines findings for factual contradictions.
              </p>
            </div>

            {/* Results preview */}
            <div className="space-y-4">
              <h3 className="text-xs font-semibold text-slate-400 uppercase tracking-wider">Discovered Sources & Snippets</h3>
              {sources.map((src) => (
                <div key={src.id} className="bg-slate-900/80 border border-slate-800 rounded-xl p-4 space-y-2 hover:border-slate-700 transition-colors">
                  <div className="flex items-start justify-between gap-4">
                    <div>
                      <h4 className="text-sm font-medium text-cyan-300 hover:underline cursor-pointer flex items-center gap-1.5">
                        {src.title}
                        <ExternalLink className="w-3.5 h-3.5 text-slate-500" />
                      </h4>
                      <p className="text-xs text-slate-500 font-mono mt-0.5">{src.url_or_path}</p>
                    </div>
                    <span
                      className={`text-xs px-2.5 py-0.5 rounded-full font-medium ${
                        src.verification_status === 'Verified'
                          ? 'bg-emerald-950/70 border border-emerald-500/40 text-emerald-300'
                          : 'bg-rose-950/70 border border-rose-500/40 text-rose-300'
                      }`}
                    >
                      {src.verification_status}
                    </span>
                  </div>
                  <p className="text-xs text-slate-300 bg-slate-950/60 p-2.5 rounded-lg border border-slate-800/80">
                    "{src.raw_snippet}"
                  </p>
                  <div className="flex items-center gap-4 text-[11px] text-slate-400">
                    <span>Published: {src.published_date || 'Unknown'}</span>
                    <span>Retrieved: {new Date(src.retrieved_at).toLocaleString()}</span>
                    <span className="font-mono text-slate-500">Hash: {src.content_hash.slice(0, 16)}...</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Tab 2: Sources Library */}
        {activeTab === 'sources' && (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-sm font-semibold text-slate-200">Registered Research Source Catalog</h2>
              <span className="text-xs text-slate-400">Deduplicated by canonical URL and SHA-256 content hashes</span>
            </div>
            <div className="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden">
              <table className="w-full text-left text-xs">
                <thead className="bg-slate-800/60 text-slate-400 border-b border-slate-800">
                  <tr>
                    <th className="py-3 px-4 font-medium">Source Title</th>
                    <th className="py-3 px-4 font-medium">Type</th>
                    <th className="py-3 px-4 font-medium">URL / Location</th>
                    <th className="py-3 px-4 font-medium">Status</th>
                    <th className="py-3 px-4 font-medium">Retrieved</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-800/60 text-slate-300">
                  {sources.map((s) => (
                    <tr key={s.id} className="hover:bg-slate-800/30">
                      <td className="py-3 px-4 font-medium text-slate-200">{s.title}</td>
                      <td className="py-3 px-4 text-cyan-400 font-mono">{s.source_type}</td>
                      <td className="py-3 px-4 text-slate-400 font-mono">{s.url_or_path}</td>
                      <td className="py-3 px-4">
                        <span
                          className={`px-2 py-0.5 rounded text-[11px] font-medium ${
                            s.verification_status === 'Verified'
                              ? 'bg-emerald-950 text-emerald-300 border border-emerald-800'
                              : 'bg-rose-950 text-rose-300 border border-rose-800'
                          }`}
                        >
                          {s.verification_status}
                        </span>
                      </td>
                      <td className="py-3 px-4 text-slate-400">{new Date(s.retrieved_at).toLocaleDateString()}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {/* Tab 3: Evidence & Contradiction Detection */}
        {activeTab === 'evidence' && (
          <div className="space-y-6">
            {/* Contradiction Alert Banner */}
            <div className="bg-amber-950/40 border border-amber-500/40 rounded-xl p-4 flex items-start gap-3">
              <AlertTriangle className="w-5 h-5 text-amber-400 shrink-0 mt-0.5" />
              <div>
                <h3 className="text-sm font-semibold text-amber-300">Discrepancy Detected Between Sources</h3>
                <p className="text-xs text-amber-200/80 mt-1">
                  Source 2 (IDC) states worldwide AI spending exceeds $300B, while Source 3 claims $600B in enterprise valuations. Flagged for supervisor review and cross-examination.
                </p>
              </div>
            </div>

            <div className="space-y-4">
              <h2 className="text-sm font-semibold text-slate-200">Evidence Store Items ({evidence.length})</h2>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {evidence.map((ev) => (
                  <div key={ev.id} className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3">
                    <div className="flex items-center justify-between">
                      <span className="text-[11px] px-2 py-0.5 bg-slate-800 text-cyan-300 font-semibold rounded">
                        {ev.evidence_type}
                      </span>
                      <span className="text-xs text-slate-400">Score: {(ev.confidence_score * 100).toFixed(0)}%</span>
                    </div>
                    <p className="text-sm text-slate-200 font-medium">"{ev.claim_text}"</p>
                    <div className="text-xs text-slate-400 pt-2 border-t border-slate-800">
                      <p>Location: {ev.page_or_section || 'Full Document'}</p>
                      <p className="font-mono text-slate-500 mt-0.5">Source: {ev.source_id}</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {/* Tab 4: Document Parser */}
        {activeTab === 'parser' && (
          <div className="space-y-6 max-w-4xl mx-auto">
            <div className="bg-slate-900 border border-slate-800 rounded-xl p-6 space-y-4">
              <div className="flex items-start justify-between">
                <div>
                  <h2 className="text-base font-semibold text-slate-200">{parsedDoc.title}</h2>
                  <p className="text-xs text-slate-400 font-mono mt-0.5">{parsedDoc.file_path}</p>
                </div>
                <span className="px-3 py-1 bg-cyan-950 text-cyan-300 font-semibold border border-cyan-800 rounded-lg text-xs">
                  {parsedDoc.format} Document
                </span>
              </div>

              <div className="grid grid-cols-4 gap-4 py-4 border-y border-slate-800 text-center">
                <div>
                  <div className="text-lg font-bold text-slate-100">{parsedDoc.page_count}</div>
                  <div className="text-xs text-slate-400">Total Pages</div>
                </div>
                <div>
                  <div className="text-lg font-bold text-slate-100">{parsedDoc.word_count?.toLocaleString()}</div>
                  <div className="text-xs text-slate-400">Words Extracted</div>
                </div>
                <div>
                  <div className="text-lg font-bold text-slate-100">{parsedDoc.table_count}</div>
                  <div className="text-xs text-slate-400">Tables Parsed</div>
                </div>
                <div>
                  <div className="text-lg font-bold text-slate-100">{(parsedDoc.file_size_bytes / 1024).toFixed(1)} KB</div>
                  <div className="text-xs text-slate-400">File Size</div>
                </div>
              </div>

              <div className="space-y-2">
                <h4 className="text-xs font-semibold text-slate-400 uppercase tracking-wider">Structured Content Preview</h4>
                <div className="bg-slate-950 p-4 rounded-lg border border-slate-800 text-xs font-mono text-slate-300 space-y-2">
                  <p className="text-cyan-400 font-bold"># Executive Summary</p>
                  <p className="text-slate-300">The global enterprise autonomous AI workspace market is expanding rapidly as organizations replace single prompt models with multi-agent runtimes.</p>
                  <p className="text-cyan-400 font-bold"># Market Share Breakdown (Table 1)</p>
                  <p className="text-slate-400">| Sector | Growth Rate | Primary Driver |</p>
                  <p className="text-slate-400">| Enterprise Software | 28% | Local Data Privacy & Tool Orchestration |</p>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Tab 5: Cited Reports */}
        {activeTab === 'reports' && (
          <div className="max-w-4xl mx-auto bg-slate-900 border border-slate-800 rounded-xl p-8 space-y-6">
            <div className="border-b border-slate-800 pb-4">
              <h2 className="text-xl font-bold text-slate-100">Executive Report: Q3 Autonomous AI Market Overview</h2>
              <p className="text-xs text-slate-400 mt-1">Generated by Research & Document Agent with Verifiable Citations</p>
            </div>

            <div className="prose prose-invert max-w-none text-sm text-slate-300 space-y-4">
              <h3 className="text-base font-semibold text-slate-100">1. Executive Summary</h3>
              <p>
                The enterprise market for autonomous AI workspaces is experiencing accelerated adoption across Fortune 500 organizations, driven by multi-agent coordination and data sovereignty [^1]. Current industry expenditure is forecast to exceed $300 billion annually by the conclusion of the fiscal cycle [^2].
              </p>

              <h3 className="text-base font-semibold text-slate-100">2. Discrepancy & Verification Notice</h3>
              <p className="text-amber-300/90 bg-amber-950/30 p-3 rounded-lg border border-amber-800/50">
                ⚠️ Conflicting claims exist regarding total market valuation metrics: While preliminary press releases suggested an immediate $600B market, audited analyst research indicates a verified figure of $300B [^3].
              </p>

              <h3 className="text-base font-semibold text-slate-100 pt-4 border-t border-slate-800">References & Evidence Sources</h3>
              <ol className="list-decimal list-inside space-y-1 text-xs text-slate-400 font-mono">
                <li>[^1] Gartner Predicts Enterprise AI Market to Double by 2028 (https://gartner.example.com/ai-market-2026)</li>
                <li>[^2] IDC Worldwide AI Spending Guide 2026 (https://idc.example.com/market-spend)</li>
                <li>[^3] Competitor Press Statement: Enterprise Valuations (https://competitor.example.com/press-release)</li>
              </ol>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
