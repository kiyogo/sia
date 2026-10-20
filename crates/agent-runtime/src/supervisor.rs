use agent_core::{
    AgentType, ChatMessage, CoreError, DagEdge, DagNode, ModelProvider, ModelRequest, TaskDag,
    WorkflowBudgetConfig,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{RuntimeError, RuntimeResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTaskDto {
    pub id: Option<String>,
    pub title: String,
    pub description: String,
    pub assigned_agent: String,
    pub priority: Option<i32>,
    pub dependencies: Option<Vec<String>>,
    pub acceptance_criteria: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDecompositionResponse {
    pub tasks: Vec<PlannedTaskDto>,
}

pub struct SupervisorEngine {
    provider: Option<Arc<dyn ModelProvider>>,
}

impl SupervisorEngine {
    pub fn new(provider: Option<Arc<dyn ModelProvider>>) -> Self {
        Self { provider }
    }

    /// Decomposes a user goal into a validated TaskDag
    pub async fn plan_workflow(
        &self,
        goal: &str,
        budget: &WorkflowBudgetConfig,
    ) -> RuntimeResult<TaskDag> {
        // Attempt model-backed decomposition if provider is configured
        if let Some(ref prov) = self.provider {
            if let Ok(dag) = self.decompose_via_llm(prov.as_ref(), goal, budget).await {
                return Ok(dag);
            }
        }

        // Deterministic fallback decomposition
        self.decompose_deterministically(goal, budget)
    }

    async fn decompose_via_llm(
        &self,
        provider: &dyn ModelProvider,
        goal: &str,
        budget: &WorkflowBudgetConfig,
    ) -> RuntimeResult<TaskDag> {
        let system_prompt = r#"You are the Supervisor AI for an Autonomous Multi-Agent Workspace.
Your job is to analyze the user's high-level goal and decompose it into a clean, acyclic Task Directed Acyclic Graph (DAG).
Available specialized agent types:
- "Coding": Writes code, edits files, builds, runs tests, git operations.
- "Browser": Navigates web pages, inspects DOM, fills forms, verifies web apps.
- "Computer": Interacts with Windows applications, window switching, desktop controls.
- "Research": Searches documentation, summarizes files, analyzes symbols.
- "Supervisor": High-level synthesis and final verification.

Return ONLY a valid JSON object matching this schema:
{
  "tasks": [
    {
      "id": "t1",
      "title": "Title",
      "description": "Details",
      "assigned_agent": "Coding" | "Browser" | "Computer" | "Research" | "Supervisor",
      "priority": 10,
      "dependencies": [],
      "acceptance_criteria": "Deterministic proof of completion"
    }
  ]
}"#;

        let req = ModelRequest {
            model: "default".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: Some(system_prompt.to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: Some(format!("Goal: {}", goal)),
                    tool_calls: None,
                    tool_call_id: None,
                },
            ],
            tools: None,
            temperature: Some(0.1),
            max_tokens: Some(2000),
        };

        let resp = provider
            .generate(&req)
            .await
            .map_err(|e| RuntimeError::Core(e))?;

        let content = resp.content.unwrap_or_default();
        let json_start = content.find('{').unwrap_or(0);
        let json_end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        let json_slice = &content[json_start..json_end];

        let parsed: PlanDecompositionResponse =
            serde_json::from_str(json_slice).map_err(|_| RuntimeError::ProviderNotConfigured)?;

        let mut dag = TaskDag::new();
        let mut id_map: std::collections::HashMap<String, Uuid> = std::collections::HashMap::new();

        if parsed.tasks.is_empty() {
            return Err(RuntimeError::ProviderNotConfigured);
        }

        // Check node limit
        if parsed.tasks.len() > budget.max_tasks {
            return Err(RuntimeError::Core(CoreError::Model(
                format!("Plan exceeds max task limit of {}", budget.max_tasks),
            )));
        }

        for (idx, t) in parsed.tasks.iter().enumerate() {
            let node_id = Uuid::new_v4();
            let key = t.id.clone().unwrap_or_else(|| format!("t{}", idx + 1));
            id_map.insert(key, node_id);

            let agent_type: AgentType = t.assigned_agent.parse().unwrap_or(AgentType::Coding);

            dag.add_node(DagNode {
                task_id: node_id,
                title: t.title.clone(),
                description: t.description.clone(),
                assigned_agent: agent_type,
                priority: t.priority.unwrap_or(5),
                inputs: serde_json::json!({}),
                acceptance_criteria: t.acceptance_criteria.clone().unwrap_or_default(),
            })
            .map_err(|e| RuntimeError::Core(CoreError::Model(e.to_string())))?;
        }

        for (idx, t) in parsed.tasks.iter().enumerate() {
            let to_key = t.id.clone().unwrap_or_else(|| format!("t{}", idx + 1));
            if let Some(&to_id) = id_map.get(&to_key) {
                if let Some(deps) = &t.dependencies {
                    for from_key in deps {
                        if let Some(&from_id) = id_map.get(from_key) {
                            dag.add_edge(from_id, to_id, "Success")
                                .map_err(|e| RuntimeError::Core(CoreError::Model(e.to_string())))?;
                        }
                    }
                }
            }
        }

        dag.validate()
            .map_err(|e| RuntimeError::Core(CoreError::Model(e.to_string())))?;

        Ok(dag)
    }

    /// Deterministic decomposition based on task domain heuristics
    pub fn decompose_deterministically(
        &self,
        goal: &str,
        _budget: &WorkflowBudgetConfig,
    ) -> RuntimeResult<TaskDag> {
        let mut dag = TaskDag::new();
        let goal_lower = goal.to_lowercase();

        if goal_lower.contains("web") || goal_lower.contains("browser") || goal_lower.contains("site") || goal_lower.contains("html") {
            // Full Web + Coding + Browser Workflow
            let t1 = Uuid::new_v4();
            let t2 = Uuid::new_v4();
            let t3 = Uuid::new_v4();
            let t4 = Uuid::new_v4();
            let t5 = Uuid::new_v4();

            dag.add_node(DagNode {
                task_id: t1,
                title: "Scaffold Web Application".to_string(),
                description: format!("Initialize project structure for: {}", goal),
                assigned_agent: AgentType::Coding,
                priority: 10,
                inputs: serde_json::json!({"action": "scaffold"}),
                acceptance_criteria: "Files exist on disk".to_string(),
            }).unwrap();

            dag.add_node(DagNode {
                task_id: t2,
                title: "Implement Components & Assets".to_string(),
                description: "Write application source code and styling".to_string(),
                assigned_agent: AgentType::Coding,
                priority: 9,
                inputs: serde_json::json!({"action": "code"}),
                acceptance_criteria: "Valid syntax with zero compile errors".to_string(),
            }).unwrap();

            dag.add_node(DagNode {
                task_id: t3,
                title: "Build & Compiler Verification".to_string(),
                description: "Execute compiler and verify clean build".to_string(),
                assigned_agent: AgentType::Coding,
                priority: 8,
                inputs: serde_json::json!({"action": "build"}),
                acceptance_criteria: "Exit code 0".to_string(),
            }).unwrap();

            dag.add_node(DagNode {
                task_id: t4,
                title: "Browser Navigation & DOM Verification".to_string(),
                description: "Navigate to page and verify interactive elements via DOM tree".to_string(),
                assigned_agent: AgentType::Browser,
                priority: 7,
                inputs: serde_json::json!({"action": "verify_browser"}),
                acceptance_criteria: "Page renders without console errors".to_string(),
            }).unwrap();

            dag.add_node(DagNode {
                task_id: t5,
                title: "Supervisor Synthesis & Report".to_string(),
                description: "Review artifacts and summarize completion".to_string(),
                assigned_agent: AgentType::Supervisor,
                priority: 5,
                inputs: serde_json::json!({"action": "report"}),
                acceptance_criteria: "Comprehensive report produced".to_string(),
            }).unwrap();

            dag.add_edge(t1, t2, "Success").unwrap();
            dag.add_edge(t2, t3, "Success").unwrap();
            dag.add_edge(t3, t4, "Success").unwrap();
            dag.add_edge(t4, t5, "Success").unwrap();
        } else if goal_lower.contains("desktop") || goal_lower.contains("window") || goal_lower.contains("notepad") {
            // Desktop Automation Workflow
            let t1 = Uuid::new_v4();
            let t2 = Uuid::new_v4();
            let t3 = Uuid::new_v4();

            dag.add_node(DagNode {
                task_id: t1,
                title: "Inspect Desktop Windows".to_string(),
                description: "Enumerate active windows and locate target application".to_string(),
                assigned_agent: AgentType::Computer,
                priority: 10,
                inputs: serde_json::json!({"action": "inspect_windows"}),
                acceptance_criteria: "Target window found".to_string(),
            }).unwrap();

            dag.add_node(DagNode {
                task_id: t2,
                title: "Execute Desktop GUI Interaction".to_string(),
                description: "Focus window, input text, and interact with controls".to_string(),
                assigned_agent: AgentType::Computer,
                priority: 8,
                inputs: serde_json::json!({"action": "interact_gui"}),
                acceptance_criteria: "UI state verified".to_string(),
            }).unwrap();

            dag.add_node(DagNode {
                task_id: t3,
                title: "Supervisor Verification".to_string(),
                description: "Validate outcome of desktop operations".to_string(),
                assigned_agent: AgentType::Supervisor,
                priority: 5,
                inputs: serde_json::json!({"action": "verify"}),
                acceptance_criteria: "All actions confirmed".to_string(),
            }).unwrap();

            dag.add_edge(t1, t2, "Success").unwrap();
            dag.add_edge(t2, t3, "Success").unwrap();
        } else {
            // General Code & Build Workflow
            let t1 = Uuid::new_v4();
            let t2 = Uuid::new_v4();
            let t3 = Uuid::new_v4();

            dag.add_node(DagNode {
                task_id: t1,
                title: "Analyze Repository & Requirements".to_string(),
                description: format!("Analyze project outline for: {}", goal),
                assigned_agent: AgentType::Research,
                priority: 10,
                inputs: serde_json::json!({"action": "analyze"}),
                acceptance_criteria: "Context gathered".to_string(),
            }).unwrap();

            dag.add_node(DagNode {
                task_id: t2,
                title: "Execute Code Modifications".to_string(),
                description: "Apply atomic edits or patches to workspace files".to_string(),
                assigned_agent: AgentType::Coding,
                priority: 8,
                inputs: serde_json::json!({"action": "modify"}),
                acceptance_criteria: "Patches applied cleanly".to_string(),
            }).unwrap();

            dag.add_node(DagNode {
                task_id: t3,
                title: "Run Test Suite & Compiler".to_string(),
                description: "Execute compiler or tests to ensure zero regressions".to_string(),
                assigned_agent: AgentType::Coding,
                priority: 6,
                inputs: serde_json::json!({"action": "test"}),
                acceptance_criteria: "All tests pass".to_string(),
            }).unwrap();

            dag.add_edge(t1, t2, "Success").unwrap();
            dag.add_edge(t2, t3, "Success").unwrap();
        }

        dag.validate()
            .map_err(|e| RuntimeError::Core(CoreError::Model(e.to_string())))?;

        Ok(dag)
    }

    /// Injects a corrective task node when a task fails during workflow execution
    pub fn replan_failed_task(
        &self,
        dag: &mut TaskDag,
        failed_task_id: Uuid,
        error_msg: &str,
    ) -> RuntimeResult<Uuid> {
        let fix_task_id = Uuid::new_v4();
        let failed_node = dag
            .get_node(&failed_task_id)
            .cloned()
            .ok_or_else(|| RuntimeError::TaskNotFound(failed_task_id))?;

        let fix_node = DagNode {
            task_id: fix_task_id,
            title: format!("Autonomous Fix: {}", failed_node.title),
            description: format!("Diagnose and repair failure: {}", error_msg),
            assigned_agent: failed_node.assigned_agent.clone(),
            priority: failed_node.priority + 1,
            inputs: serde_json::json!({
                "original_task_id": failed_task_id.to_string(),
                "error": error_msg,
            }),
            acceptance_criteria: "Error resolved and tests verified".to_string(),
        };

        // Find nodes that depended on the failed task, and redirect them to depend on fix_task
        let mut edges_to_add = Vec::new();
        edges_to_add.push(DagEdge {
            from: failed_task_id,
            to: fix_task_id,
            condition: "Completed".to_string(), // Can run after failed task finishes
        });

        for edge in dag.edges().iter() {
            if edge.from == failed_task_id && edge.to != fix_task_id {
                edges_to_add.push(DagEdge {
                    from: fix_task_id,
                    to: edge.to,
                    condition: "Success".to_string(),
                });
            }
        }

        dag.dynamic_replan(vec![fix_node], edges_to_add, vec![])
            .map_err(|e| RuntimeError::Core(CoreError::Model(e.to_string())))?;

        Ok(fix_task_id)
    }
}
