export type ViewMode = 'dashboard' | 'chat' | 'tasks' | 'coding' | 'computer' | 'browser' | 'workflows' | 'business' | 'research' | 'runtime' | 'workspace' | 'terminal' | 'settings';

export type CodingMode = 'Ask' | 'Plan' | 'Build';

export type WorkflowStatus = 'Pending' | 'Planning' | 'Running' | 'WaitingApproval' | 'Pausing' | 'Paused' | 'Completed' | 'PartiallyCompleted' | 'Failed' | 'Cancelled';

export type WorkflowTaskStatus = 'Pending' | 'Ready' | 'Assigned' | 'Running' | 'WaitingApproval' | 'WaitingResource' | 'Verifying' | 'Completed' | 'Failed' | 'Cancelled' | 'Blocked';

export interface WorkflowDto {
  id: string;
  title: string;
  goal: string;
  status: WorkflowStatus;
  budget_json: string;
  result_summary?: string;
  created_at: string;
  updated_at: string;
}

export interface WorkflowTaskDto {
  id: string;
  workflow_id: string;
  parent_task_id?: string;
  title: string;
  description: string;
  assigned_agent: string;
  status: WorkflowTaskStatus;
  priority: number;
  inputs_json: string;
  outputs_json: string;
  acceptance_criteria: string;
  retry_count: number;
  max_retries: number;
  timeout_secs: number;
  started_at?: string;
  completed_at?: string;
  error?: string;
}

export interface ArtifactRefDto {
  id: string;
  workflow_id: string;
  task_id?: string;
  artifact_type: string;
  name: string;
  hash: string;
  path_or_uri: string;
  summary: string;
  byte_size: number;
  created_at: string;
}

export interface AgentDefinitionDto {
  id: string;
  agent_type: string;
  display_name: string;
  capabilities: string[];
  supported_tools: string[];
  required_permissions: string[];
  max_concurrency: number;
  is_enabled: boolean;
  model_name?: string;
}

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

export interface ResearchSourceDto {
  id: string;
  url_or_path: string;
  title: string;
  source_type: 'Web' | 'WorkspaceFile' | 'Api' | 'Document';
  retrieved_at: string;
  published_date?: string;
  content_hash: string;
  verification_status: 'Verified' | 'Unverified' | 'Contradicted' | 'Outdated';
  raw_snippet?: string;
}

export interface EvidenceItemDto {
  id: string;
  source_id: string;
  task_id?: string;
  claim_text: string;
  evidence_type: 'Fact' | 'Claim' | 'Inference' | 'Estimate' | 'Opinion';
  page_or_section?: string;
  confidence_score: number;
  verification_status: 'Verified' | 'Unverified' | 'Contradicted' | 'Outdated';
  created_at: string;
}

export interface DocumentMetadataDto {
  id: string;
  file_path: string;
  format: string;
  file_size_bytes: number;
  title?: string;
  page_count?: number;
  word_count?: number;
  table_count?: number;
  content_hash: string;
  parsed_summary?: string;
  created_at: string;
}

export interface BusinessConnectorDto {
  id: string;
  provider: string;
  auth_type: string;
  status: 'Connected' | 'Disconnected' | 'PendingAuth' | 'RateLimited' | 'Error';
  scopes: string[];
  rate_limit_per_minute: number;
  last_connected_at?: string;
  created_at: string;
}

export interface EmailDraftDto {
  id: string;
  connector_id: string;
  thread_id?: string;
  to_recipients: string[];
  cc_recipients: string[];
  bcc_recipients: string[];
  subject: string;
  body: string;
  status: 'Draft' | 'PendingApproval' | 'Approved' | 'Sent' | 'Cancelled';
  operation_id: string;
  created_at: string;
  updated_at: string;
}

export interface BusinessApprovalDto {
  id: string;
  workflow_id?: string;
  task_id?: string;
  action_type: string;
  target_resource: string;
  payload_preview: string;
  status: 'Pending' | 'Approved' | 'Rejected' | 'Expired';
  approved_by?: string;
  decision_reason?: string;
  created_at: string;
  resolved_at?: string;
}

export interface ChartSeriesDto {
  label: string;
  value: number;
}

export interface ColumnSummaryDto {
  column_name: string;
  count: number;
  null_count: number;
  sum: number;
  mean: number;
  median: number;
  min: number;
  max: number;
  std_dev: number;
}

export interface ScheduledJobDto {
  id: string;
  name: string;
  schedule_type: string;
  cron_expression?: string;
  interval_seconds?: number;
  timezone: string;
  next_run_at?: string;
  last_run_at?: string;
  status: 'Active' | 'Paused' | 'Disabled';
  missed_run_policy: 'Skip' | 'RunOnce' | 'CatchUp' | 'RequireApproval';
  overlap_policy: 'Skip' | 'Queue' | 'Replace' | 'RunConcurrently';
  max_runtime_seconds?: number;
  budget_limit?: number;
  workflow_template: string;
  payload_json: string;
  created_at: string;
  updated_at: string;
}

export interface SupervisedProcessDto {
  id: string;
  process_id: number;
  process_type: string;
  workspace_path: string;
  task_id?: string;
  start_time: string;
  last_heartbeat: string;
  exit_code?: number;
  health_status: 'Healthy' | 'Unresponsive' | 'Crashed' | 'Terminated' | 'Quarantined';
  restart_count: number;
  resource_usage_json?: string;
}

export interface SystemResourceSnapshotDto {
  cpu_usage_percent: number;
  memory_used_mb: number;
  memory_total_mb: number;
  memory_free_mb: number;
  memory_used_percent: number;
  disk_free_gb: number;
  process_count: number;
  timestamp: string;
}

export interface BudgetStatusDto {
  daily_spent: number;
  active_reserved: number;
  daily_limit: number;
  monthly_limit: number;
  is_under_budget: boolean;
}
