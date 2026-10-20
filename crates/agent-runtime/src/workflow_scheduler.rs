use agent_core::{
    AgentRegistry, DagNode, ResourceLockManager, SharedArtifactStore, TaskDag,
    WorkflowBudgetConfig, WorkflowBudgetTracker,
};
use agent_storage::{
    Database, SharedArtifact, TaskAttempt, TaskDependency, Workflow, WorkflowStatus,
    WorkflowTask, WorkflowTaskStatus, WorkflowTokenRecord,
};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::info;
use uuid::Uuid;

use crate::supervisor::SupervisorEngine;
use crate::worker_dispatcher::{WorkerDispatcher, WorkerExecutionResult};
use crate::{RuntimeError, RuntimeResult};

struct WorkflowControl {
    cancel_flag: Arc<AtomicBool>,
    pause_flag: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct WorkflowScheduler {
    database: Database,
    dispatcher: Arc<WorkerDispatcher>,
    supervisor: Arc<SupervisorEngine>,
    registry: Arc<RwLock<AgentRegistry>>,
    artifact_store: Arc<SharedArtifactStore>,
    lock_manager: Arc<ResourceLockManager>,
    active_workflows: Arc<RwLock<HashMap<Uuid, WorkflowControl>>>,
}

impl WorkflowScheduler {
    pub fn new(
        database: Database,
        dispatcher: Arc<WorkerDispatcher>,
        supervisor: Arc<SupervisorEngine>,
        registry: Arc<RwLock<AgentRegistry>>,
        artifact_store: Arc<SharedArtifactStore>,
        lock_manager: Arc<ResourceLockManager>,
    ) -> Self {
        Self {
            database,
            dispatcher,
            supervisor,
            registry,
            artifact_store,
            lock_manager,
            active_workflows: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn registry(&self) -> &Arc<RwLock<AgentRegistry>> {
        &self.registry
    }

    pub fn artifact_store(&self) -> &Arc<SharedArtifactStore> {
        &self.artifact_store
    }

    pub fn lock_manager(&self) -> &Arc<ResourceLockManager> {
        &self.lock_manager
    }

    /// Initializes and persists a workflow with its initial Task DAG
    pub async fn create_workflow(
        &self,
        title: &str,
        goal: &str,
        budget: WorkflowBudgetConfig,
    ) -> RuntimeResult<(Uuid, TaskDag)> {
        let workflow_id = Uuid::new_v4();

        // Supervisor plans DAG
        let dag = self.supervisor.plan_workflow(goal, &budget).await?;

        // Validate budget node limit
        let tracker = WorkflowBudgetTracker::new(budget.clone());
        tracker
            .check_node_count(dag.node_count())
            .map_err(|e| RuntimeError::Core(agent_core::CoreError::Model(e.to_string())))?;

        let wf = Workflow {
            id: workflow_id,
            title: title.to_string(),
            goal: goal.to_string(),
            status: WorkflowStatus::Pending,
            budget_json: serde_json::to_string(&budget).unwrap_or_default(),
            result_summary: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.database.create_workflow(&wf).await?;

        // Persist tasks in database
        for node in dag.nodes().values() {
            let task = WorkflowTask {
                id: node.task_id,
                workflow_id,
                parent_task_id: None,
                title: node.title.clone(),
                description: node.description.clone(),
                assigned_agent: node.assigned_agent.to_string(),
                status: WorkflowTaskStatus::Pending,
                priority: node.priority,
                inputs_json: serde_json::to_string(&node.inputs).unwrap_or_default(),
                outputs_json: "{}".to_string(),
                acceptance_criteria: node.acceptance_criteria.clone(),
                retry_count: 0,
                max_retries: budget.max_retries_per_task,
                timeout_secs: 300,
                started_at: None,
                completed_at: None,
                error: None,
            };
            self.database.create_workflow_task(&task).await?;
        }

        // Persist dependencies
        for edge in dag.edges() {
            let dep = TaskDependency {
                workflow_id,
                task_id: edge.to,
                depends_on_task_id: edge.from,
                condition: edge.condition.clone(),
            };
            self.database.add_task_dependency(&dep).await?;
        }

        Ok((workflow_id, dag))
    }

    /// Executes a workflow to completion with bounded concurrency
    pub async fn run_workflow(
        &self,
        workflow_id: Uuid,
        mut dag: TaskDag,
        budget: WorkflowBudgetConfig,
    ) -> RuntimeResult<WorkflowStatus> {
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let pause_flag = Arc::new(AtomicBool::new(false));

        {
            let mut active = self.active_workflows.write().await;
            active.insert(
                workflow_id,
                WorkflowControl {
                    cancel_flag: cancel_flag.clone(),
                    pause_flag: pause_flag.clone(),
                },
            );
        }

        self.database
            .update_workflow_status(workflow_id, WorkflowStatus::Running, None)
            .await?;

        let mut completed_tasks: HashSet<Uuid> = HashSet::new();
        let mut in_progress_tasks: HashSet<Uuid> = HashSet::new();
        let mut budget_tracker = WorkflowBudgetTracker::new(budget.clone());

        let max_concurrency = budget.max_concurrent_tasks.max(1);
        let (tx, mut rx) = mpsc::channel::<WorkerExecutionResult>(32);

        loop {
            // Check cancellation
            if cancel_flag.load(Ordering::SeqCst) {
                info!("Workflow {} cancelled by user", workflow_id);
                self.database
                    .update_workflow_status(workflow_id, WorkflowStatus::Cancelled, Some("Cancelled by user"))
                    .await?;
                self.cleanup_workflow(workflow_id).await;
                return Ok(WorkflowStatus::Cancelled);
            }

            // Check pause
            if pause_flag.load(Ordering::SeqCst) {
                info!("Workflow {} paused", workflow_id);
                self.database
                    .update_workflow_status(workflow_id, WorkflowStatus::Paused, Some("Paused"))
                    .await?;
                return Ok(WorkflowStatus::Paused);
            }

            // Check if all DAG tasks are completed
            if completed_tasks.len() == dag.node_count() {
                info!("All {} tasks completed for workflow {}", dag.node_count(), workflow_id);
                self.database
                    .update_workflow_status(
                        workflow_id,
                        WorkflowStatus::Completed,
                        Some("All tasks completed successfully"),
                    )
                    .await?;
                self.cleanup_workflow(workflow_id).await;
                return Ok(WorkflowStatus::Completed);
            }

            // Find tasks ready to execute
            let ready_tasks = dag.get_ready_tasks(&completed_tasks, &in_progress_tasks);

            // Dispatch ready tasks up to available concurrency slots
            for &task_id in &ready_tasks {
                if in_progress_tasks.len() >= max_concurrency {
                    break;
                }

                if let Some(node) = dag.get_node(&task_id).cloned() {
                    in_progress_tasks.insert(task_id);

                    self.database
                        .update_workflow_task_status(task_id, WorkflowTaskStatus::Running, None, None)
                        .await?;

                    let dispatcher = self.dispatcher.clone();
                    let tx_clone = tx.clone();

                    let wf_task = WorkflowTask {
                        id: node.task_id,
                        workflow_id,
                        parent_task_id: None,
                        title: node.title.clone(),
                        description: node.description.clone(),
                        assigned_agent: node.assigned_agent.to_string(),
                        status: WorkflowTaskStatus::Running,
                        priority: node.priority,
                        inputs_json: serde_json::to_string(&node.inputs).unwrap_or_default(),
                        outputs_json: "{}".to_string(),
                        acceptance_criteria: node.acceptance_criteria.clone(),
                        retry_count: 0,
                        max_retries: budget.max_retries_per_task,
                        timeout_secs: 300,
                        started_at: Some(Utc::now()),
                        completed_at: None,
                        error: None,
                    };

                    tokio::spawn(async move {
                        let res = match dispatcher.execute_task(&wf_task).await {
                            Ok(r) => r,
                            Err(e) => WorkerExecutionResult {
                                task_id,
                                status: WorkflowTaskStatus::Failed,
                                artifacts: Vec::new(),
                                outputs: serde_json::json!({}),
                                error: Some(e.to_string()),
                                token_in: 0,
                                token_out: 0,
                            },
                        };
                        let _ = tx_clone.send(res).await;
                    });
                }
            }

            // Wait for at least one in-progress task to finish
            if !in_progress_tasks.is_empty() {
                if let Some(result) = rx.recv().await {
                    in_progress_tasks.remove(&result.task_id);

                    // Track tokens & budget
                    let _ = budget_tracker.record_tokens(result.token_in, result.token_out);
                    let token_rec = WorkflowTokenRecord {
                        id: Uuid::new_v4(),
                        workflow_id,
                        task_id: Some(result.task_id),
                        agent_type: "Worker".to_string(),
                        token_in: result.token_in,
                        token_out: result.token_out,
                        estimated_cost: ((result.token_in + result.token_out) as f64 / 1_000_000.0) * 3.0,
                        created_at: Utc::now(),
                    };
                    let _ = self.database.record_workflow_tokens(&token_rec).await;

                    // Save artifacts in database
                    for art_ref in &result.artifacts {
                        let art_record = SharedArtifact {
                            id: art_ref.id,
                            workflow_id,
                            task_id: Some(result.task_id),
                            artifact_type: art_ref.artifact_type.clone(),
                            name: art_ref.name.clone(),
                            path_or_uri: art_ref.path_or_uri.clone(),
                            content_preview: art_ref.summary.clone(),
                            hash: art_ref.hash.clone(),
                            metadata_json: "{}".to_string(),
                            created_at: art_ref.created_at,
                        };
                        let _ = self.database.save_shared_artifact(&art_record).await;
                    }

                    // Record task attempt
                    let attempt = TaskAttempt {
                        id: Uuid::new_v4(),
                        task_id: result.task_id,
                        attempt_number: 1,
                        agent_id: "WorkerAgent".to_string(),
                        status: result.status.to_string(),
                        error: result.error.clone(),
                        started_at: Utc::now(),
                        completed_at: Some(Utc::now()),
                        token_in: result.token_in,
                        token_out: result.token_out,
                    };
                    let _ = self.database.create_task_attempt(&attempt).await;

                    let out_json = serde_json::to_string(&result.outputs).unwrap_or_default();

                    if result.status == WorkflowTaskStatus::Completed {
                        completed_tasks.insert(result.task_id);
                        self.database
                            .update_workflow_task_status(
                                result.task_id,
                                WorkflowTaskStatus::Completed,
                                None,
                                Some(&out_json),
                            )
                            .await?;
                    } else {
                        // Task failed
                        self.database
                            .update_workflow_task_status(
                                result.task_id,
                                WorkflowTaskStatus::Failed,
                                result.error.as_deref(),
                                Some(&out_json),
                            )
                            .await?;

                        // Try dynamic replan if budget allows
                        let err_msg = result.error.clone().unwrap_or_else(|| "Unknown failure".to_string());
                        if let Ok(fix_id) = self.supervisor.replan_failed_task(&mut dag, result.task_id, &err_msg) {
                            info!("Supervisor replanned: injected fix task {}", fix_id);
                            // Persist fix task in DB
                            if let Some(fix_node) = dag.get_node(&fix_id) {
                                let fix_wf_task = WorkflowTask {
                                    id: fix_id,
                                    workflow_id,
                                    parent_task_id: Some(result.task_id),
                                    title: fix_node.title.clone(),
                                    description: fix_node.description.clone(),
                                    assigned_agent: fix_node.assigned_agent.to_string(),
                                    status: WorkflowTaskStatus::Pending,
                                    priority: fix_node.priority,
                                    inputs_json: serde_json::to_string(&fix_node.inputs).unwrap_or_default(),
                                    outputs_json: "{}".to_string(),
                                    acceptance_criteria: fix_node.acceptance_criteria.clone(),
                                    retry_count: 0,
                                    max_retries: budget.max_retries_per_task,
                                    timeout_secs: 300,
                                    started_at: None,
                                    completed_at: None,
                                    error: None,
                                };
                                let _ = self.database.create_workflow_task(&fix_wf_task).await;
                            }
                        } else {
                            // Cannot replan, fail workflow
                            self.database
                                .update_workflow_status(
                                    workflow_id,
                                    WorkflowStatus::Failed,
                                    Some(&format!("Task failed: {}", err_msg)),
                                )
                                .await?;
                            self.cleanup_workflow(workflow_id).await;
                            return Ok(WorkflowStatus::Failed);
                        }
                    }
                }
            } else if ready_tasks.is_empty() && completed_tasks.len() < dag.node_count() {
                // Deadlock / unmet dependencies
                self.database
                    .update_workflow_status(
                        workflow_id,
                        WorkflowStatus::Failed,
                        Some("Deadlock: remaining tasks have unmet dependencies"),
                    )
                    .await?;
                self.cleanup_workflow(workflow_id).await;
                return Ok(WorkflowStatus::Failed);
            }
        }
    }

    pub async fn cancel_workflow(&self, workflow_id: Uuid) -> bool {
        let active = self.active_workflows.read().await;
        if let Some(ctrl) = active.get(&workflow_id) {
            ctrl.cancel_flag.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub async fn pause_workflow(&self, workflow_id: Uuid) -> bool {
        let active = self.active_workflows.read().await;
        if let Some(ctrl) = active.get(&workflow_id) {
            ctrl.pause_flag.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub async fn resume_workflow(&self, workflow_id: Uuid) -> RuntimeResult<WorkflowStatus> {
        // Read workflow & tasks from database, reconstruct TaskDag, and resume
        let wf = self
            .database
            .get_workflow(workflow_id)
            .await?
            .ok_or_else(|| RuntimeError::TaskNotFound(workflow_id))?;

        let tasks = self.database.list_workflow_tasks(workflow_id).await?;
        let deps = self.database.get_task_dependencies(workflow_id).await?;

        let mut dag = TaskDag::new();
        for t in &tasks {
            let agent_type = t.assigned_agent.parse().unwrap_or(agent_core::AgentType::Coding);
            let _ = dag.add_node(DagNode {
                task_id: t.id,
                title: t.title.clone(),
                description: t.description.clone(),
                assigned_agent: agent_type,
                priority: t.priority,
                inputs: serde_json::from_str(&t.inputs_json).unwrap_or(serde_json::json!({})),
                acceptance_criteria: t.acceptance_criteria.clone(),
            });
        }

        for d in &deps {
            let _ = dag.add_edge(d.depends_on_task_id, d.task_id, &d.condition);
        }

        let budget: WorkflowBudgetConfig =
            serde_json::from_str(&wf.budget_json).unwrap_or_default();

        self.run_workflow(workflow_id, dag, budget).await
    }

    async fn cleanup_workflow(&self, workflow_id: Uuid) {
        let mut active = self.active_workflows.write().await;
        active.remove(&workflow_id);
    }
}
