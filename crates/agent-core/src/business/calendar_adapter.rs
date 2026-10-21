use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub attendees: Vec<String>,
    pub is_confirmed: bool,
}

#[async_trait]
pub trait CalendarProviderAdapter: Send + Sync {
    fn provider_name(&self) -> &str;
    async fn is_connected(&self) -> bool;
    async fn list_events(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<CalendarEvent>, String>;
    async fn check_conflicts(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<CalendarEvent>, String>;
    async fn draft_event(&self, event: &CalendarEvent) -> Result<String, String>;
    async fn create_event_with_approval(&self, event_id: &str, approval_token: &str) -> Result<String, String>;
}

pub struct MockTestCalendarProvider {
    connected: bool,
    events: Arc<Mutex<HashMap<String, CalendarEvent>>>,
    valid_approval_tokens: Arc<Mutex<HashSet<String>>>,
}

impl MockTestCalendarProvider {
    pub fn new() -> Self {
        let mut initial = HashMap::new();
        let now = Utc::now();
        let e1 = CalendarEvent {
            id: "evt_101".to_string(),
            title: "Sprint Planning".to_string(),
            description: Some("Bi-weekly planning session".to_string()),
            location: Some("Room A / Meet".to_string()),
            start_time: now + chrono::Duration::hours(2),
            end_time: now + chrono::Duration::hours(3),
            attendees: vec!["lead@company.com".to_string(), "analyst@company.com".to_string()],
            is_confirmed: true,
        };
        initial.insert(e1.id.clone(), e1);

        Self {
            connected: true,
            events: Arc::new(Mutex::new(initial)),
            valid_approval_tokens: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn add_valid_approval_token(&self, token: &str) {
        self.valid_approval_tokens.lock().await.insert(token.to_string());
    }
}

#[async_trait]
impl CalendarProviderAdapter for MockTestCalendarProvider {
    fn provider_name(&self) -> &str {
        "MockTestCalendarProvider"
    }

    async fn is_connected(&self) -> bool {
        self.connected
    }

    async fn list_events(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<CalendarEvent>, String> {
        let events = self.events.lock().await;
        let matched = events
            .values()
            .filter(|e| e.end_time >= start && e.start_time <= end)
            .cloned()
            .collect();
        Ok(matched)
    }

    async fn check_conflicts(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<CalendarEvent>, String> {
        let events = self.events.lock().await;
        let conflicts = events
            .values()
            .filter(|e| e.is_confirmed && e.end_time > start && e.start_time < end)
            .cloned()
            .collect();
        Ok(conflicts)
    }

    async fn draft_event(&self, event: &CalendarEvent) -> Result<String, String> {
        let id = format!("evt_{}", Uuid::new_v4().simple());
        let mut event_draft = event.clone();
        event_draft.id = id.clone();
        event_draft.is_confirmed = false;

        let mut events = self.events.lock().await;
        events.insert(id.clone(), event_draft);
        Ok(id)
    }

    async fn create_event_with_approval(&self, event_id: &str, approval_token: &str) -> Result<String, String> {
        let tokens = self.valid_approval_tokens.lock().await;
        if !tokens.contains(approval_token) {
            return Err("Action rejected: Creating calendar event with attendees requires human approval".to_string());
        }

        let mut events = self.events.lock().await;
        let evt = events
            .get_mut(event_id)
            .ok_or_else(|| format!("Event '{}' not found", event_id))?;

        evt.is_confirmed = true;
        Ok(evt.id.clone())
    }
}
