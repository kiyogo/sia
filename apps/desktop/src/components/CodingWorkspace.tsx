import React, { useState, useEffect } from 'react';
import { invokeCommand } from '../lib/tauriBridge';
import { CodingMode, BuildResult, ProjectInfo, WorkspaceFileInfo } from '../types';
import {
  Code2,
  GitBranch,
  Play,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  FileCode,
  Layers,
  Wrench,
  Sparkles,
  RefreshCw,
  FolderTree,
  FileDiff,
  BookOpen,
} from 'lucide-react';

export const CodingWorkspace: React.FC = () => {
  const [mode, setMode] = useState<CodingMode>('Build');
  const [projectInfo, setProjectInfo] = useState<ProjectInfo | null>(null);
  const [files, setFiles] = useState<WorkspaceFileInfo[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileContent, setFileContent] = useState<string>('');
  const [activeTab, setActiveTab] = useState<'editor' | 'diff' | 'plan' | 'repoMap'>('editor');
  const [diffContent, setDiffContent] = useState<string>('');
  const [repoMapContent, setRepoMapContent] = useState<string>('');
  const [buildResult, setBuildResult] = useState<BuildResult | null>(null);
  const [isBuilding, setIsBuilding] = useState<boolean>(false);
  const [taskPrompt, setTaskPrompt] = useState<string>('');
  const [isLaunching, setIsLaunching] = useState<boolean>(false);
  const [planSteps, setPlanSteps] = useState<string[]>([
    '1. Inspect project structure and dependencies',
    '2. Discover build system and active compiler',
    '3. Formulate targeted modification plan',
    '4. Apply atomic multi-file edits with rollback safety',
    '5. Execute build & typechecker to detect regressions',
    '6. Auto-repair diagnostics if compilation fails',
  ]);

  const handleModeChange = (newMode: CodingMode) => {
    setMode(newMode);
    if (newMode === 'Ask') {
      setPlanSteps([
        '1. Trace code references and symbol definitions',
        '2. Analyze architecture patterns and data flow',
        '3. Formulate thorough explanation and walkthrough',
      ]);
    } else if (newMode === 'Plan') {
      setPlanSteps([
        '1. Map impacted modules and source dependencies',
        '2. Outline required interface changes and data models',
        '3. Draft step-by-step verification checklist',
      ]);
    } else {
      setPlanSteps([
        '1. Inspect project structure and dependencies',
        '2. Discover build system and active compiler',
        '3. Formulate targeted modification plan',
        '4. Apply atomic multi-file edits with rollback safety',
        '5. Execute build & typechecker to detect regressions',
        '6. Auto-repair diagnostics if compilation fails',
      ]);
    }
  };

  useEffect(() => {
    loadProject();
    loadFiles();
  }, []);

  const loadProject = async () => {
    try {
      const info = await invokeCommand<ProjectInfo | null>('detect_project', { subpath: null });
      setProjectInfo(info);
    } catch (e) {
      console.error('Failed to detect project', e);
    }
  };

  const loadFiles = async () => {
    try {
      const fileList = await invokeCommand<WorkspaceFileInfo[]>('list_workspace_files', { subpath: null });
      setFiles(fileList);
      if (fileList.length > 0 && !selectedFile) {
        const firstCode = fileList.find((f) => !f.is_dir) || fileList[0];
        handleSelectFile(firstCode.path);
      }
    } catch (e) {
      console.error('Failed to list files', e);
    }
  };

  const handleSelectFile = async (path: string) => {
    setSelectedFile(path);
    try {
      const content = await invokeCommand<string>('read_workspace_file', { path });
      setFileContent(content);
      setActiveTab('editor');

      try {
        const diff = await invokeCommand<string>('preview_diff', { path, new_content: content });
        setDiffContent(diff);
      } catch {
        setDiffContent('');
      }
    } catch (e) {
      setFileContent(`Error reading file ${path}: ${e}`);
    }
  };

  const handleRunBuild = async () => {
    setIsBuilding(true);
    try {
      const res = await invokeCommand<BuildResult>('run_build', { command: null, path: null });
      setBuildResult(res);
    } catch (e) {
      console.error('Build execution failed', e);
    } finally {
      setIsBuilding(false);
    }
  };

  const handleRunTest = async () => {
    setIsBuilding(true);
    try {
      const res = await invokeCommand<BuildResult>('run_test', { command: null, filter: null, path: null });
      setBuildResult(res);
    } catch (e) {
      console.error('Test execution failed', e);
    } finally {
      setIsBuilding(false);
    }
  };

  const handleGenerateRepoMap = async () => {
    try {
      const map = await invokeCommand<string>('get_repo_map', { max_chars: 4000, subdir: null });
      setRepoMapContent(map);
      setActiveTab('repoMap');
    } catch (e) {
      console.error('Failed to get repo map', e);
    }
  };

  const handleLaunchCodingTask = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!taskPrompt.trim()) return;

    setIsLaunching(true);
    try {
      await invokeCommand<string>('start_coding_task', {
        title: taskPrompt.slice(0, 50),
        goal: taskPrompt,
        user_prompt: taskPrompt,
        mode: mode,
        conversation_id: null,
      });
      setTaskPrompt('');
      alert(`Autonomous ${mode} task started successfully! You can track its live execution in the Tasks tab.`);
    } catch (e) {
      alert(`Failed to start coding task: ${e}`);
    } finally {
      setIsLaunching(false);
    }
  };

  return (
    <div className="flex-1 flex flex-col h-full bg-slate-950 text-slate-100 overflow-hidden">
      {/* Top Header & Project Metadata Bar */}
      <div className="border-b border-slate-800 bg-slate-900/60 p-3 flex items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-indigo-500/10 border border-indigo-500/20 text-indigo-400">
            <Code2 className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="font-semibold text-sm text-slate-200">Coding Workspace</h2>
              <span className="text-xs px-2 py-0.5 rounded-full bg-indigo-500/20 text-indigo-300 font-medium">
                Phase 2 Core
              </span>
            </div>
            <div className="flex items-center gap-2 text-xs text-slate-400 mt-0.5">
              <span>Lang: <strong className="text-slate-300">{projectInfo?.primary_language || 'Auto-Detect'}</strong></span>
              <span>•</span>
              <span className="flex items-center gap-1">
                <GitBranch className="w-3 h-3 text-indigo-400" />
                {projectInfo?.git_branch || 'main'}
              </span>
              <span>•</span>
              <span>Build: <strong className="text-slate-300">{projectInfo?.build_systems[0] || 'cargo'}</strong></span>
            </div>
          </div>
        </div>

        {/* Mode Selector & Quick Actions */}
        <div className="flex items-center gap-2">
          {/* Mode Segmented Controls */}
          <div className="bg-slate-900 p-0.5 rounded-lg border border-slate-800 flex items-center">
            {(['Ask', 'Plan', 'Build'] as CodingMode[]).map((m) => (
              <button
                key={m}
                onClick={() => handleModeChange(m)}
                className={`px-3 py-1 rounded-md text-xs font-medium transition-all ${
                  mode === m
                    ? 'bg-indigo-600 text-white shadow-sm'
                    : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                {m === 'Ask' && 'Ask (Read-Only)'}
                {m === 'Plan' && 'Plan Only'}
                {m === 'Build' && 'Build (Autonomous)'}
              </button>
            ))}
          </div>

          <button
            onClick={handleGenerateRepoMap}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium border border-slate-700 transition"
          >
            <Layers className="w-3.5 h-3.5 text-indigo-400" />
            Repo Map
          </button>

          <button
            onClick={handleRunBuild}
            disabled={isBuilding}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium shadow-sm transition disabled:opacity-50"
          >
            <Play className={`w-3.5 h-3.5 ${isBuilding ? 'animate-spin' : ''}`} />
            {isBuilding ? 'Compiling...' : 'Run Build'}
          </button>

          <button
            onClick={handleRunTest}
            disabled={isBuilding}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-emerald-600/80 hover:bg-emerald-500 text-white text-xs font-medium transition disabled:opacity-50"
          >
            <CheckCircle2 className="w-3.5 h-3.5" />
            Run Tests
          </button>
        </div>
      </div>

      {/* Main Workspace Grid */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Column: File Explorer */}
        <div className="w-64 border-r border-slate-800 bg-slate-900/30 flex flex-col">
          <div className="p-3 border-b border-slate-800 flex items-center justify-between">
            <span className="text-xs font-medium text-slate-400 uppercase tracking-wider flex items-center gap-1.5">
              <FolderTree className="w-3.5 h-3.5" />
              Files
            </span>
            <button onClick={loadFiles} className="text-slate-400 hover:text-slate-200">
              <RefreshCw className="w-3.5 h-3.5" />
            </button>
          </div>
          <div className="flex-1 overflow-y-auto p-2 space-y-0.5 font-mono text-xs">
            {files.map((f) => (
              <button
                key={f.path}
                onClick={() => !f.is_dir && handleSelectFile(f.path)}
                className={`w-full text-left px-2 py-1.5 rounded flex items-center gap-2 truncate ${
                  selectedFile === f.path
                    ? 'bg-indigo-600/20 text-indigo-300 font-semibold border border-indigo-500/30'
                    : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'
                }`}
              >
                <FileCode className="w-3.5 h-3.5 shrink-0 text-slate-500" />
                <span className="truncate">{f.name}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Center Column: Code Editor / Diff Inspector / Plan */}
        <div className="flex-1 flex flex-col min-w-0 bg-slate-950">
          {/* Viewport Tabs */}
          <div className="flex items-center border-b border-slate-800 bg-slate-900/40 px-3">
            <button
              onClick={() => setActiveTab('editor')}
              className={`px-3 py-2 text-xs font-medium border-b-2 flex items-center gap-1.5 transition ${
                activeTab === 'editor'
                  ? 'border-indigo-500 text-indigo-400'
                  : 'border-transparent text-slate-400 hover:text-slate-200'
              }`}
            >
              <FileCode className="w-3.5 h-3.5" />
              {selectedFile || 'Source Viewer'}
            </button>
            <button
              onClick={() => setActiveTab('diff')}
              className={`px-3 py-2 text-xs font-medium border-b-2 flex items-center gap-1.5 transition ${
                activeTab === 'diff'
                  ? 'border-indigo-500 text-indigo-400'
                  : 'border-transparent text-slate-400 hover:text-slate-200'
              }`}
            >
              <FileDiff className="w-3.5 h-3.5" />
              Diff Preview
            </button>
            <button
              onClick={() => setActiveTab('plan')}
              className={`px-3 py-2 text-xs font-medium border-b-2 flex items-center gap-1.5 transition ${
                activeTab === 'plan'
                  ? 'border-indigo-500 text-indigo-400'
                  : 'border-transparent text-slate-400 hover:text-slate-200'
              }`}
            >
              <BookOpen className="w-3.5 h-3.5" />
              Agent Plan & Steps
            </button>
            <button
              onClick={() => setActiveTab('repoMap')}
              className={`px-3 py-2 text-xs font-medium border-b-2 flex items-center gap-1.5 transition ${
                activeTab === 'repoMap'
                  ? 'border-indigo-500 text-indigo-400'
                  : 'border-transparent text-slate-400 hover:text-slate-200'
              }`}
            >
              <Layers className="w-3.5 h-3.5" />
              Compact Repo Map
            </button>
          </div>

          {/* Viewport Content */}
          <div className="flex-1 overflow-auto p-4 font-mono text-xs">
            {activeTab === 'editor' && (
              <pre className="text-slate-300 whitespace-pre-wrap leading-relaxed select-text">
                {fileContent || '// Select a file to view source code.'}
              </pre>
            )}

            {activeTab === 'diff' && (
              <div className="space-y-1">
                {diffContent ? (
                  diffContent.split('\n').map((line, idx) => {
                    const isAdd = line.startsWith('+') && !line.startsWith('+++');
                    const isDel = line.startsWith('-') && !line.startsWith('---');
                    return (
                      <div
                        key={idx}
                        className={`px-2 py-0.5 rounded ${
                          isAdd
                            ? 'bg-emerald-950/40 text-emerald-300'
                            : isDel
                            ? 'bg-rose-950/40 text-rose-300'
                            : 'text-slate-400'
                        }`}
                      >
                        {line}
                      </div>
                    );
                  })
                ) : (
                  <div className="text-slate-500 py-8 text-center">
                    No active diff. Modifications made by the agent will be previewed here.
                  </div>
                )}
              </div>
            )}

            {activeTab === 'plan' && (
              <div className="space-y-3 font-sans">
                <div className="p-3 bg-indigo-950/20 border border-indigo-800/30 rounded-lg">
                  <h3 className="font-semibold text-sm text-indigo-300 mb-1 flex items-center gap-2">
                    <Sparkles className="w-4 h-4" />
                    Autonomous Execution Workflow ({mode} Mode)
                  </h3>
                  <p className="text-xs text-slate-400">
                    The agent breaks complex tasks down into atomic verification milestones.
                  </p>
                </div>
                <div className="space-y-2">
                  {planSteps.map((step, idx) => (
                    <div
                      key={idx}
                      className="p-2.5 rounded-lg bg-slate-900 border border-slate-800 flex items-center gap-3"
                    >
                      <div className="w-5 h-5 rounded-full bg-slate-800 border border-slate-700 flex items-center justify-center text-[10px] text-slate-300 font-bold">
                        {idx + 1}
                      </div>
                      <span className="text-xs text-slate-300 font-medium">{step}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {activeTab === 'repoMap' && (
              <pre className="text-indigo-300 whitespace-pre-wrap leading-relaxed select-text">
                {repoMapContent || 'Click "Repo Map" in the toolbar above to generate a compact symbol map.'}
              </pre>
            )}
          </div>

          {/* Bottom Panel: Problems & Diagnostics */}
          <div className="h-44 border-t border-slate-800 bg-slate-900/60 flex flex-col">
            <div className="px-3 py-1.5 border-b border-slate-800 flex items-center justify-between">
              <span className="text-xs font-semibold text-slate-300 flex items-center gap-2">
                <Wrench className="w-3.5 h-3.5 text-indigo-400" />
                Problems & Diagnostics
                {buildResult && (
                  <span
                    className={`ml-2 px-1.5 py-0.5 rounded text-[10px] font-bold ${
                      buildResult.success
                        ? 'bg-emerald-500/20 text-emerald-300'
                        : 'bg-rose-500/20 text-rose-300'
                    }`}
                  >
                    {buildResult.total_errors} Errors, {buildResult.total_warnings} Warnings
                  </span>
                )}
              </span>
              {buildResult && (
                <span className="text-[11px] text-slate-400">
                  Build duration: {buildResult.duration_ms}ms
                </span>
              )}
            </div>

            <div className="flex-1 overflow-y-auto p-2 space-y-1 font-mono text-xs">
              {buildResult && buildResult.diagnostics.length > 0 ? (
                buildResult.diagnostics.map((d, idx) => (
                  <div
                    key={idx}
                    className="p-1.5 rounded bg-slate-950 border border-slate-800/80 flex items-start gap-2 hover:border-slate-700 cursor-pointer"
                    onClick={() => handleSelectFile(d.file)}
                  >
                    {d.severity === 'Error' ? (
                      <XCircle className="w-3.5 h-3.5 text-rose-400 shrink-0 mt-0.5" />
                    ) : (
                      <AlertTriangle className="w-3.5 h-3.5 text-amber-400 shrink-0 mt-0.5" />
                    )}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-semibold text-slate-200 truncate">{d.message}</span>
                        {d.code && <span className="text-slate-500 font-mono text-[10px]">[{d.code}]</span>}
                      </div>
                      <div className="text-[11px] text-slate-400">
                        {d.file}:{d.line}:{d.column}
                      </div>
                    </div>
                  </div>
                ))
              ) : (
                <div className="h-full flex items-center justify-center text-slate-500 text-xs">
                  {buildResult ? 'No compiler diagnostics reported. Clean build!' : 'Run build to inspect compiler problems.'}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Autonomous Coding Agent Prompt Bar */}
      <form onSubmit={handleLaunchCodingTask} className="p-3 border-t border-slate-800 bg-slate-900/90 flex items-center gap-3">
        <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-indigo-950 border border-indigo-800/40 text-indigo-300 text-xs font-semibold shrink-0">
          <Sparkles className="w-3.5 h-3.5 text-indigo-400" />
          {mode}
        </div>
        <input
          type="text"
          value={taskPrompt}
          onChange={(e) => setTaskPrompt(e.target.value)}
          placeholder={`Instruct Autonomous Agent in ${mode} mode (e.g., 'Refactor error handling in auth.rs and verify with cargo test')...`}
          className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:border-indigo-500 transition"
        />
        <button
          type="submit"
          disabled={isLaunching || !taskPrompt.trim()}
          className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/20 disabled:opacity-50 transition flex items-center gap-2 shrink-0"
        >
          <Play className="w-3.5 h-3.5" />
          {isLaunching ? 'Dispatching...' : 'Execute'}
        </button>
      </form>
    </div>
  );
};
