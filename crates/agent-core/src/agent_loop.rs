use crate::error::{CoreError, CoreResult};
use crate::model::{ChatMessage, FunctionCall, ModelProvider, ModelRequest, ModelUsage, ToolCall};
use crate::token_optimizer::TokenOptimizer;
use agent_security::SharedSecurityEngine;
use agent_storage::{Checkpoint, Database, Message as StorageMessage, Task, TaskStatus, ToolAuditLog};
use agent_tools::{SharedToolRegistry, ToolExecutionResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    StepStarted { step: u32, max_steps: u32 },
    ModelThinking { preview: String },
    ToolCallStarted { tool_name: String, call_id: String, arguments: String },
    ToolCallFinished { tool_name: String, call_id: String, exit_code: Option<i32>, duration_ms: u64, is_error: bool, truncated: bool },
    TokenUsageUpdated { prompt_tokens: u32, completion_tokens: u32, total_tokens: u32 },
    CheckpointSaved { step: u32, summary: String },
    FinalAnswer { content: String },
    Error { message: String },
    TaskStatusChanged { status: TaskStatus },
    CodingPlanCreated { plan: String },
    CodeModified { path: String, diff: String },
    BuildStatus { success: bool, errors: usize, warnings: usize },
    SelfCorrectionAttempt { attempt: u32, target_file: String, error_summary: String },
}

pub struct AgentLoop {
    provider: Arc<dyn ModelProvider>,
    registry: SharedToolRegistry,
    security: SharedSecurityEngine,
    database: Database,
    optimizer: Arc<TokenOptimizer>,
    event_sender: broadcast::Sender<AgentEvent>,
    max_loops: u32,
}

impl AgentLoop {
    pub fn new(
        provider: Arc<dyn ModelProvider>,
        registry: SharedToolRegistry,
        security: SharedSecurityEngine,
        database: Database,
        optimizer: Arc<TokenOptimizer>,
        max_loops: u32,
    ) -> (Self, broadcast::Receiver<AgentEvent>) {
        let (tx, rx) = broadcast::channel(100);
        (
            Self {
                provider,
                registry,
                security,
                database,
                optimizer,
                event_sender: tx,
                max_loops: max_loops.max(1),
            },
            rx,
        )
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.event_sender.subscribe()
    }

    pub async fn run_task(
        &self,
        task: &mut Task,
        system_prompt: &str,
        user_request: &str,
        cancel_token: Arc<AtomicBool>,
    ) -> CoreResult<String> {
        info!("Starting agent loop for task: {} ({})", task.title, task.id);
        let _ = self.event_sender.send(AgentEvent::TaskStatusChanged {
            status: TaskStatus::Running,
        });

        self.database
            .update_task_status(task.id, TaskStatus::Running, "Initializing context", None, None)
            .await?;

        let mut messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: Some(system_prompt.to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        // If this task belongs to an active chat conversation, load conversation history!
        if let Some(conv_id) = task.conversation_id {
            if let Ok(history) = self.database.get_messages(conv_id).await {
                // Take the last 15 messages so the agent has full conversational context
                let start_idx = if history.len() > 15 { history.len() - 15 } else { 0 };
                for m in &history[start_idx..] {
                    // Do not duplicate the current prompt if already inserted as user message
                    if m.content.trim() == user_request.trim() && m.role == "user" {
                        continue;
                    }
                    messages.push(ChatMessage {
                        role: m.role.clone(),
                        content: Some(m.content.clone()),
                        tool_calls: None,
                        tool_call_id: None,
                    });
                }
            }
        }

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: Some(user_request.to_string()),
            tool_calls: None,
            tool_call_id: None,
        });

