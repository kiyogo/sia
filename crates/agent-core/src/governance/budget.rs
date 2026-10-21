use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetHierarchyConfig {
    pub global_daily_cost_limit: f64,
    pub global_monthly_cost_limit: f64,
    pub default_workflow_cost_limit: f64,
    pub default_task_cost_limit: f64,
}

impl Default for BudgetHierarchyConfig {
    fn default() -> Self {
        Self {
            global_daily_cost_limit: 25.0,
            global_monthly_cost_limit: 200.0,
            default_workflow_cost_limit: 5.0,
            default_task_cost_limit: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationReceipt {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub reserved_tokens: u64,
    pub reserved_cost: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct ActiveReservation {
    pub receipt: ReservationReceipt,
    pub expires_at: DateTime<Utc>,
}

pub struct BudgetGovernanceEngine {
    config: RwLock<BudgetHierarchyConfig>,
    spent_today: RwLock<(u32, f64)>, // (day_of_year, total_cost)
    spent_month: RwLock<(u32, f64)>, // (month, total_cost)
    workflow_spent: RwLock<HashMap<String, f64>>,
    active_reservations: RwLock<HashMap<Uuid, ActiveReservation>>,
}

impl BudgetGovernanceEngine {
    pub fn new(config: BudgetHierarchyConfig) -> Self {
        let now = Utc::now();
        Self {
            config: RwLock::new(config),
            spent_today: RwLock::new((now.ordinal(), 0.0)),
            spent_month: RwLock::new((now.month(), 0.0)),
            workflow_spent: RwLock::new(HashMap::new()),
            active_reservations: RwLock::new(HashMap::new()),
        }
    }

    /// Pre-request budget reservation to eliminate multi-agent race conditions
    pub async fn reserve(
        &self,
        entity_type: &str,
        entity_id: &str,
        estimated_tokens: u64,
        estimated_cost: f64,
    ) -> Result<ReservationReceipt, String> {
        let now = Utc::now();
        let cfg = self.config.read().await;

        // Clean up expired reservations (> 5 minutes old)
        {
            let mut reservations = self.active_reservations.write().await;
            reservations.retain(|_, r| r.expires_at > now);
        }

        // Calculate total pending reserved cost
        let total_active_reserved: f64 = {
            let reservations = self.active_reservations.read().await;
            reservations.values().map(|r| r.receipt.reserved_cost).sum()
        };

        // Calculate pending reserved cost for this specific entity
        let entity_active_reserved: f64 = {
            let reservations = self.active_reservations.read().await;
            reservations
                .values()
                .filter(|r| r.receipt.entity_id == entity_id)
                .map(|r| r.receipt.reserved_cost)
                .sum()
        };

        // 1. Check daily budget
        {
            let mut daily = self.spent_today.write().await;
            if daily.0 != now.ordinal() {
                // New day, reset
                *daily = (now.ordinal(), 0.0);
            }
            if daily.1 + total_active_reserved + estimated_cost > cfg.global_daily_cost_limit {
                return Err(format!(
                    "Daily budget exceeded: spent=${:.2}, reserved=${:.2}, requested=${:.2}, limit=${:.2}",
                    daily.1, total_active_reserved, estimated_cost, cfg.global_daily_cost_limit
                ));
            }
        }

        // 2. Check monthly budget
        {
            let mut monthly = self.spent_month.write().await;
            if monthly.0 != now.month() {
                // New month, reset
                *monthly = (now.month(), 0.0);
            }
            if monthly.1 + total_active_reserved + estimated_cost > cfg.global_monthly_cost_limit {
                return Err(format!(
                    "Monthly budget exceeded: spent=${:.2}, limit=${:.2}",
                    monthly.1, cfg.global_monthly_cost_limit
                ));
            }
        }

        // 3. Check workflow budget if entity is a workflow
        if entity_type == "Workflow" {
            let wf_map = self.workflow_spent.read().await;
            let current_wf_spent = wf_map.get(entity_id).copied().unwrap_or(0.0);
            if current_wf_spent + entity_active_reserved + estimated_cost > cfg.default_workflow_cost_limit {
                return Err(format!(
                    "Workflow budget exceeded for {}: spent=${:.2}, reserved=${:.2}, requested=${:.2}, limit=${:.2}",
                    entity_id, current_wf_spent, entity_active_reserved, estimated_cost, cfg.default_workflow_cost_limit
                ));
            }
        }

        // Create receipt
        let receipt = ReservationReceipt {
            id: Uuid::new_v4(),
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            reserved_tokens: estimated_tokens,
            reserved_cost: estimated_cost,
            created_at: now,
        };

        let mut reservations = self.active_reservations.write().await;
        reservations.insert(
            receipt.id,
            ActiveReservation {
                receipt: receipt.clone(),
                expires_at: now + chrono::Duration::minutes(5),
            },
        );

        Ok(receipt)
    }

    /// Reconciles actual consumption after model response completes
    pub async fn reconcile(
        &self,
        receipt: &ReservationReceipt,
        _actual_tokens: u64,
        actual_cost: f64,
    ) {
        let now = Utc::now();

        // Remove from active reservations
        {
            let mut reservations = self.active_reservations.write().await;
            reservations.remove(&receipt.id);
        }

        // Update daily spent
        {
            let mut daily = self.spent_today.write().await;
            if daily.0 == now.ordinal() {
                daily.1 += actual_cost;
            } else {
                *daily = (now.ordinal(), actual_cost);
            }
        }

        // Update monthly spent
        {
            let mut monthly = self.spent_month.write().await;
            if monthly.0 == now.month() {
                monthly.1 += actual_cost;
            } else {
                *monthly = (now.month(), actual_cost);
            }
        }

        // Update workflow spent
        if receipt.entity_type == "Workflow" {
            let mut wf_map = self.workflow_spent.write().await;
            *wf_map.entry(receipt.entity_id.clone()).or_insert(0.0) += actual_cost;
        }
    }

    /// Releases reservation if the request was cancelled or failed before sending
    pub async fn release(&self, receipt_id: Uuid) {
        let mut reservations = self.active_reservations.write().await;
        reservations.remove(&receipt_id);
    }

    pub async fn get_daily_spent(&self) -> f64 {
        self.spent_today.read().await.1
    }

    pub async fn get_active_reserved_sum(&self) -> f64 {
        let reservations = self.active_reservations.read().await;
        reservations.values().map(|r| r.receipt.reserved_cost).sum()
    }
}
