export type ViewMode = 'dashboard' | 'chat' | 'tasks' | 'coding' | 'computer' | 'browser' | 'workspace' | 'terminal' | 'settings';

export type CodingMode = 'Ask' | 'Plan' | 'Build';

export interface WindowInfo {
  id: string;
  title: string;
  process_name: string;
  is_active: boolean;
}

export interface OwnershipStatusReport {
  current_controller: 'None' | 'User' | 'Agent';
  active_lease?: {
    id: string;
    agent_name: string;
    expires_at: string;
    is_active: boolean;
  };
  remaining_secs: number;
  is_emergency_stopped: boolean;
}

export interface BrowserTabInfo {
  id: string;
  title: string;
  url: string;
  is_active: boolean;
}

export interface DiagnosticMessage {
  file: string;
  line: number;
  column: number;
  severity: 'Error' | 'Warning' | 'Info';
  message: string;
  code?: string;
  context_snippet?: string;
}

export interface BuildResult {
  command: string;
  success: boolean;
  exit_code?: number;
  duration_ms: number;
  total_errors: number;
  total_warnings: number;
  root_cause?: DiagnosticMessage;
  diagnostics: DiagnosticMessage[];
  stdout: string;
  stderr: string;
}

export interface ProjectInfo {
  root_path: string;
  primary_language: string | null;
  languages: string[];
  build_systems: string[];
  test_runners: string[];
  entry_points: string[];
  is_git: boolean;
  git_branch: string | null;
}

export type TaskStatus = 'Pending' | 'Running' | 'WaitingApproval' | 'Paused' | 'Completed' | 'Failed' | 'Cancelled';

export type PermissionTier = 'ReadOnly' | 'WorkspaceWrite' | 'ShellExecution' | 'BrowserAutomation' | 'DesktopControl' | 'DestructiveAction' | 'ExternalSideEffect';

export type PermissionMode = 'Strict' | 'WorkspaceAuto' | 'FullAuto';

export interface DashboardStats {
  runtime_active: boolean;
  provider_connected: boolean;
  provider_name: string;
  model_name: string;
  windows_mcp_connected: boolean;
  windows_mcp_prerequisites: boolean;
  windows_mcp_hint: string;
  browser_connected: boolean;
  desktop_controller: string;
  desktop_remaining_secs: number;
  is_emergency_stopped: boolean;
  tasks_running: number;
  tasks_completed: number;
  tasks_failed: number;
  total_conversations: number;
  pending_approvals: number;
  workspace_path: string;
  permission_mode: string;
}

export interface PrerequisiteStatus {
  name: string;
  installed: boolean;
  version: string | null;
  required_version: string;
  install_hint: string;
}

export interface Conversation {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  token_in: number;
  token_out: number;
}

export interface Message {
  id: string;
  conversation_id: string;
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  tool_calls: any | null;
  tool_call_id: string | null;
  token_count: number;
  created_at: string;
}

export interface Task {
  id: string;
  conversation_id: string | null;
  title: string;
  status: TaskStatus;
  goal: string;
  current_step: string;
  result: string | null;
  error: string | null;
  created_at: string;
  updated_at: string;
}

export interface Checkpoint {
  id: string;
  task_id: string;
  step_index: number;
  state_json: string;
  summary: string;
  created_at: string;
}

export interface RecoveredTaskInfo {
  task_id: string;
  title: string;
  previous_status: TaskStatus;
  new_status: TaskStatus;
  checkpoint_step: number | null;
  checkpoint_summary: string | null;
}

export interface WorkspaceFileInfo {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
}

export interface ProcessOutput {
  stdout: string;
  stderr: string;
  exit_code: number | null;
  duration_ms: number;
  truncated: boolean;
}

export interface ApprovalRequest {
  id: string;
  tier: PermissionTier;
  description: string;
  details: any;
  status: 'Pending' | 'Approved' | 'Rejected';
  requested_at: string;
  resolved_at: string | null;
}

export interface RuntimeConfig {
  provider_url: string;
  api_key: string;
  model_name: string;
  workspace_path: string;
  permission_mode: PermissionMode;
  max_loops: number;
  budget_config: {
    max_context_tokens: number;
    system_budget: number;
    history_budget: number;
    tool_result_budget: number;
    completion_reserve: number;
    max_tool_output_lines: number;
    max_tool_output_bytes: number;
  };
}

export type AgentEvent =
  | { StepStarted: { step: number; max_steps: number } }
  | { ModelThinking: { preview: string } }
  | { ToolCallStarted: { tool_name: string; call_id: string; arguments: string } }
  | { ToolCallFinished: { tool_name: string; call_id: string; exit_code: number | null; duration_ms: number; is_error: boolean; truncated: boolean } }
  | { TokenUsageUpdated: { prompt_tokens: number; completion_tokens: number; total_tokens: number } }
  | { CheckpointSaved: { step: number; summary: string } }
  | { FinalAnswer: { content: string } }
  | { Error: { message: string } }
  | { TaskStatusChanged: { status: TaskStatus } };