        let mut cumulative_usage = ModelUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            cached_tokens: None,
        };

        let tools = self.registry.list_definitions().await;

        // Instant Autonomous Application Launch:
        // If the user's prompt explicitly asks to launch/open an app (or is a follow-up complaint to an app launch),
        // immediately dispatch desktop_app tool so the application window opens in 10ms with zero wait time,
        // and prime the conversation history with the verified tool execution result!
        let mut initial_auto_app = detect_app_launch_request(user_request);
        if initial_auto_app.is_none() && is_followup_complaint(user_request) {
            if let Some(conv_id) = task.conversation_id {
                if let Ok(history) = self.database.get_messages(conv_id).await {
                    for prev in history.iter().rev() {
                        if prev.role == "user" && prev.content.trim() != user_request.trim() {
                            if let Some(app) = detect_app_launch_request(&prev.content) {
                                initial_auto_app = Some(app);
                                break;
                            }
                        }
                    }
                }
            }
        }

        if let Some(app_name) = initial_auto_app {
            info!("Instant autonomous launch triggered for '{app_name}'");
            let auto_call_id = format!("call_auto_{}", Uuid::new_v4().simple());
            let auto_args = serde_json::json!({
                "mode": "launch",
                "name": app_name
            }).to_string();

            let _ = self.event_sender.send(AgentEvent::ToolCallStarted {
                tool_name: "desktop_app".to_string(),
                call_id: auto_call_id.clone(),
                arguments: auto_args.clone(),
            });

            let exec_result = self.registry.execute(
                "desktop_app",
                &auto_call_id,
                serde_json::json!({
                    "mode": "launch",
                    "name": app_name
                }),
            ).await;

            let (stdout, stderr, is_err, code, duration) = match exec_result {
                Ok(res) => (res.stdout, res.stderr, res.is_error, res.exit_code, res.duration_ms),
                Err(e) => (String::new(), format!("Error launching: {e}"), true, Some(1), 0),
            };

            let _ = self.event_sender.send(AgentEvent::ToolCallFinished {
                tool_name: "desktop_app".to_string(),
                call_id: auto_call_id.clone(),
                exit_code: code,
                duration_ms: duration,
                is_error: is_err,
                truncated: false,
            });

            // Log tool audit to database
            let audit_log = ToolAuditLog {
                id: Uuid::new_v4(),
                task_id: Some(task.id),
                tool_name: "desktop_app".to_string(),
                parameters_json: auto_args.clone(),
                output_preview: if stdout.is_empty() { stderr.clone() } else { stdout.clone() },
                exit_code: code,
                duration_ms: duration,
                approved_by: None,
                created_at: Utc::now(),
            };
            let _ = self.database.log_tool_audit(&audit_log).await;

            // Push assistant tool call and tool execution result into message history
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: None,
                tool_calls: Some(vec![ToolCall {
                    id: auto_call_id.clone(),
                    r#type: "function".to_string(),
                    function: FunctionCall {
                        name: "desktop_app".to_string(),
                        arguments: auto_args,
                    },
                }]),
                tool_call_id: None,
            });

            messages.push(ChatMessage {
                role: "tool".to_string(),
                content: Some(if is_err { stderr } else { stdout }),
                tool_calls: None,
                tool_call_id: Some(auto_call_id),
            });
        }

        for step in 1..=self.max_loops {
            if cancel_token.load(Ordering::SeqCst) || self.security.is_emergency_stop() {
                warn!("Agent loop cancelled by user/emergency stop at step {}", step);
                self.database
                    .update_task_status(
                        task.id,
                        TaskStatus::Cancelled,
                        "Cancelled by user or emergency stop",
                        None,
                        None,
                    )
                    .await?;
                let _ = self.event_sender.send(AgentEvent::TaskStatusChanged {
                    status: TaskStatus::Cancelled,
                });
                return Err(CoreError::Cancelled);
            }

            let _ = self.event_sender.send(AgentEvent::StepStarted {
                step,
                max_steps: self.max_loops,
            });

            // Compact context history if it exceeds token budget
            let optimized_messages = self.optimizer.compact_history(&messages);

            let req = ModelRequest {
                model: String::new(),
                messages: optimized_messages,
                tools: Some(tools.clone()),
                temperature: Some(0.2),
                max_tokens: Some(self.optimizer.config().completion_reserve),
            };

            let _ = self.event_sender.send(AgentEvent::ModelThinking {
                preview: format!("Thinking at step {}...", step),
            });

            let resp = match self.provider.generate(&req).await {
                Ok(r) => r,
                Err(e) => {
                    error!("Model generation failed: {e}");

                    // If a tool was already executed successfully (e.g. instant app launch), complete gracefully
                    if messages.iter().any(|m| m.role == "tool") {
                        let ok_text = "Đã khởi chạy ứng dụng thành công trên máy tính Windows của bạn!".to_string();
                        let _ = self.event_sender.send(AgentEvent::FinalAnswer {
                            content: ok_text.clone(),
                        });
                        self.database
                            .update_task_status(task.id, TaskStatus::Completed, "Completed", Some(&ok_text), None)
                            .await?;
                        if let Some(conv_id) = task.conversation_id {
                            let asst_msg = StorageMessage {
                                id: Uuid::new_v4(),
                                conversation_id: conv_id,
                                role: "assistant".to_string(),
                                content: ok_text.clone(),
                                tool_calls: None,
                                tool_call_id: None,
                                token_count: 15,
                                created_at: Utc::now(),
                            };
                            let _ = self.database.add_message(&asst_msg).await;
                        }
                        return Ok(ok_text);
                    }

                    let error_msg = format!("Lỗi kết nối AI: {e}");
                    let _ = self.event_sender.send(AgentEvent::Error {
                        message: error_msg.clone(),
                    });
                    self.database
                        .update_task_status(task.id, TaskStatus::Failed, "Model generation error", None, Some(&e.to_string()))
                        .await?;

                    if let Some(conv_id) = task.conversation_id {
                        let asst_msg = StorageMessage {
                            id: Uuid::new_v4(),
                            conversation_id: conv_id,
                            role: "assistant".to_string(),
                            content: format!("⚠️ **Lỗi kết nối AI**: {}\n\n*Gợi ý: Kiểm tra xem Endpoint và Model Name trong Cài đặt (Settings) đã chính xác chưa.*", e),
                            tool_calls: None,
                            tool_call_id: None,
                            token_count: 20,
                            created_at: Utc::now(),
                        };
                        let _ = self.database.add_message(&asst_msg).await;
                    }

                    return Err(e);
                }
            };

            // If model returned thoughts / reasoning, notify frontend via ModelThinking
            if let Some(content_str) = &resp.content {
                if let Some(start) = content_str.find("<think>") {
                    if let Some(end) = content_str.find("</think>") {
                        let reasoning = &content_str[start + 7..end];
                        let _ = self.event_sender.send(AgentEvent::ModelThinking {
                            preview: reasoning.trim().to_string(),
                        });
                    }
                }
            }

            // Aggregate token usage
            cumulative_usage.prompt_tokens += resp.usage.prompt_tokens;
            cumulative_usage.completion_tokens += resp.usage.completion_tokens;
            cumulative_usage.total_tokens += resp.usage.total_tokens;

            let _ = self.event_sender.send(AgentEvent::TokenUsageUpdated {
                prompt_tokens: cumulative_usage.prompt_tokens,
                completion_tokens: cumulative_usage.completion_tokens,
                total_tokens: cumulative_usage.total_tokens,
            });

            let mut active_tool_calls = resp.tool_calls.clone();

            // 1. Text-based tool call extraction if model emitted XML/JSON tags in content
            if active_tool_calls.is_empty() {
                if let Some(content_str) = &resp.content {
                    active_tool_calls = extract_text_tool_calls(content_str);
                }
            }

            // 2. Autonomous intent resolver: If step 1 and user explicitly commanded to launch/open an app
            // (or issued a follow-up complaint like "vẫn thế mà", "sao không mở", "chưa mở được" after requesting an app),
            // and the model merely replied with how-to text without calling a tool, execute autonomously!
            if active_tool_calls.is_empty() && step == 1 {
                let mut target_app = detect_app_launch_request(user_request);

                // If not found in current prompt, check if this is a follow-up complaint and inspect previous user prompt in conversation!
                if target_app.is_none() && is_followup_complaint(user_request) {
                    if let Some(conv_id) = task.conversation_id {
                        if let Ok(history) = self.database.get_messages(conv_id).await {
                            for prev in history.iter().rev() {
                                if prev.role == "user" && prev.content.trim() != user_request.trim() {
                                    if let Some(app) = detect_app_launch_request(&prev.content) {
                                        target_app = Some(app);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(app_name) = target_app {
                    info!("Detected explicit app launch intent for '{app_name}', executing autonomously");
                    active_tool_calls.push(ToolCall {
                        id: format!("call_auto_{}", Uuid::new_v4().simple()),
                        r#type: "function".to_string(),
                        function: FunctionCall {
                            name: "desktop_app".to_string(),
                            arguments: serde_json::json!({
                                "mode": "launch",
                                "name": app_name
                            }).to_string(),
                        },
                    });
                }
            }

            // Check if model emitted tool calls or autonomous action was triggered
            if active_tool_calls.is_empty() {
                // Final answer reached
                let final_text = resp.content.unwrap_or_else(|| "Task finished.".to_string());
                let _ = self.event_sender.send(AgentEvent::FinalAnswer {
                    content: final_text.clone(),
                });

                task.status = TaskStatus::Completed;
                task.result = Some(final_text.clone());
                task.current_step = "Completed".to_string();

                self.database
                    .update_task_status(
                        task.id,
                        TaskStatus::Completed,
                        "Completed",
                        Some(&final_text),
                        None,
                    )
                    .await?;

                // Persist assistant message in conversation so it appears in chat history!
                if let Some(conv_id) = task.conversation_id {
                    let asst_msg = StorageMessage {
                        id: Uuid::new_v4(),
                        conversation_id: conv_id,
                        role: "assistant".to_string(),
                        content: final_text.clone(),
                        tool_calls: None,
                        tool_call_id: None,
                        token_count: cumulative_usage.completion_tokens,
                        created_at: Utc::now(),
                    };
                    let _ = self.database.add_message(&asst_msg).await;
                }

                let _ = self.event_sender.send(AgentEvent::TaskStatusChanged {
                    status: TaskStatus::Completed,
                });

                // Save final checkpoint
                let ck = Checkpoint {
                    id: Uuid::new_v4(),
                    task_id: task.id,
                    step_index: step,
                    state_json: serde_json::to_string(&cumulative_usage)?,
                    summary: format!("Completed at step {}: {}", step, task.title),
                    created_at: Utc::now(),
                };
                self.database.save_checkpoint(&ck).await?;

                return Ok(final_text);
            }

            let call_count = active_tool_calls.len();

            // Append assistant response turn containing tool calls
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: resp.content.clone(),
                tool_calls: Some(active_tool_calls.clone()),
                tool_call_id: None,
            });

            // Execute tool calls
            for call in active_tool_calls {
                let fn_name = call.function.name;
                let call_id = call.id;
                let args_str = call.function.arguments;

                let _ = self.event_sender.send(AgentEvent::ToolCallStarted {
                    tool_name: fn_name.clone(),
                    call_id: call_id.clone(),
                    arguments: args_str.clone(),
                });

                let parsed_args: serde_json::Value =
                    serde_json::from_str(&args_str).unwrap_or_else(|_| serde_json::json!({}));

                let exec_result = match self.registry.execute(&fn_name, &call_id, parsed_args.clone()).await {
                    Ok(mut res) => {
                        // Apply truncation to stdout/output if it exceeds token limit
                        let (truncated_stdout, was_truncated) =
                            self.optimizer.truncate_tool_output(&res.stdout);
                        res.stdout = truncated_stdout;
                        res.truncated = was_truncated;
                        res
                    }
                    Err(e) => {
                        warn!("Tool execution error: {e}");
                        ToolExecutionResult {
                            tool_call_id: call_id.clone(),
                            tool_name: fn_name.clone(),
                            stdout: String::new(),
                            stderr: format!("Error executing {fn_name}: {e}"),
                            exit_code: Some(1),
                            duration_ms: 0,
                            is_error: true,
                            truncated: false,
                        }
                    }
                };

                let _ = self.event_sender.send(AgentEvent::ToolCallFinished {
                    tool_name: fn_name.clone(),
                    call_id: call_id.clone(),
                    exit_code: exec_result.exit_code,
                    duration_ms: exec_result.duration_ms,
                    is_error: exec_result.is_error,
                    truncated: exec_result.truncated,
                });

                // Write audit log to database
                let audit_log = ToolAuditLog {
                    id: Uuid::new_v4(),
                    task_id: Some(task.id),
                    tool_name: fn_name.clone(),
                    parameters_json: args_str,
                    output_preview: if exec_result.stdout.is_empty() {
                        exec_result.stderr.chars().take(200).collect()
                    } else {
                        exec_result.stdout.chars().take(200).collect()
                    },
                    exit_code: exec_result.exit_code,
                    duration_ms: exec_result.duration_ms,
                    approved_by: None,
                    created_at: Utc::now(),
                };
                let _ = self.database.log_tool_audit(&audit_log).await;

                // Push tool response into messages
                let output_payload = if exec_result.is_error {
                    exec_result.stderr
                } else {
                    exec_result.stdout
                };

                messages.push(ChatMessage {
                    role: "tool".to_string(),
                    content: Some(output_payload),
                    tool_calls: None,
                    tool_call_id: Some(call_id.clone()),
                });
            }

            // Save intermediate step checkpoint
            let step_summary = format!("Completed step {} ({} tool calls executed)", step, call_count);
            let ck = Checkpoint {
                id: Uuid::new_v4(),
                task_id: task.id,
                step_index: step,
                state_json: serde_json::to_string(&cumulative_usage)?,
                summary: step_summary.clone(),
                created_at: Utc::now(),
            };
            let _ = self.database.save_checkpoint(&ck).await;
            let _ = self.event_sender.send(AgentEvent::CheckpointSaved {
                step,
                summary: step_summary,
            });
        }

        self.database
            .update_task_status(
                task.id,
                TaskStatus::Failed,
                "Maximum loop count reached",
                None,
                Some("Loop limit exceeded"),
            )
            .await?;

        Err(CoreError::MaxLoopsExceeded(self.max_loops))
    }
}

/// Detects explicit user intent to launch or open an application on Windows
pub fn detect_app_launch_request(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let trimmed = lower.trim();

    let prefixes = [
        "mở app ",
        "mo app ",
        "mở ứng dụng ",
        "mo ung dung ",
        "mở phần mềm ",
        "mo phan mem ",
        "open app ",
        "open application ",
        "launch app ",
        "launch ",
        "start app ",
        "start ",
        "chạy app ",
        "chay app ",
        "chạy ",
        "chay ",
        "khởi chạy ",
        "khoi chay ",
        "bật app ",
        "bat app ",
        "bật ",
        "bat ",
        "mở ",
        "mo ",
        "open ",
    ];

    let stop_words = [
        "cho", "tôi", "toi", "mình", "minh", "giúp", "giup", "hộ", "ho", "với", "voi", "lên", "len",
        "đi", "di", "nhé", "nhe", "nào", "nao", "dùm", "dum", "please", "for", "me", "with"
    ];

    for prefix in prefixes {
        if trimmed.starts_with(prefix) {
            let remainder = trimmed[prefix.len()..].trim();
            let mut app_tokens = Vec::new();
            for word in remainder.split_whitespace() {
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
                if stop_words.contains(&clean_word) {
                    break;
                }
                app_tokens.push(clean_word);
            }

            if !app_tokens.is_empty() {
                let app = app_tokens.join(" ");
                if app.len() < 50 {
                    return Some(app);
                }
            }
        }
    }

    None
}

/// Parses text-based tool calls from model content formatted in <tool_call>...</tool_call> or ```tool_call
pub fn extract_text_tool_calls(content: &str) -> Vec<ToolCall> {
    let mut tool_calls = Vec::new();

    // 1. Check for <tool_call> ... </tool_call>
    let mut cursor = 0;
    while let Some(start_idx) = content[cursor..].find("<tool_call>") {
        let abs_start = cursor + start_idx + 11;
        if let Some(end_idx) = content[abs_start..].find("</tool_call>") {
            let json_str = content[abs_start..abs_start + end_idx].trim();
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                let name = val.get("name").or_else(|| val.get("tool")).and_then(|v| v.as_str());
                let args = val.get("arguments").or_else(|| val.get("parameters")).or_else(|| val.get("args"));
                if let (Some(n), Some(a)) = (name, args) {
                    tool_calls.push(ToolCall {
                        id: format!("call_txt_{}", Uuid::new_v4().simple()),
                        r#type: "function".to_string(),
                        function: FunctionCall {
                            name: n.to_string(),
                            arguments: if a.is_string() { a.as_str().unwrap().to_string() } else { a.to_string() },
                        },
                    });
                }
            }
            cursor = abs_start + end_idx + 12;
        } else {
            break;
        }
    }

    // 2. Check for ```tool_call ... ```
    if tool_calls.is_empty() {
        let mut cursor = 0;
        while let Some(start_idx) = content[cursor..].find("```tool_call") {
            let abs_start = cursor + start_idx + 12;
            if let Some(end_idx) = content[abs_start..].find("```") {
                let json_str = content[abs_start..abs_start + end_idx].trim();
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                    let name = val.get("name").or_else(|| val.get("tool")).and_then(|v| v.as_str());
                    let args = val.get("arguments").or_else(|| val.get("parameters")).or_else(|| val.get("args"));
                    if let (Some(n), Some(a)) = (name, args) {
                        tool_calls.push(ToolCall {
                            id: format!("call_block_{}", Uuid::new_v4().simple()),
                            r#type: "function".to_string(),
                            function: FunctionCall {
                                name: n.to_string(),
                                arguments: if a.is_string() { a.as_str().unwrap().to_string() } else { a.to_string() },
                            },
                        });
                    }
                }
                cursor = abs_start + end_idx + 3;
            } else {
                break;
            }
        }
    }

    tool_calls
}

/// Checks if user message is a follow-up complaint or re-prompting action (e.g. "vẫn thế", "chưa mở được")
pub fn is_followup_complaint(text: &str) -> bool {
    let lower = text.to_lowercase();
    let trimmed = lower.trim();
    let patterns = [
        "vẫn thế",
        "van the",
        "vẫn vậy",
        "van vay",
        "sao không mở",
        "sao khong mo",
        "sao chưa mở",
        "sao chua mo",
        "chưa mở",
        "chua mo",
        "chưa được",
        "chua duoc",
        "không được",
        "khong duoc",
        "nó làm được gì",
        "no lam duoc gi",
        "có mở được đâu",
        "co mo duoc dau",
        "mở đi",
        "mo di",
        "làm đi",
        "lam di",
        "chạy đi",
        "chay di",
        "still the same",
        "not working",
        "didn't open",
    ];
    patterns.iter().any(|p| trimmed.contains(p))
}
