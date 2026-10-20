use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;
use uuid::Uuid;

use super::agent_registry::AgentType;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum DagError {
    #[error("Cycle detected in task DAG: {0}")]
    CycleDetected(String),

    #[error("Dangling edge references missing node: {0}")]
    DanglingEdge(Uuid),

    #[error("Node cannot depend on itself: {0}")]
    SelfDependency(Uuid),

    #[error("Duplicate node ID: {0}")]
    DuplicateNode(Uuid),

    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNode {
    pub task_id: Uuid,
    pub title: String,
    pub description: String,
    pub assigned_agent: AgentType,
    pub priority: i32,
    pub inputs: serde_json::Value,
    pub acceptance_criteria: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DagEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub condition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskDag {
    nodes: HashMap<Uuid, DagNode>,
    edges: Vec<DagEdge>,
}

impl TaskDag {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: DagNode) -> Result<(), DagError> {
        if self.nodes.contains_key(&node.task_id) {
            return Err(DagError::DuplicateNode(node.task_id));
        }
        self.nodes.insert(node.task_id, node);
        Ok(())
    }

    pub fn add_edge(&mut self, from: Uuid, to: Uuid, condition: &str) -> Result<(), DagError> {
        if from == to {
            return Err(DagError::SelfDependency(from));
        }
        if !self.nodes.contains_key(&from) {
            return Err(DagError::DanglingEdge(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(DagError::DanglingEdge(to));
        }

        let edge = DagEdge {
            from,
            to,
            condition: condition.to_string(),
        };

        if !self.edges.contains(&edge) {
            self.edges.push(edge);
        }

        // Validate cycle upon insertion
        self.validate()?;
        Ok(())
    }

    pub fn get_node(&self, id: &Uuid) -> Option<&DagNode> {
        self.nodes.get(id)
    }

    pub fn nodes(&self) -> &HashMap<Uuid, DagNode> {
        &self.nodes
    }

    pub fn edges(&self) -> &[DagEdge] {
        &self.edges
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Validates DAG integrity: no dangling edges, no self-dependency, and strictly acyclic
    pub fn validate(&self) -> Result<(), DagError> {
        // 1. Verify edge endpoints exist
        for edge in &self.edges {
            if edge.from == edge.to {
                return Err(DagError::SelfDependency(edge.from));
            }
            if !self.nodes.contains_key(&edge.from) {
                return Err(DagError::DanglingEdge(edge.from));
            }
            if !self.nodes.contains_key(&edge.to) {
                return Err(DagError::DanglingEdge(edge.to));
            }
        }

        // 2. Cycle detection via Kahn's Topological Sort Algorithm
        self.topological_sort().map(|_| ())
    }

    /// Computes topological ordering of tasks using Kahn's algorithm
    pub fn topological_sort(&self) -> Result<Vec<Uuid>, DagError> {
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        let mut adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        for node_id in self.nodes.keys() {
            in_degree.insert(*node_id, 0);
            adj.insert(*node_id, Vec::new());
        }

        for edge in &self.edges {
            *in_degree.entry(edge.to).or_insert(0) += 1;
            adj.entry(edge.from).or_default().push(edge.to);
        }

        let mut queue: VecDeque<Uuid> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut sorted = Vec::with_capacity(self.nodes.len());

        while let Some(u) = queue.pop_front() {
            sorted.push(u);
            if let Some(neighbors) = adj.get(&u) {
                for &v in neighbors {
                    if let Some(deg) = in_degree.get_mut(&v) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(v);
                        }
                    }
                }
            }
        }

        if sorted.len() != self.nodes.len() {
            return Err(DagError::CycleDetected(format!(
                "Visited {} out of {} nodes in graph. Remaining nodes form a cycle.",
                sorted.len(),
                self.nodes.len()
            )));
        }

        Ok(sorted)
    }

    /// Resolves which tasks are currently Ready to execute
    /// (all prerequisites are in completed_tasks, and node is not already completed or in-progress)
    pub fn get_ready_tasks(
        &self,
        completed_tasks: &HashSet<Uuid>,
        in_progress_tasks: &HashSet<Uuid>,
    ) -> Vec<Uuid> {
        let mut ready = Vec::new();

        // Map each task to its incoming prerequisite task IDs
        let mut prereqs: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for edge in &self.edges {
            prereqs.entry(edge.to).or_default().push(edge.from);
        }

        for &node_id in self.nodes.keys() {
            if completed_tasks.contains(&node_id) || in_progress_tasks.contains(&node_id) {
                continue;
            }

            // Check if all prerequisites are satisfied
            let is_ready = match prereqs.get(&node_id) {
                Some(deps) => deps.iter().all(|dep_id| completed_tasks.contains(dep_id)),
                None => true, // Root node with no prerequisites
            };

            if is_ready {
                ready.push(node_id);
            }
        }

        // Sort by priority descending
        ready.sort_by(|a, b| {
            let p_a = self.nodes.get(a).map(|n| n.priority).unwrap_or(0);
            let p_b = self.nodes.get(b).map(|n| n.priority).unwrap_or(0);
            p_b.cmp(&p_a)
        });

        ready
    }

    /// Dynamic replanning: inject new node and edges or remove cancelled branches
    pub fn dynamic_replan(
        &mut self,
        add_nodes: Vec<DagNode>,
        add_edges: Vec<DagEdge>,
        remove_node_ids: Vec<Uuid>,
    ) -> Result<(), DagError> {
        // Clone for atomic rollback if validation fails
        let mut draft = self.clone();

        for node in add_nodes {
            draft.add_node(node)?;
        }

        for edge in add_edges {
            draft.add_edge(edge.from, edge.to, &edge.condition)?;
        }

        for id in remove_node_ids {
            draft.nodes.remove(&id);
            draft.edges.retain(|e| e.from != id && e.to != id);
        }

        draft.validate()?;
        *self = draft;
        Ok(())
    }
}
