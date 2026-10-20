use crate::error::{CoreError, CoreResult};
use crate::model::{ChatMessage, ModelProvider, ModelRequest, ModelUsage};
use crate::token_optimizer::TokenOptimizer;
use agent_security::SharedSecurityEngine;
use agent_storage::{Checkpoint, Database, Task, TaskStatus, ToolAuditLog};
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
            ChatMessage {
                role: "user".to_string(),
                content: Some(user_request.to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let mut cumulative_usage = ModelUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            cached_tokens: None,
        };

        let tools = self.registry.list_definitions().await;

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
                    let _ = self.event_sender.send(AgentEvent::Error {
                        message: e.to_string(),
                    });
                    self.database
                        .update_task_status(task.id, TaskStatus::Failed, "Model generation error", None, Some(&e.to_string()))
                        .await?;
                    return Err(e);
                }
            };

            // Aggregate token usage
            cumulative_usage.prompt_tokens += resp.usage.prompt_tokens;
            cumulative_usage.completion_tokens += resp.usage.completion_tokens;
            cumulative_usage.total_tokens += resp.usage.total_tokens;

            let _ = self.event_sender.send(AgentEvent::TokenUsageUpdated {
                prompt_tokens: cumulative_usage.prompt_tokens,
                completion_tokens: cumulative_usage.completion_tokens,
                total_tokens: cumulative_usage.total_tokens,
            });

            // Check if model emitted tool calls
            if resp.tool_calls.is_empty() {
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

            let call_count = resp.tool_calls.len();

            // Append assistant response turn containing tool calls
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: resp.content.clone(),
                tool_calls: Some(resp.tool_calls.clone()),
                tool_call_id: None,
            });

            // Execute tool calls
            for call in resp.tool_calls {
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
