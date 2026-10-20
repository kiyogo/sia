import React, { useState, useEffect } from 'react';
import {
  WorkflowDto,
  WorkflowTaskDto,
  ArtifactRefDto,
  AgentDefinitionDto,
  WorkflowStatus,
  WorkflowTaskStatus,
} from '../types';
import { invokeCommand } from '../lib/tauriBridge';
import {
  Network,
  Users,
  Play,
  Pause,
  XCircle,
  Plus,
  RefreshCw,
  AlertTriangle,
  FileCode,
  Globe,
  Monitor,
  Search,
  Sliders,
  Layers,
  ArrowRight,
  Shield,
} from 'lucide-react';

export const WorkflowOrchestrator: React.FC = () => {
  const [workflows, setWorkflows] = useState<WorkflowDto[]>([]);
  const [selectedWorkflow, setSelectedWorkflow] = useState<WorkflowDto | null>(null);
  const [tasks, setTasks] = useState<WorkflowTaskDto[]>([]);
  const [artifacts, setArtifacts] = useState<ArtifactRefDto[]>([]);
  const [agents, setAgents] = useState<AgentDefinitionDto[]>([]);
  const [selectedTask, setSelectedTask] = useState<WorkflowTaskDto | null>(null);
  const [activeTab, setActiveTab] = useState<'dag' | 'agents' | 'artifacts' | 'inspector'>('dag');
  const [loading, setLoading] = useState(false);

  // New Workflow Modal state
  const [showNewModal, setShowNewModal] = useState(false);
  const [newTitle, setNewTitle] = useState('');
  const [newGoal, setNewGoal] = useState('');

  const fetchWorkflows = async () => {
    try {
      const list = await invokeCommand<WorkflowDto[]>('list_workflows');
      setWorkflows(list);
      if (list.length > 0 && !selectedWorkflow) {
        setSelectedWorkflow(list[0]);
      }
    } catch (e) {
      console.error('Failed to list workflows:', e);
    }
  };

  const fetchAgents = async () => {
    try {
      const list = await invokeCommand<AgentDefinitionDto[]>('list_registered_agents');
      setAgents(list);
    } catch (e) {
      console.error('Failed to list agents:', e);
    }
  };

  const fetchWorkflowDetails = async (wfId: string) => {
    try {
      setLoading(true);
      const [tList, aList] = await Promise.all([
        invokeCommand<WorkflowTaskDto[]>('list_workflow_tasks', { workflowId: wfId }),
        invokeCommand<ArtifactRefDto[]>('list_workflow_artifacts', { workflowId: wfId }),
      ]);
      setTasks(tList);
      setArtifacts(aList);
      if (tList.length > 0 && !selectedTask) {
        setSelectedTask(tList[0]);
      }
    } catch (e) {
      console.error('Failed to fetch workflow details:', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchWorkflows();
    fetchAgents();
  }, []);

  useEffect(() => {
    if (selectedWorkflow) {
      fetchWorkflowDetails(selectedWorkflow.id);
    }
  }, [selectedWorkflow]);

  const handleCreateWorkflow = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newTitle.trim() || !newGoal.trim()) return;

    try {
      setLoading(true);
      const wfId = await invokeCommand<string>('create_workflow', {
        title: newTitle,
        goal: newGoal,
      });
      setShowNewModal(false);
      setNewTitle('');
      setNewGoal('');
      await fetchWorkflows();
      const updatedList = await invokeCommand<WorkflowDto[]>('list_workflows');
      const created = updatedList.find((w) => w.id === wfId);
      if (created) setSelectedWorkflow(created);
    } catch (e) {
      console.error('Failed to create workflow:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleRunWorkflow = async () => {
    if (!selectedWorkflow) return;
    try {
      setLoading(true);
      await invokeCommand('run_workflow', { workflowId: selectedWorkflow.id });
      await fetchWorkflows();
      if (selectedWorkflow) fetchWorkflowDetails(selectedWorkflow.id);
    } catch (e) {
      console.error('Failed to run workflow:', e);
    } finally {
      setLoading(false);
    }
  };

  const handlePauseWorkflow = async () => {
    if (!selectedWorkflow) return;
    try {
      await invokeCommand('pause_workflow', { workflowId: selectedWorkflow.id });
      fetchWorkflows();
    } catch (e) {
      console.error('Failed to pause workflow:', e);
    }
  };

  const handleCancelWorkflow = async () => {
    if (!selectedWorkflow) return;
    try {
      await invokeCommand('cancel_workflow', { workflowId: selectedWorkflow.id });
      fetchWorkflows();
    } catch (e) {
      console.error('Failed to cancel workflow:', e);
    }
  };

  const handleToggleAgent = async (agentId: string, currentEnabled: boolean) => {
    try {
      await invokeCommand('toggle_agent_status', { agentId, enabled: !currentEnabled });
      fetchAgents();
    } catch (e) {
      console.error('Failed to toggle agent:', e);
    }
  };

  const getStatusBadge = (status: WorkflowStatus | WorkflowTaskStatus) => {
    switch (status) {
      case 'Completed':
        return <span className="px-2 py-0.5 text-xs rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-medium">Completed</span>;
      case 'Running':
        return <span className="px-2 py-0.5 text-xs rounded bg-blue-500/10 text-blue-400 border border-blue-500/20 font-medium animate-pulse">Running</span>;
      case 'WaitingApproval':
        return <span className="px-2 py-0.5 text-xs rounded bg-amber-500/10 text-amber-400 border border-amber-500/20 font-medium">Approval</span>;
      case 'Failed':
        return <span className="px-2 py-0.5 text-xs rounded bg-rose-500/10 text-rose-400 border border-rose-500/20 font-medium">Failed</span>;
      case 'Cancelled':
        return <span className="px-2 py-0.5 text-xs rounded bg-slate-500/10 text-slate-400 border border-slate-500/20 font-medium">Cancelled</span>;
      case 'Paused':
        return <span className="px-2 py-0.5 text-xs rounded bg-purple-500/10 text-purple-400 border border-purple-500/20 font-medium">Paused</span>;
      default:
        return <span className="px-2 py-0.5 text-xs rounded bg-slate-800 text-slate-400 border border-slate-700 font-medium">{status}</span>;
    }
  };

  const getAgentIcon = (agentType: string) => {
    switch (agentType) {
      case 'Coding':
        return <FileCode className="w-4 h-4 text-emerald-400" />;
      case 'Browser':
        return <Globe className="w-4 h-4 text-sky-400" />;
      case 'Computer':
        return <Monitor className="w-4 h-4 text-indigo-400" />;
      case 'Research':
        return <Search className="w-4 h-4 text-amber-400" />;
      default:
        return <Network className="w-4 h-4 text-purple-400" />;
    }
  };

  return (
    <div className="flex flex-col h-full bg-slate-950 text-slate-100 overflow-hidden">
      {/* Top Bar */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800 bg-slate-900/40">
        <div className="flex items-center gap-4">
          <div className="p-2.5 rounded-xl bg-purple-500/10 border border-purple-500/20 text-purple-400">
            <Network className="w-6 h-6" />
          </div>
          <div>
            <h1 className="text-xl font-bold text-slate-100 flex items-center gap-2">
              Multi-Agent Orchestration
              <span className="text-xs px-2 py-0.5 rounded-full bg-purple-500/20 text-purple-300 border border-purple-500/30 font-normal">
                Phase 4 Verified
              </span>
            </h1>
            <p className="text-xs text-slate-400">
              Supervisor AI · Task DAG · Tokio Scheduler · Shared Artifacts · Resource Locks
            </p>
          </div>
        </div>

        {/* Workflow Selector & Controls */}
        <div className="flex items-center gap-3">
          {workflows.length > 0 && (
            <select
              value={selectedWorkflow?.id || ''}
              onChange={(e) => {
                const found = workflows.find((w) => w.id === e.target.value);
                if (found) setSelectedWorkflow(found);
              }}
              className="bg-slate-900 border border-slate-700 text-slate-200 text-sm rounded-lg px-3 py-1.5 focus:outline-none focus:ring-1 focus:ring-purple-500"
            >
              {workflows.map((w) => (
                <option key={w.id} value={w.id}>
                  {w.title} ({w.status})
                </option>
              ))}
            </select>
          )}

          {selectedWorkflow && (
            <>
              {selectedWorkflow.status === 'Running' ? (
                <button
                  onClick={handlePauseWorkflow}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-amber-500/10 text-amber-400 border border-amber-500/30 hover:bg-amber-500/20 text-xs font-medium transition"
                >
                  <Pause className="w-3.5 h-3.5" /> Pause
                </button>
              ) : (
                <button
                  onClick={handleRunWorkflow}
                  disabled={loading || selectedWorkflow.status === 'Completed'}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 hover:bg-emerald-500/20 text-xs font-medium transition disabled:opacity-50"
                >
                  <Play className="w-3.5 h-3.5" /> Run Workflow
                </button>
              )}

              <button
                onClick={handleCancelWorkflow}
                disabled={selectedWorkflow.status === 'Cancelled' || selectedWorkflow.status === 'Completed'}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-rose-500/10 text-rose-400 border border-rose-500/30 hover:bg-rose-500/20 text-xs font-medium transition disabled:opacity-50"
              >
                <XCircle className="w-3.5 h-3.5" /> Cancel
              </button>
            </>
          )}

          <button
            onClick={() => setShowNewModal(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-purple-600 hover:bg-purple-500 text-white text-xs font-medium transition"
          >
            <Plus className="w-3.5 h-3.5" /> New Workflow
          </button>

          <button
            onClick={() => {
              fetchWorkflows();
              if (selectedWorkflow) fetchWorkflowDetails(selectedWorkflow.id);
            }}
            className="p-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 transition"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* Tabs Sub-nav */}
      <div className="flex items-center justify-between px-6 border-b border-slate-800 bg-slate-900/20">
        <div className="flex gap-4">
          <button
            onClick={() => setActiveTab('dag')}
            className={`py-3 px-2 text-sm font-medium border-b-2 flex items-center gap-2 transition ${
              activeTab === 'dag'
                ? 'border-purple-500 text-purple-400'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Network className="w-4 h-4" /> Task DAG Graph ({tasks.length})
          </button>
          <button
            onClick={() => setActiveTab('agents')}
            className={`py-3 px-2 text-sm font-medium border-b-2 flex items-center gap-2 transition ${
              activeTab === 'agents'
                ? 'border-purple-500 text-purple-400'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Users className="w-4 h-4" /> Agent Registry ({agents.length})
          </button>
          <button
            onClick={() => setActiveTab('artifacts')}
            className={`py-3 px-2 text-sm font-medium border-b-2 flex items-center gap-2 transition ${
              activeTab === 'artifacts'
                ? 'border-purple-500 text-purple-400'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Layers className="w-4 h-4" /> Artifact Store ({artifacts.length})
          </button>
          <button
            onClick={() => setActiveTab('inspector')}
            className={`py-3 px-2 text-sm font-medium border-b-2 flex items-center gap-2 transition ${
              activeTab === 'inspector'
                ? 'border-purple-500 text-purple-400'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Sliders className="w-4 h-4" /> Task Inspector
          </button>
        </div>

        {selectedWorkflow && (
          <div className="flex items-center gap-3 text-xs text-slate-400">
            <span>Goal: <span className="text-slate-200 font-medium">{selectedWorkflow.goal}</span></span>
            {getStatusBadge(selectedWorkflow.status)}
          </div>
        )}
      </div>

      {/* Main Content Area */}
      <div className="flex-1 overflow-auto p-6">
        {/* Tab 1: Task DAG Graph View */}
        {activeTab === 'dag' && (
          <div className="space-y-4">
            {tasks.length === 0 ? (
              <div className="flex flex-col items-center justify-center p-16 text-center border border-dashed border-slate-800 rounded-2xl bg-slate-900/20">
                <Network className="w-12 h-12 text-slate-600 mb-3" />
                <h3 className="text-base font-semibold text-slate-300">No Task Graph Loaded</h3>
                <p className="text-xs text-slate-500 max-w-md mt-1">
                  Click "New Workflow" to let the Supervisor AI decompose your natural language goal into a validated DAG.
                </p>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {tasks.map((task) => (
                  <div
                    key={task.id}
                    onClick={() => {
                      setSelectedTask(task);
                      setActiveTab('inspector');
                    }}
                    className={`p-4 rounded-xl border transition cursor-pointer flex flex-col justify-between ${
                      selectedTask?.id === task.id
                        ? 'bg-purple-950/20 border-purple-500/50 shadow-lg shadow-purple-500/5'
                        : 'bg-slate-900/60 border-slate-800 hover:border-slate-700'
                    }`}
                  >
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <span className="flex items-center gap-1.5 text-xs text-slate-400">
                          {getAgentIcon(task.assigned_agent)}
                          <span className="font-semibold text-slate-200">{task.assigned_agent} Agent</span>
                        </span>
                        {getStatusBadge(task.status)}
                      </div>

                      <h4 className="text-sm font-semibold text-slate-100 mb-1">{task.title}</h4>
                      <p className="text-xs text-slate-400 line-clamp-2 mb-3">{task.description}</p>
                    </div>

                    <div className="pt-3 border-t border-slate-800/80 flex items-center justify-between text-[11px] text-slate-500">
                      <span>Priority: <span className="text-slate-300 font-mono">{task.priority}</span></span>
                      <span>Retries: {task.retry_count}/{task.max_retries}</span>
                      <span className="flex items-center gap-1 text-purple-400">
                        Inspect <ArrowRight className="w-3 h-3" />
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Tab 2: Agent Registry */}
        {activeTab === 'agents' && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {agents.map((agent) => (
              <div
                key={agent.id}
                className="p-5 rounded-xl border border-slate-800 bg-slate-900/50 flex flex-col justify-between"
              >
                <div>
                  <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-2">
                      <div className="p-2 rounded-lg bg-slate-800 border border-slate-700">
                        {getAgentIcon(agent.agent_type)}
                      </div>
                      <div>
                        <h4 className="text-sm font-bold text-slate-200">{agent.display_name}</h4>
                        <span className="text-[11px] text-slate-500 font-mono">{agent.id}</span>
                      </div>
                    </div>
                    <button
                      onClick={() => handleToggleAgent(agent.id, agent.is_enabled)}
                      className={`text-xs px-2.5 py-1 rounded-full font-medium transition ${
                        agent.is_enabled
                          ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30'
                          : 'bg-slate-800 text-slate-500 border border-slate-700'
                      }`}
                    >
                      {agent.is_enabled ? 'Active' : 'Disabled'}
                    </button>
                  </div>

                  <div className="mb-4">
                    <div className="text-[11px] uppercase tracking-wider text-slate-500 font-semibold mb-1.5">
                      Capabilities
                    </div>
                    <div className="flex flex-wrap gap-1.5">
                      {agent.capabilities.map((cap) => (
                        <span
                          key={cap}
                          className="px-2 py-0.5 rounded bg-slate-800/80 text-slate-300 border border-slate-700/60 text-[11px]"
                        >
                          {cap}
                        </span>
                      ))}
                    </div>
                  </div>

                  <div>
                    <div className="text-[11px] uppercase tracking-wider text-slate-500 font-semibold mb-1.5">
                      Selective Tools ({agent.supported_tools.length})
                    </div>
                    <div className="flex flex-wrap gap-1">
                      {agent.supported_tools.slice(0, 5).map((tool) => (
                        <span key={tool} className="text-[10px] px-1.5 py-0.5 rounded bg-slate-950 text-slate-400 border border-slate-800 font-mono">
                          {tool}
                        </span>
                      ))}
                      {agent.supported_tools.length > 5 && (
                        <span className="text-[10px] text-slate-500">+{agent.supported_tools.length - 5} more</span>
                      )}
                    </div>
                  </div>
                </div>

                <div className="pt-4 mt-4 border-t border-slate-800/80 flex items-center justify-between text-xs text-slate-400">
                  <span>Max Concurrency: <span className="text-purple-400 font-mono font-bold">{agent.max_concurrency}</span></span>
                  <span className="flex items-center gap-1 text-slate-500">
                    <Shield className="w-3.5 h-3.5" /> Scoped Sandbox
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Tab 3: Artifact Store */}
        {activeTab === 'artifacts' && (
          <div className="space-y-3">
            {artifacts.length === 0 ? (
              <div className="p-12 text-center border border-dashed border-slate-800 rounded-xl text-slate-500 text-sm">
                No artifacts produced yet for this workflow.
              </div>
            ) : (
              <div className="overflow-hidden border border-slate-800 rounded-xl bg-slate-900/40">
                <table className="w-full text-left text-xs text-slate-300">
                  <thead className="bg-slate-900 border-b border-slate-800 text-slate-400 uppercase text-[10px] tracking-wider">
                    <tr>
                      <th className="py-3 px-4">Name</th>
                      <th className="py-3 px-4">Type</th>
                      <th className="py-3 px-4">Path / URI</th>
                      <th className="py-3 px-4">Hash</th>
                      <th className="py-3 px-4">Size</th>
                      <th className="py-3 px-4">Summary</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-800/60 font-mono">
                    {artifacts.map((art) => (
                      <tr key={art.id} className="hover:bg-slate-800/30 transition">
                        <td className="py-3 px-4 font-semibold text-slate-100 flex items-center gap-2">
                          <FileCode className="w-3.5 h-3.5 text-purple-400" /> {art.name}
                        </td>
                        <td className="py-3 px-4">
                          <span className="px-2 py-0.5 rounded bg-purple-500/10 text-purple-300 border border-purple-500/20 text-[10px]">
                            {art.artifact_type}
                          </span>
                        </td>
                        <td className="py-3 px-4 text-slate-400">{art.path_or_uri}</td>
                        <td className="py-3 px-4 text-slate-500">{art.hash.slice(0, 15)}...</td>
                        <td className="py-3 px-4 text-slate-400">{art.byte_size} B</td>
                        <td className="py-3 px-4 text-slate-300 max-w-xs truncate">{art.summary}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        )}

        {/* Tab 4: Task Inspector */}
        {activeTab === 'inspector' && (
          <div className="max-w-4xl mx-auto space-y-6">
            {selectedTask ? (
              <div className="bg-slate-900/60 border border-slate-800 rounded-2xl p-6 space-y-6">
                <div className="flex items-center justify-between pb-4 border-b border-slate-800">
                  <div>
                    <span className="text-xs text-purple-400 font-mono mb-1 block">Task ID: {selectedTask.id}</span>
                    <h3 className="text-lg font-bold text-slate-100">{selectedTask.title}</h3>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="flex items-center gap-1.5 text-xs text-slate-300 font-medium">
                      {getAgentIcon(selectedTask.assigned_agent)}
                      {selectedTask.assigned_agent} Agent
                    </span>
                    {getStatusBadge(selectedTask.status)}
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div className="p-4 rounded-xl bg-slate-950/60 border border-slate-800">
                    <span className="text-[11px] font-semibold text-slate-500 uppercase tracking-wider block mb-1">
                      Description
                    </span>
                    <p className="text-xs text-slate-300 leading-relaxed">{selectedTask.description}</p>
                  </div>
                  <div className="p-4 rounded-xl bg-slate-950/60 border border-slate-800">
                    <span className="text-[11px] font-semibold text-slate-500 uppercase tracking-wider block mb-1">
                      Acceptance Criteria
                    </span>
                    <p className="text-xs text-slate-300 leading-relaxed">
                      {selectedTask.acceptance_criteria || 'Deterministic validation on output'}
                    </p>
                  </div>
                </div>

                <div>
                  <span className="text-[11px] font-semibold text-slate-500 uppercase tracking-wider block mb-2">
                    Task Output Payload
                  </span>
                  <pre className="p-4 rounded-xl bg-slate-950 border border-slate-800 text-xs font-mono text-emerald-400 overflow-x-auto">
                    {selectedTask.outputs_json || '{}'}
                  </pre>
                </div>

                {selectedTask.error && (
                  <div className="p-4 rounded-xl bg-rose-950/20 border border-rose-500/30 text-rose-400 text-xs font-mono">
                    <span className="font-bold block mb-1 flex items-center gap-1.5">
                      <AlertTriangle className="w-4 h-4" /> Error Diagnostic
                    </span>
                    {selectedTask.error}
                  </div>
                )}
              </div>
            ) : (
              <div className="p-12 text-center text-slate-500 text-sm">
                Select a task from the DAG graph to inspect details.
              </div>
            )}
          </div>
        )}
      </div>

      {/* New Workflow Modal */}
      {showNewModal && (
        <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-lg p-6 shadow-2xl">
            <h3 className="text-lg font-bold text-slate-100 mb-1">Create Multi-Agent Workflow</h3>
            <p className="text-xs text-slate-400 mb-4">
              Enter your goal. The Supervisor AI will automatically generate a DAG with specialized tasks.
            </p>

            <form onSubmit={handleCreateWorkflow} className="space-y-4">
              <div>
                <label className="text-xs font-medium text-slate-300 block mb-1">Workflow Title</label>
                <input
                  type="text"
                  value={newTitle}
                  onChange={(e) => setNewTitle(e.target.value)}
                  placeholder="e.g. Build and Verify Landing Page"
                  className="w-full bg-slate-950 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-100 focus:outline-none focus:ring-1 focus:ring-purple-500"
                  required
                />
              </div>

              <div>
                <label className="text-xs font-medium text-slate-300 block mb-1">High-Level Goal</label>
                <textarea
                  value={newGoal}
                  onChange={(e) => setNewGoal(e.target.value)}
                  rows={4}
                  placeholder="e.g. Scaffold a modern web application, write components, verify cleanly in browser, and produce a synthesis report."
                  className="w-full bg-slate-950 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-100 focus:outline-none focus:ring-1 focus:ring-purple-500"
                  required
                />
              </div>

              <div className="flex justify-end gap-3 pt-3 border-t border-slate-800">
                <button
                  type="button"
                  onClick={() => setShowNewModal(false)}
                  className="px-4 py-2 rounded-lg text-xs font-medium text-slate-400 hover:text-slate-200 transition"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={loading}
                  className="px-4 py-2 rounded-lg text-xs font-medium bg-purple-600 hover:bg-purple-500 text-white transition disabled:opacity-50"
                >
                  {loading ? 'Decomposing Goal...' : 'Generate Plan'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
