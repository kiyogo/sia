// Check if running inside Tauri WebView
export const isTauri = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

// Safe invoke wrapper
export async function invokeCommand<T>(cmd: string, args?: Record<string, any>): Promise<T> {
  if (isTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(cmd, args);
  }

  // Fallback dev mode when previewed in regular browser
  console.warn(`[Tauri Mock] Invoke: ${cmd}`, args);
  return getMockResponse<T>(cmd);
}

// Safe event listener wrapper
export async function listenToEvent<T>(
  event: string,
  callback: (payload: T) => void
): Promise<() => void> {
  if (isTauri()) {
    const { listen } = await import('@tauri-apps/api/event');
    const unlisten = await listen<T>(event, (e) => callback(e.payload));
    return unlisten;
  }
  return () => {};
}

// Fallback mocks for UI preview outside Tauri
function getMockResponse<T>(cmd: string): T {
  switch (cmd) {
    case 'get_dashboard_stats':
      return {
        runtime_active: true,
        provider_connected: true,
        provider_name: 'https://api.openai.com/v1',
        model_name: 'gpt-4o',
        windows_mcp_connected: false,
        windows_mcp_prerequisites: true,
        windows_mcp_hint: 'Python 3.14+ and uv ready.',
        tasks_running: 1,
        tasks_completed: 4,
        tasks_failed: 0,
        total_conversations: 2,
        pending_approvals: 0,
        workspace_path: 'C:\\workspace\\test_project',
        permission_mode: 'WorkspaceAuto',
      } as unknown as T;

    case 'list_conversations':
      return [
        {
          id: 'c1',
          title: 'Initial Setup Conversation',
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          token_in: 1200,
          token_out: 450,
        },
      ] as unknown as T;

    case 'list_tasks':
      return [
        {
          id: 't1',
          conversation_id: 'c1',
          title: 'Compile and test workspace',
          status: 'Running',
          goal: 'Build application and run unit tests',
          current_step: 'Running cargo test',
          result: null,
          error: null,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
      ] as unknown as T;

    case 'list_workspace_files':
      return [
        { name: 'src', path: 'src', is_dir: true, size: 0 },
        { name: 'Cargo.toml', path: 'Cargo.toml', is_dir: false, size: 450 },
        { name: 'README.md', path: 'README.md', is_dir: false, size: 1200 },
      ] as unknown as T;

    case 'read_workspace_file':
      return '# Workspace Project\nWelcome to your universal autonomous workspace.' as unknown as T;

    case 'execute_terminal_command':
      return {
        stdout: 'Rust workspace initialized successfully.\nTarget: x86_64-pc-windows-msvc',
        stderr: '',
        exit_code: 0,
        duration_ms: 120,
        truncated: false,
      } as unknown as T;

    case 'get_runtime_config':
      return {
        provider_url: 'https://api.openai.com/v1',
        api_key: 'sk-mock...',
        model_name: 'gpt-4o',
        workspace_path: 'C:\\workspace\\demo',
        permission_mode: 'WorkspaceAuto',
        max_loops: 25,
        budget_config: {
          max_context_tokens: 32000,
          system_budget: 2500,
          history_budget: 18000,
          tool_result_budget: 7500,
          completion_reserve: 4000,
          max_tool_output_lines: 150,
          max_tool_output_bytes: 16384,
        },
      } as unknown as T;

    default:
      return {} as T;
  }
}
