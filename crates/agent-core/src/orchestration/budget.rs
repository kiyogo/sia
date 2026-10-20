use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum BudgetError {
    #[error("Workflow token budget exceeded: used {used} of {max} tokens")]
    TokenLimitExceeded { used: u64, max: u64 },

    #[error("Task limit exceeded: graph has {count} nodes, max allowed is {max}")]
    MaxTasksExceeded { count: usize, max: usize },

    #[error("Maximum delegation depth exceeded: depth {depth} exceeds limit {max}")]
    MaxDepthExceeded { depth: usize, max: usize },

    #[error("Max retries exceeded for task: attempted {attempts} times")]
    MaxRetriesExceeded { attempts: u32, max: u32 },

    #[error("Execution time exceeded: elapsed {elapsed_secs}s exceeds limit {max_secs}s")]
    TimeoutExceeded { elapsed_secs: u64, max_secs: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowBudgetConfig {
    pub max_tokens: u64,
    pub max_cost_usd: f64,
    pub max_tasks: usize,
    pub max_delegation_depth: usize,
    pub max_retries_per_task: u32,
    pub max_duration_secs: u64,
    pub max_concurrent_tasks: usize,
}

impl Default for WorkflowBudgetConfig {
    fn default() -> Self {
        Self {
            max_tokens: 100_000,
            max_cost_usd: 5.0,
            max_tasks: 25,
            max_delegation_depth: 5,
            max_retries_per_task: 3,
            max_duration_secs: 1800, // 30 minutes
            max_concurrent_tasks: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowBudgetTracker {
    pub config: WorkflowBudgetConfig,
    pub total_token_in: u64,
    pub total_token_out: u64,
    pub total_cost_usd: f64,
    pub tasks_created: usize,
    pub tasks_completed: usize,
    pub tasks_failed: usize,
}

impl WorkflowBudgetTracker {
    pub fn new(config: WorkflowBudgetConfig) -> Self {
        Self {
            config,
            total_token_in: 0,
            total_token_out: 0,
            total_cost_usd: 0.0,
            tasks_created: 0,
            tasks_completed: 0,
            tasks_failed: 0,
        }
    }

    pub fn check_node_count(&self, count: usize) -> Result<(), BudgetError> {
        if count > self.config.max_tasks {
            return Err(BudgetError::MaxTasksExceeded {
                count,
                max: self.config.max_tasks,
            });
        }
        Ok(())
    }

    pub fn check_depth(&self, depth: usize) -> Result<(), BudgetError> {
        if depth > self.config.max_delegation_depth {
            return Err(BudgetError::MaxDepthExceeded {
                depth,
                max: self.config.max_delegation_depth,
            });
        }
        Ok(())
    }

    pub fn record_tokens(&mut self, token_in: u64, token_out: u64) -> Result<(), BudgetError> {
        self.total_token_in += token_in;
        self.total_token_out += token_out;
        let total = self.total_token_in + self.total_token_out;

        // Approx pricing ($3 / 1M tokens)
        self.total_cost_usd = (total as f64 / 1_000_000.0) * 3.0;

        if total > self.config.max_tokens {
            return Err(BudgetError::TokenLimitExceeded {
                used: total,
                max: self.config.max_tokens,
            });
        }
        Ok(())
    }

    pub fn check_remaining_tokens(&self) -> Result<(), BudgetError> {
        let total = self.total_token_in + self.total_token_out;
        if total >= self.config.max_tokens {
            return Err(BudgetError::TokenLimitExceeded {
                used: total,
                max: self.config.max_tokens,
            });
        }
        Ok(())
    }
}
