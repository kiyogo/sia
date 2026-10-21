use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub id: String,
    pub thread_id: Option<String>,
    pub sender: String,
    pub recipients: Vec<String>,
    pub subject: String,
    pub snippet: String,
    pub body: String,
    pub date: DateTime<Utc>,
    pub has_attachments: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDraftRequest {
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body: String,
    pub operation_id: String,
}

#[async_trait]
pub trait EmailProviderAdapter: Send + Sync {
    fn provider_name(&self) -> &str;
    async fn is_connected(&self) -> bool;
    async fn search_messages(&self, query: &str, limit: usize) -> Result<Vec<EmailMessage>, String>;
    async fn get_message(&self, message_id: &str) -> Result<EmailMessage, String>;
    async fn create_draft(&self, draft: &EmailDraftRequest) -> Result<String, String>;
    async fn send_message_with_approval(&self, draft_id: &str, approval_token: &str) -> Result<String, String>;
}

/// In-memory test email provider for deterministic verification
pub struct MockTestEmailProvider {
    connected: bool,
    messages: Arc<Mutex<HashMap<String, EmailMessage>>>,
    drafts: Arc<Mutex<HashMap<String, EmailDraftRequest>>>,
    sent_operation_ids: Arc<Mutex<HashSet<String>>>,
    valid_approval_tokens: Arc<Mutex<HashSet<String>>>,
}

impl MockTestEmailProvider {
    pub fn new() -> Self {
        let mut initial_messages = HashMap::new();
        let m1 = EmailMessage {
            id: "msg_101".to_string(),
            thread_id: Some("thread_01".to_string()),
            sender: "vp_strategy@enterprise.com".to_string(),
            recipients: vec!["analyst@workspace.ai".to_string()],
            subject: "Q3 Market Research Deliverable Required".to_string(),
            snippet: "Can your team analyze competitor pricing and send over a draft by tomorrow?".to_string(),
            body: "Hi team,\n\nPlease run a competitive analysis on our top 3 competitors, analyze their pricing sheets, and prepare a summary report.\n\nThanks,\nVP Strategy".to_string(),
            date: Utc::now() - chrono::Duration::hours(4),
            has_attachments: true,
        };
        initial_messages.insert(m1.id.clone(), m1);

        Self {
            connected: true,
            messages: Arc::new(Mutex::new(initial_messages)),
            drafts: Arc::new(Mutex::new(HashMap::new())),
            sent_operation_ids: Arc::new(Mutex::new(HashSet::new())),
            valid_approval_tokens: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn add_valid_approval_token(&self, token: &str) {
        self.valid_approval_tokens.lock().await.insert(token.to_string());
    }

    pub async fn get_draft(&self, draft_id: &str) -> Option<EmailDraftRequest> {
        self.drafts.lock().await.get(draft_id).cloned()
    }
}

#[async_trait]
impl EmailProviderAdapter for MockTestEmailProvider {
    fn provider_name(&self) -> &str {
        "MockTestEmailProvider"
    }

    async fn is_connected(&self) -> bool {
        self.connected
    }

    async fn search_messages(&self, query: &str, limit: usize) -> Result<Vec<EmailMessage>, String> {
        let q_lower = query.to_lowercase();
        let msgs = self.messages.lock().await;

        let mut results: Vec<EmailMessage> = msgs
            .values()
            .filter(|m| {
                if q_lower.is_empty() {
                    return true;
                }
                m.subject.to_lowercase().contains(&q_lower)
                    || m.sender.to_lowercase().contains(&q_lower)
                    || m.snippet.to_lowercase().contains(&q_lower)
            })
            .cloned()
            .collect();

        results.truncate(limit);
        Ok(results)
    }

    async fn get_message(&self, message_id: &str) -> Result<EmailMessage, String> {
        let msgs = self.messages.lock().await;
        msgs.get(message_id)
            .cloned()
            .ok_or_else(|| format!("Message '{}' not found", message_id))
    }

    async fn create_draft(&self, draft: &EmailDraftRequest) -> Result<String, String> {
        if draft.to.is_empty() {
            return Err("Cannot create draft: 'to' recipient list is empty".to_string());
        }

        let draft_id = format!("draft_{}", Uuid::new_v4().simple());
        let mut drafts = self.drafts.lock().await;
        drafts.insert(draft_id.clone(), draft.clone());

        Ok(draft_id)
    }

    async fn send_message_with_approval(&self, draft_id: &str, approval_token: &str) -> Result<String, String> {
        // 1. Strict approval check
        let valid_tokens = self.valid_approval_tokens.lock().await;
        if !valid_tokens.contains(approval_token) {
            return Err("Action rejected: Sending email requires valid human approval token".to_string());
        }

        // 2. Draft existence
        let drafts = self.drafts.lock().await;
        let draft = drafts
            .get(draft_id)
            .ok_or_else(|| format!("Draft '{}' not found", draft_id))?;

        // 3. Idempotency check
        let mut sent_ops = self.sent_operation_ids.lock().await;
        if sent_ops.contains(&draft.operation_id) {
            return Err(format!(
                "Duplicate send prevented: Operation '{}' has already been dispatched",
                draft.operation_id
            ));
        }

        sent_ops.insert(draft.operation_id.clone());
        let sent_id = format!("sent_{}", Uuid::new_v4().simple());
        Ok(sent_id)
    }
}
