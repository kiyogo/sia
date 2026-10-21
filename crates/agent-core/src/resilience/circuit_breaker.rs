use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    cooldown: Duration,
    state: Mutex<(CircuitState, u32, Option<Instant>)>, // (state, consecutive_failures, last_failure)
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown_secs: u64) -> Self {
        Self {
            failure_threshold,
            cooldown: Duration::from_secs(cooldown_secs),
            state: Mutex::new((CircuitState::Closed, 0, None)),
        }
    }

    pub fn can_execute(&self) -> bool {
        let mut guard = self.state.lock().unwrap();
        match guard.0 {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_fail) = guard.2 {
                    if last_fail.elapsed() >= self.cooldown {
                        // Transition to HalfOpen to probe
                        guard.0 = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&self) {
        let mut guard = self.state.lock().unwrap();
        guard.0 = CircuitState::Closed;
        guard.1 = 0;
        guard.2 = None;
    }

    pub fn record_failure(&self) {
        let mut guard = self.state.lock().unwrap();
        guard.1 += 1;
        guard.2 = Some(Instant::now());

        if guard.1 >= self.failure_threshold {
            guard.0 = CircuitState::Open;
        }
    }

    pub fn state(&self) -> CircuitState {
        let guard = self.state.lock().unwrap();
        guard.0
    }

    pub fn consecutive_failures(&self) -> u32 {
        let guard = self.state.lock().unwrap();
        guard.1
    }
}
