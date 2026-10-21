use agent_core::{
    AgentRegistry, AgentType, DagNode, TaskDag,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use crate::{RuntimeResult, SupervisorEngine};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusinessWorkflowTemplate {
    MarketResearch {
        topic: String,
        target_industry: String,
        output_format: String, // "DOCX" or "PDF"
    },
    DataReporting {
        dataset_path: String,
        analysis_metric: String,
        group_column: Option<String>,
        output_format: String,
    },
    EmailAssistance {
        query: String,
        action_requested: String,
        sender_filter: Option<String>,
    },
    ResearchToDocument {
        topic: String,
        output_format: String,
    },
    HybridPipeline {
        dataset_path: String,
        analysis_metric: String,
        recipient_email: String,
        subject: String,
    },
}

pub struct BusinessWorkflowCoordinator {
    workspace_root: PathBuf,
    supervisor: Arc<SupervisorEngine>,
    agent_registry: Arc<AgentRegistry>,
}

impl BusinessWorkflowCoordinator {
    pub fn new(
        workspace_root: PathBuf,
        supervisor: Arc<SupervisorEngine>,
        agent_registry: Arc<AgentRegistry>,
    ) -> Self {
        Self {
            workspace_root,
            supervisor,
            agent_registry,
        }
    }

    pub fn workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }

    pub fn supervisor(&self) -> &Arc<SupervisorEngine> {
        &self.supervisor
    }

    pub fn agent_registry(&self) -> &Arc<AgentRegistry> {
        &self.agent_registry
    }

    /// Builds a validated TaskDag for the specified business template
    pub fn build_template_dag(
        &self,
        template: &BusinessWorkflowTemplate,
        _workflow_id: Uuid,
    ) -> RuntimeResult<TaskDag> {
        let mut dag = TaskDag::new();

        match template {
            BusinessWorkflowTemplate::MarketResearch {
                topic,
                target_industry,
                output_format,
            } => {
                let t1_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t1_id,
                    title: format!("Search & Ingest Market Data for {}", topic),
                    description: format!(
                        "Query industry sources for {} in the {} sector and extract verifiable evidence items.",
                        topic, target_industry
                    ),
                    assigned_agent: AgentType::Research,
                    priority: 1,
                    inputs: serde_json::json!({ "topic": topic, "industry": target_industry }),
                    acceptance_criteria: "At least 2 verified sources retrieved".to_string(),
                })?;

                let t2_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t2_id,
                    title: "Verify Evidence & Check Contradictions".to_string(),
                    description: "Cross-examine retrieved evidence for metric divergence and semantic contradictions.".to_string(),
                    assigned_agent: AgentType::Research,
                    priority: 2,
                    inputs: serde_json::json!({ "check_discrepancies": true }),
                    acceptance_criteria: "Contradiction report compiled".to_string(),
                })?;
                dag.add_edge(t1_id, t2_id, "Success")?;

                let t3_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t3_id,
                    title: format!("Generate {} Research Deliverable", output_format),
                    description: format!(
                        "Synthesize findings, structured tables, and citations into an executive {} deliverable.",
                        output_format
                    ),
                    assigned_agent: AgentType::Document,
                    priority: 3,
                    inputs: serde_json::json!({ "format": output_format, "title": format!("{} Market Report", topic) }),
                    acceptance_criteria: format!("Valid {} file produced in workspace", output_format),
                })?;
                dag.add_edge(t2_id, t3_id, "Success")?;
            }

            BusinessWorkflowTemplate::DataReporting {
                dataset_path,
                analysis_metric,
                group_column,
                output_format,
            } => {
                let t1_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t1_id,
                    title: format!("Ingest & Analyze {}", analysis_metric),
                    description: format!("Read {} and calculate count, sum, mean, median, min, max, stddev.", dataset_path),
                    assigned_agent: AgentType::DataAnalysis,
                    priority: 1,
                    inputs: serde_json::json!({ "file": dataset_path, "metric": analysis_metric }),
                    acceptance_criteria: "Statistical summary produced with non-zero count".to_string(),
                })?;

                let t2_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t2_id,
                    title: "Aggregate & Generate Visualization".to_string(),
                    description: format!("Group by {:?} and generate SVG bar/line charts.", group_column),
                    assigned_agent: AgentType::DataAnalysis,
                    priority: 2,
                    inputs: serde_json::json!({ "group_by": group_column }),
                    acceptance_criteria: "SVG chart artifact generated".to_string(),
                })?;
                dag.add_edge(t1_id, t2_id, "Success")?;

                let t3_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t3_id,
                    title: format!("Export Final {} Financial/Data Report", output_format),
                    description: format!("Assemble statistical summary and SVG chart into a clean {} document.", output_format),
                    assigned_agent: AgentType::Document,
                    priority: 3,
                    inputs: serde_json::json!({ "format": output_format }),
                    acceptance_criteria: format!("{} deliverable successfully saved", output_format),
                })?;
                dag.add_edge(t2_id, t3_id, "Success")?;
            }

            BusinessWorkflowTemplate::EmailAssistance {
                query,
                action_requested,
                sender_filter,
            } => {
                let t1_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t1_id,
                    title: format!("Search Mailbox for '{}'", query),
                    description: format!("Search messages matching query '{}' from sender {:?}", query, sender_filter),
                    assigned_agent: AgentType::Email,
                    priority: 1,
                    inputs: serde_json::json!({ "query": query, "sender": sender_filter }),
                    acceptance_criteria: "Email messages queried".to_string(),
                })?;

                let t2_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t2_id,
                    title: "Summarize Thread & Extract Action Items".to_string(),
                    description: format!("Read email body and extract key action items for '{}'", action_requested),
                    assigned_agent: AgentType::Research,
                    priority: 2,
                    inputs: serde_json::json!({ "action": action_requested }),
                    acceptance_criteria: "Action items listed".to_string(),
                })?;
                dag.add_edge(t1_id, t2_id, "Success")?;

                let t3_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t3_id,
                    title: "Compose Email Draft (Requires Approval Before Sending)".to_string(),
                    description: "Create draft reply with operation ID. Delivery held until explicit human approval.".to_string(),
                    assigned_agent: AgentType::Email,
                    priority: 3,
                    inputs: serde_json::json!({ "requires_approval": true }),
                    acceptance_criteria: "Email draft stored in database".to_string(),
                })?;
                dag.add_edge(t2_id, t3_id, "Success")?;
            }

            BusinessWorkflowTemplate::ResearchToDocument { topic, output_format } => {
                let t1_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t1_id,
                    title: format!("Research Topic: {}", topic),
                    description: format!("Collect facts and findings on {}", topic),
                    assigned_agent: AgentType::Research,
                    priority: 1,
                    inputs: serde_json::json!({ "topic": topic }),
                    acceptance_criteria: "Findings compiled".to_string(),
                })?;

                let t2_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t2_id,
                    title: format!("Produce {} Document", output_format),
                    description: format!("Format research findings into {} format", output_format),
                    assigned_agent: AgentType::Document,
                    priority: 2,
                    inputs: serde_json::json!({ "format": output_format }),
                    acceptance_criteria: "File created".to_string(),
                })?;
                dag.add_edge(t1_id, t2_id, "Success")?;
            }

            BusinessWorkflowTemplate::HybridPipeline {
                dataset_path,
                analysis_metric,
                recipient_email,
                subject,
            } => {
                let t1_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t1_id,
                    title: "Analyze Dataset Metrics".to_string(),
                    description: format!("Analyze {} in {}", analysis_metric, dataset_path),
                    assigned_agent: AgentType::DataAnalysis,
                    priority: 1,
                    inputs: serde_json::json!({ "path": dataset_path }),
                    acceptance_criteria: "Dataset metrics calculated".to_string(),
                })?;

                let t2_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t2_id,
                    title: "Generate Executive Report Document".to_string(),
                    description: "Compile data analysis into DOCX/PDF report".to_string(),
                    assigned_agent: AgentType::Document,
                    priority: 2,
                    inputs: serde_json::json!({ "format": "DOCX" }),
                    acceptance_criteria: "Document artifact generated".to_string(),
                })?;
                dag.add_edge(t1_id, t2_id, "Success")?;

                let t3_id = Uuid::new_v4();
                dag.add_node(DagNode {
                    task_id: t3_id,
                    title: "Draft Executive Delivery Email".to_string(),
                    description: format!("Draft delivery email to {} with subject '{}'", recipient_email, subject),
                    assigned_agent: AgentType::Email,
                    priority: 3,
                    inputs: serde_json::json!({ "to": recipient_email, "subject": subject }),
                    acceptance_criteria: "Draft ready for user approval".to_string(),
                })?;
                dag.add_edge(t2_id, t3_id, "Success")?;
            }
        }

        dag.validate()?;
        Ok(dag)
    }
}
