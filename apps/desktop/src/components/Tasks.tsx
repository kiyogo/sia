import React, { useState, useEffect } from 'react';
import { Checkpoint, Task, TaskStatus } from '../types';
import { invokeCommand } from '../lib/tauriBridge';
import {
  ListTodo,
  Clock,
  CheckCircle2,
  AlertTriangle,
  Pause,
  XCircle,
  RotateCcw,
  History,
} from 'lucide-react';

export const Tasks: React.FC = () => {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);
  const [checkpoints, setCheckpoints] = useState<Checkpoint[]>([]);
  const [filter, setFilter] = useState<string>('all');

  useEffect(() => {
    loadTasks();
  }, []);

  useEffect(() => {
    if (selectedTask) {
      loadCheckpoints(selectedTask.id);
    }
  }, [selectedTask]);

  const loadTasks = async () => {
    try {
      const list = await invokeCommand<Task[]>('list_tasks');
      setTasks(list);
      if (list.length > 0 && !selectedTask) {
        setSelectedTask(list[0]);
      }
    } catch (e) {
      console.error('Failed to load tasks', e);
    }
  };

  const loadCheckpoints = async (taskId: string) => {
    try {
      const list = await invokeCommand<Checkpoint[]>('get_task_checkpoints', { taskId });
      setCheckpoints(list);
    } catch (e) {
      console.error('Failed to load checkpoints', e);
    }
  };

  const handleCancelTask = async (taskId: string) => {
    try {
      await invokeCommand('cancel_task', { taskId });
      loadTasks();
    } catch (e) {
      console.error('Failed to cancel task', e);
    }
  };

  const handleRecoverTasks = async () => {
    try {
      await invokeCommand('recover_tasks');
      loadTasks();
    } catch (e) {
      console.error('Failed to recover tasks', e);
    }
  };

  const getStatusBadge = (status: TaskStatus) => {
    switch (status) {
      case 'Running':
        return (
          <span className="px-2 py-0.5 text-[10px] font-semibold rounded-full bg-blue-500/20 text-blue-300 border border-blue-500/40 flex items-center gap-1">
            <Clock className="w-2.5 h-2.5 animate-spin" />
            Running
          </span>
        );
      case 'Completed':
        return (
          <span className="px-2 py-0.5 text-[10px] font-semibold rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 flex items-center gap-1">
            <CheckCircle2 className="w-2.5 h-2.5" />
            Completed
          </span>
        );
      case 'WaitingApproval':
        return (
          <span className="px-2 py-0.5 text-[10px] font-semibold rounded-full bg-amber-500/20 text-amber-300 border border-amber-500/40 flex items-center gap-1">
            <AlertTriangle className="w-2.5 h-2.5" />
            Waiting Approval
          </span>
        );
      case 'Paused':
        return (
          <span className="px-2 py-0.5 text-[10px] font-semibold rounded-full bg-slate-500/20 text-slate-300 border border-slate-500/40 flex items-center gap-1">
            <Pause className="w-2.5 h-2.5" />
            Paused
          </span>
        );
      case 'Failed':
        return (
          <span className="px-2 py-0.5 text-[10px] font-semibold rounded-full bg-rose-500/20 text-rose-300 border border-rose-500/40 flex items-center gap-1">
            <AlertTriangle className="w-2.5 h-2.5" />
            Failed
          </span>
        );
      case 'Cancelled':
        return (
          <span className="px-2 py-0.5 text-[10px] font-semibold rounded-full bg-slate-700/40 text-slate-400 border border-slate-700">
            Cancelled
          </span>
        );
      default:
        return (
          <span className="px-2 py-0.5 text-[10px] font-semibold rounded-full bg-slate-800 text-slate-300 border border-slate-700">
            Pending
          </span>
        );
    }
  };

  const filteredTasks = tasks.filter((t) => {
    if (filter === 'all') return true;
    return t.status.toLowerCase() === filter.toLowerCase();
  });

  return (
    <div className="flex-1 flex h-full overflow-hidden">
      {/* Task List Column */}
      <div className="w-80 border-r border-slate-800 bg-slate-950/70 flex flex-col justify-between">
        <div className="p-4 border-b border-slate-800 space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="font-semibold text-sm text-slate-200">Execution Tasks</h3>
            <button
              onClick={handleRecoverTasks}
              title="Recover paused tasks from checkpoint"
              className="text-xs text-indigo-400 hover:text-indigo-300 flex items-center gap-1"
            >
              <RotateCcw className="w-3.5 h-3.5" />
              <span>Recover</span>
            </button>
          </div>

          {/* Filter Bar */}
          <div className="flex gap-1 bg-slate-900 p-1 rounded-lg text-xs">
            {['all', 'running', 'completed', 'failed'].map((f) => (
              <button
                key={f}
                onClick={() => setFilter(f)}
                className={`flex-1 py-1 rounded capitalize font-medium transition ${
                  filter === f
                    ? 'bg-slate-800 text-indigo-300 shadow-sm'
                    : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                {f}
              </button>
            ))}
          </div>
        </div>

        {/* Task Items */}
        <div className="flex-1 overflow-y-auto p-2 space-y-1.5">
          {filteredTasks.length === 0 ? (
            <div className="p-8 text-center text-xs text-slate-500">No tasks found</div>
          ) : (
            filteredTasks.map((t) => {
              const isSelected = selectedTask?.id === t.id;
              return (
                <div
                  key={t.id}
                  onClick={() => setSelectedTask(t)}
                  className={`p-3 rounded-xl border text-xs cursor-pointer transition ${
                    isSelected
                      ? 'bg-slate-900 border-indigo-500/40 shadow-sm'
                      : 'bg-slate-950 border-slate-800/80 hover:bg-slate-900/60'
                  }`}
                >
                  <div className="flex items-start justify-between gap-2 mb-1.5">
                    <span className="font-medium text-slate-200 line-clamp-1">{t.title}</span>
                    {getStatusBadge(t.status)}
                  </div>
                  <div className="text-[11px] text-slate-400 line-clamp-1">{t.current_step}</div>
                  <div className="text-[10px] text-slate-400 mt-2">
                    {new Date(t.created_at).toLocaleString()}
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>

      {/* Task Details & Checkpoints Inspector */}
      <div className="flex-1 flex flex-col bg-slate-950 overflow-y-auto p-8">
        {selectedTask ? (
          <div className="space-y-6 max-w-4xl">
            {/* Header info */}
            <div className="flex items-start justify-between pb-6 border-b border-slate-800">
              <div>
                <div className="flex items-center gap-3">
                  <h2 className="text-xl font-bold text-slate-100">{selectedTask.title}</h2>
                  {getStatusBadge(selectedTask.status)}
                </div>
                <p className="text-xs text-slate-400 mt-1 font-mono">ID: {selectedTask.id}</p>
              </div>

              {selectedTask.status === 'Running' && (
                <button
                  onClick={() => handleCancelTask(selectedTask.id)}
                  className="px-3.5 py-1.5 rounded-lg bg-rose-600 hover:bg-rose-500 text-white font-medium text-xs flex items-center gap-1.5 shadow transition"
                >
                  <XCircle className="w-4 h-4" />
                  <span>Cancel Task</span>
                </button>
              )}
            </div>

            {/* Goal & Current Step */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 space-y-1">
                <span className="text-xs text-slate-400 font-medium">Assigned Goal</span>
                <p className="text-xs text-slate-200 font-mono">{selectedTask.goal}</p>
              </div>
              <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 space-y-1">
                <span className="text-xs text-slate-400 font-medium">Current Step</span>
                <p className="text-xs text-slate-200 font-mono">{selectedTask.current_step}</p>
              </div>
            </div>

            {/* Final Result / Error */}
            {selectedTask.result && (
              <div className="p-4 rounded-xl bg-emerald-950/20 border border-emerald-800/40 space-y-2">
                <span className="text-xs text-emerald-400 font-semibold flex items-center gap-1.5">
                  <CheckCircle2 className="w-4 h-4" />
                  Execution Result
                </span>
                <div className="p-3 bg-slate-950 rounded border border-slate-800 font-mono text-xs text-slate-200 whitespace-pre-wrap select-text">
                  {selectedTask.result}
                </div>
              </div>
            )}

            {selectedTask.error && (
              <div className="p-4 rounded-xl bg-rose-950/20 border border-rose-800/40 space-y-2">
                <span className="text-xs text-rose-400 font-semibold flex items-center gap-1.5">
                  <AlertTriangle className="w-4 h-4" />
                  Execution Error
                </span>
                <div className="p-3 bg-slate-950 rounded border border-slate-800 font-mono text-xs text-rose-300 whitespace-pre-wrap select-text">
                  {selectedTask.error}
                </div>
              </div>
            )}

            {/* Checkpoint Timeline */}
            <div className="space-y-3 pt-4">
              <h4 className="text-xs font-semibold uppercase tracking-wider text-slate-400 flex items-center gap-2">
                <History className="w-4 h-4 text-indigo-400" />
                <span>Persistence Checkpoints & Milestones</span>
              </h4>

              {checkpoints.length === 0 ? (
                <div className="p-4 rounded-xl bg-slate-900/30 border border-slate-800 text-xs text-slate-500 text-center">
                  No step checkpoints saved for this task yet.
                </div>
              ) : (
                <div className="space-y-2">
                  {checkpoints.map((ck) => (
                    <div
                      key={ck.id}
                      className="p-3.5 rounded-xl bg-slate-900/60 border border-slate-800 flex items-start justify-between text-xs"
                    >
                      <div className="space-y-1">
                        <div className="font-semibold text-slate-200">
                          Step #{ck.step_index}: {ck.summary}
                        </div>
                        <div className="text-[10px] text-slate-400 font-mono">
                          Saved: {new Date(ck.created_at).toLocaleString()}
                        </div>
                      </div>
                      <span className="px-2 py-0.5 rounded bg-indigo-500/10 text-indigo-300 text-[10px] font-mono border border-indigo-500/20">
                        Valid Checkpoint
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        ) : (
          <div className="h-full flex flex-col items-center justify-center text-slate-500 text-xs">
            <ListTodo className="w-8 h-8 text-slate-600 mb-2" />
            <span>Select a task to inspect details and checkpoints</span>
          </div>
        )}
      </div>
    </div>
  );
};
