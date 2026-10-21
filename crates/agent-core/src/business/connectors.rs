use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use agent_storage::{BusinessConnector, ConnectorStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorRateLimiter {
    pub max_requests_per_minute: u32,
    pub current_minute_requests: u32,
    pub window_start: DateTime<Utc>,
    pub circuit_breaker_tripped: bool,
}

impl ConnectorRateLimiter {
    pub fn new(max_per_minute: u32) -> Self {
        Self {
            max_requests_per_minute: max_per_minute,
            current_minute_requests: 0,
            window_start: Utc::now(),
            circuit_breaker_tripped: false,
        }
    }

    pub fn acquire_permission(&mut self) -> Result<(), String> {
        if self.circuit_breaker_tripped {
            return Err("Connector circuit breaker is active due to consecutive errors".to_string());
        }

        let now = Utc::now();
        if (now - self.window_start).num_seconds() >= 60 {
            self.window_start = now;
            self.current_minute_requests = 0;
        }

        if self.current_minute_requests >= self.max_requests_per_minute {
            return Err(format!(
                "Rate limit exceeded: {} requests per minute reached",
                self.max_requests_per_minute
            ));
        }

        self.current_minute_requests += 1;
        Ok(())
    }

    pub fn trip_circuit_breaker(&mut self) {
        self.circuit_breaker_tripped = true;
    }

    pub fn reset_circuit_breaker(&mut self) {
        self.circuit_breaker_tripped = false;
        self.current_minute_requests = 0;
        self.window_start = Utc::now();
    }
}

pub struct BusinessConnectorRegistry {
    connectors: HashMap<Uuid, BusinessConnector>,
    rate_limiters: HashMap<Uuid, ConnectorRateLimiter>,
}

impl BusinessConnectorRegistry {
    pub fn new() -> Self {
        Self {
            connectors: HashMap::new(),
            rate_limiters: HashMap::new(),
        }
    }

    pub fn register_connector(&mut self, connector: BusinessConnector) -> Uuid {
        let id = connector.id;
        let rate_limit = connector.rate_limit_per_minute;
        self.connectors.insert(id, connector);
        self.rate_limiters.insert(id, ConnectorRateLimiter::new(rate_limit));
        id
    }

    pub fn get_connector(&self, id: Uuid) -> Option<&BusinessConnector> {
        self.connectors.get(&id)
    }

    pub fn get_by_provider(&self, provider: &str) -> Option<&BusinessConnector> {
        self.connectors
            .values()
            .find(|c| c.provider.eq_ignore_ascii_case(provider))
    }

    pub fn check_rate_limit(&mut self, connector_id: Uuid) -> Result<(), String> {
        if let Some(limiter) = self.rate_limiters.get_mut(&connector_id) {
            limiter.acquire_permission()
        } else {
            Ok(())
        }
    }

    pub fn update_status(&mut self, connector_id: Uuid, status: ConnectorStatus) {
        if let Some(c) = self.connectors.get_mut(&connector_id) {
            c.status = status;
        }
    }

    pub fn list_connectors(&self) -> Vec<&BusinessConnector> {
        self.connectors.values().collect()
    }
}
