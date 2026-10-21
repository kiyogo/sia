use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Database, StorageError, StorageResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceType {
    Web,
    WorkspaceFile,
    Api,
    Document,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::Web => write!(f, "Web"),
            SourceType::WorkspaceFile => write!(f, "WorkspaceFile"),
            SourceType::Api => write!(f, "Api"),
            SourceType::Document => write!(f, "Document"),
        }
    }
}

impl std::str::FromStr for SourceType {
    type Err = StorageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Web" => Ok(SourceType::Web),
            "WorkspaceFile" => Ok(SourceType::WorkspaceFile),
            "Api" => Ok(SourceType::Api),
            "Document" => Ok(SourceType::Document),
            _ => Ok(SourceType::Web),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Contradicted,
    Outdated,
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationStatus::Verified => write!(f, "Verified"),
            VerificationStatus::Unverified => write!(f, "Unverified"),
            VerificationStatus::Contradicted => write!(f, "Contradicted"),
            VerificationStatus::Outdated => write!(f, "Outdated"),
        }
    }
}

impl std::str::FromStr for VerificationStatus {
    type Err = StorageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Verified" => Ok(VerificationStatus::Verified),
            "Unverified" => Ok(VerificationStatus::Unverified),
            "Contradicted" => Ok(VerificationStatus::Contradicted),
            "Outdated" => Ok(VerificationStatus::Outdated),
            _ => Ok(VerificationStatus::Unverified),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSource {
    pub id: Uuid,
    pub url_or_path: String,
    pub title: String,
    pub source_type: SourceType,
    pub retrieved_at: DateTime<Utc>,
    pub published_date: Option<String>,
    pub content_hash: String,
    pub verification_status: VerificationStatus,
    pub raw_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvidenceType {
    Fact,
    Claim,
    Inference,
    Estimate,
    Opinion,
}

impl std::fmt::Display for EvidenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvidenceType::Fact => write!(f, "Fact"),
            EvidenceType::Claim => write!(f, "Claim"),
            EvidenceType::Inference => write!(f, "Inference"),
            EvidenceType::Estimate => write!(f, "Estimate"),
            EvidenceType::Opinion => write!(f, "Opinion"),
        }
    }
}

impl std::str::FromStr for EvidenceType {
    type Err = StorageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Fact" => Ok(EvidenceType::Fact),
            "Claim" => Ok(EvidenceType::Claim),
            "Inference" => Ok(EvidenceType::Inference),
            "Estimate" => Ok(EvidenceType::Estimate),
            "Opinion" => Ok(EvidenceType::Opinion),
            _ => Ok(EvidenceType::Claim),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: Uuid,
    pub source_id: Uuid,
    pub task_id: Option<Uuid>,
    pub claim_text: String,
    pub evidence_type: EvidenceType,
    pub page_or_section: Option<String>,
    pub confidence_score: f64,
    pub verification_status: VerificationStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: Uuid,
    pub file_path: String,
    pub format: String,
    pub file_size_bytes: u64,
    pub title: Option<String>,
    pub page_count: Option<u32>,
    pub word_count: Option<u32>,
    pub table_count: Option<u32>,
    pub content_hash: String,
    pub parsed_summary: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectorStatus {
    Connected,
    Disconnected,
    PendingAuth,
    RateLimited,
    Error,
}

impl std::fmt::Display for ConnectorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectorStatus::Connected => write!(f, "Connected"),
            ConnectorStatus::Disconnected => write!(f, "Disconnected"),
            ConnectorStatus::PendingAuth => write!(f, "PendingAuth"),
            ConnectorStatus::RateLimited => write!(f, "RateLimited"),
            ConnectorStatus::Error => write!(f, "Error"),
        }
    }
}

impl std::str::FromStr for ConnectorStatus {
    type Err = StorageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Connected" => Ok(ConnectorStatus::Connected),
            "Disconnected" => Ok(ConnectorStatus::Disconnected),
            "PendingAuth" => Ok(ConnectorStatus::PendingAuth),
            "RateLimited" => Ok(ConnectorStatus::RateLimited),
            "Error" => Ok(ConnectorStatus::Error),
            _ => Ok(ConnectorStatus::Disconnected),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessConnector {
    pub id: Uuid,
    pub provider: String,
    pub auth_type: String,
    pub status: ConnectorStatus,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: u32,
    pub last_connected_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DraftStatus {
    Draft,
    PendingApproval,
    Approved,
    Sent,
    Cancelled,
}

impl std::fmt::Display for DraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DraftStatus::Draft => write!(f, "Draft"),
            DraftStatus::PendingApproval => write!(f, "PendingApproval"),
            DraftStatus::Approved => write!(f, "Approved"),
            DraftStatus::Sent => write!(f, "Sent"),
            DraftStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl std::str::FromStr for DraftStatus {
    type Err = StorageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Draft" => Ok(DraftStatus::Draft),
            "PendingApproval" => Ok(DraftStatus::PendingApproval),
            "Approved" => Ok(DraftStatus::Approved),
            "Sent" => Ok(DraftStatus::Sent),
            "Cancelled" => Ok(DraftStatus::Cancelled),
            _ => Ok(DraftStatus::Draft),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDraft {
    pub id: Uuid,
    pub connector_id: Uuid,
    pub thread_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body: String,
    pub status: DraftStatus,
    pub operation_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalStatus::Pending => write!(f, "Pending"),
            ApprovalStatus::Approved => write!(f, "Approved"),
            ApprovalStatus::Rejected => write!(f, "Rejected"),
            ApprovalStatus::Expired => write!(f, "Expired"),
        }
    }
}

impl std::str::FromStr for ApprovalStatus {
    type Err = StorageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(ApprovalStatus::Pending),
            "Approved" => Ok(ApprovalStatus::Approved),
            "Rejected" => Ok(ApprovalStatus::Rejected),
            "Expired" => Ok(ApprovalStatus::Expired),
            _ => Ok(ApprovalStatus::Pending),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessApproval {
    pub id: Uuid,
    pub workflow_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub action_type: String,
    pub target_resource: String,
    pub payload_preview: String,
    pub status: ApprovalStatus,
    pub approved_by: Option<String>,
    pub decision_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

impl Database {
    // ---------------------------------------------------------
    // Research Sources
    // ---------------------------------------------------------
    pub async fn create_research_source(&self, source: &ResearchSource) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"
            INSERT INTO research_sources (
                id, url_or_path, title, source_type, retrieved_at, published_date,
                content_hash, verification_status, raw_snippet
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                source.id.to_string(),
                source.url_or_path,
                source.title,
                source.source_type.to_string(),
                source.retrieved_at.to_rfc3339(),
                source.published_date,
                source.content_hash,
                source.verification_status.to_string(),
                source.raw_snippet,
            ],
        )?;
        Ok(())
    }

    pub async fn get_research_source(&self, id: Uuid) -> StorageResult<ResearchSource> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, url_or_path, title, source_type, retrieved_at, published_date,
                   content_hash, verification_status, raw_snippet
            FROM research_sources WHERE id = ?1
            "#,
        )?;
        let row = stmt.query_row(params![id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let source_type_str: String = row.get(3)?;
            let retrieved_at_str: String = row.get(4)?;
            let status_str: String = row.get(7)?;
            Ok((
                id_str,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                source_type_str,
                retrieved_at_str,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                status_str,
                row.get::<_, Option<String>>(8)?,
            ))
        })?;

        Ok(ResearchSource {
            id: Uuid::parse_str(&row.0).map_err(|_| StorageError::NotFound(row.0))?,
            url_or_path: row.1,
            title: row.2,
            source_type: row.3.parse().unwrap_or(SourceType::Web),
            retrieved_at: DateTime::parse_from_rfc3339(&row.4)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            published_date: row.5,
            content_hash: row.6,
            verification_status: row.7.parse().unwrap_or(VerificationStatus::Unverified),
            raw_snippet: row.8,
        })
    }

    pub async fn list_research_sources(&self) -> StorageResult<Vec<ResearchSource>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, url_or_path, title, source_type, retrieved_at, published_date,
                   content_hash, verification_status, raw_snippet
            FROM research_sources ORDER BY retrieved_at DESC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let source_type_str: String = row.get(3)?;
            let retrieved_at_str: String = row.get(4)?;
            let status_str: String = row.get(7)?;
            Ok((
                id_str,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                source_type_str,
                retrieved_at_str,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                status_str,
                row.get::<_, Option<String>>(8)?,
            ))
        })?;

        let mut sources = Vec::new();
        for r in rows {
            let row = r?;
            sources.push(ResearchSource {
                id: Uuid::parse_str(&row.0).map_err(|_| StorageError::NotFound(row.0))?,
                url_or_path: row.1,
                title: row.2,
                source_type: row.3.parse().unwrap_or(SourceType::Web),
                retrieved_at: DateTime::parse_from_rfc3339(&row.4)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                published_date: row.5,
                content_hash: row.6,
                verification_status: row.7.parse().unwrap_or(VerificationStatus::Unverified),
                raw_snippet: row.8,
            });
        }
        Ok(sources)
    }

    // ---------------------------------------------------------
    // Evidence Items
    // ---------------------------------------------------------
    pub async fn create_evidence_item(&self, evidence: &EvidenceItem) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"
            INSERT INTO evidence_items (
                id, source_id, task_id, claim_text, evidence_type, page_or_section,
                confidence_score, verification_status, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                evidence.id.to_string(),
                evidence.source_id.to_string(),
                evidence.task_id.map(|u| u.to_string()),
                evidence.claim_text,
                evidence.evidence_type.to_string(),
                evidence.page_or_section,
                evidence.confidence_score,
                evidence.verification_status.to_string(),
                evidence.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub async fn list_evidence_for_source(&self, source_id: Uuid) -> StorageResult<Vec<EvidenceItem>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, source_id, task_id, claim_text, evidence_type, page_or_section,
                   confidence_score, verification_status, created_at
            FROM evidence_items WHERE source_id = ?1 ORDER BY created_at ASC
            "#,
        )?;
        let rows = stmt.query_map(params![source_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let src_id_str: String = row.get(1)?;
            let task_id_str: Option<String> = row.get(2)?;
            let type_str: String = row.get(4)?;
            let status_str: String = row.get(7)?;
            let created_str: String = row.get(8)?;
            Ok((
                id_str,
                src_id_str,
                task_id_str,
                row.get::<_, String>(3)?,
                type_str,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, f64>(6)?,
                status_str,
                created_str,
            ))
        })?;

        let mut items = Vec::new();
        for r in rows {
            let row = r?;
            items.push(EvidenceItem {
                id: Uuid::parse_str(&row.0).map_err(|_| StorageError::NotFound(row.0))?,
                source_id: Uuid::parse_str(&row.1).map_err(|_| StorageError::NotFound(row.1))?,
                task_id: row.2.and_then(|s| Uuid::parse_str(&s).ok()),
                claim_text: row.3,
                evidence_type: row.4.parse().unwrap_or(EvidenceType::Claim),
                page_or_section: row.5,
                confidence_score: row.6,
                verification_status: row.7.parse().unwrap_or(VerificationStatus::Unverified),
                created_at: DateTime::parse_from_rfc3339(&row.8)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            });
        }
        Ok(items)
    }

    // ---------------------------------------------------------
    // Document Metadata
    // ---------------------------------------------------------
    pub async fn create_document_metadata(&self, doc: &DocumentMetadata) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"
            INSERT INTO document_metadata (
                id, file_path, format, file_size_bytes, title, page_count,
                word_count, table_count, content_hash, parsed_summary, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                doc.id.to_string(),
                doc.file_path,
                doc.format,
                doc.file_size_bytes as i64,
                doc.title,
                doc.page_count,
                doc.word_count,
                doc.table_count,
                doc.content_hash,
                doc.parsed_summary,
                doc.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub async fn list_documents(&self) -> StorageResult<Vec<DocumentMetadata>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, file_path, format, file_size_bytes, title, page_count,
                   word_count, table_count, content_hash, parsed_summary, created_at
            FROM document_metadata ORDER BY created_at DESC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let created_str: String = row.get(10)?;
            let size_bytes: i64 = row.get(3)?;
            Ok((
                id_str,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                size_bytes as u64,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<u32>>(5)?,
                row.get::<_, Option<u32>>(6)?,
                row.get::<_, Option<u32>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
                created_str,
            ))
        })?;

        let mut docs = Vec::new();
        for r in rows {
            let row = r?;
            docs.push(DocumentMetadata {
                id: Uuid::parse_str(&row.0).map_err(|_| StorageError::NotFound(row.0))?,
                file_path: row.1,
                format: row.2,
                file_size_bytes: row.3,
                title: row.4,
                page_count: row.5,
                word_count: row.6,
                table_count: row.7,
                content_hash: row.8,
                parsed_summary: row.9,
                created_at: DateTime::parse_from_rfc3339(&row.10)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            });
        }
        Ok(docs)
    }

    // ---------------------------------------------------------
    // Business Connectors
    // ---------------------------------------------------------
    pub async fn upsert_business_connector(&self, connector: &BusinessConnector) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        let scopes_json = serde_json::to_string(&connector.scopes)?;
        conn.execute(
            r#"
            INSERT INTO business_connectors (
                id, provider, auth_type, status, scopes_json, rate_limit_per_minute,
                last_connected_at, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                scopes_json = excluded.scopes_json,
                rate_limit_per_minute = excluded.rate_limit_per_minute,
                last_connected_at = excluded.last_connected_at
            "#,
            params![
                connector.id.to_string(),
                connector.provider,
                connector.auth_type,
                connector.status.to_string(),
                scopes_json,
                connector.rate_limit_per_minute,
                connector.last_connected_at.map(|dt| dt.to_rfc3339()),
                connector.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub async fn list_business_connectors(&self) -> StorageResult<Vec<BusinessConnector>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, provider, auth_type, status, scopes_json, rate_limit_per_minute,
                   last_connected_at, created_at
            FROM business_connectors ORDER BY created_at ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let status_str: String = row.get(3)?;
            let scopes_str: String = row.get(4)?;
            let last_conn_str: Option<String> = row.get(6)?;
            let created_str: String = row.get(7)?;
            Ok((
                id_str,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                status_str,
                scopes_str,
                row.get::<_, u32>(5)?,
                last_conn_str,
                created_str,
            ))
        })?;

        let mut connectors = Vec::new();
        for r in rows {
            let row = r?;
            let scopes: Vec<String> = serde_json::from_str(&row.4).unwrap_or_default();
            connectors.push(BusinessConnector {
                id: Uuid::parse_str(&row.0).map_err(|_| StorageError::NotFound(row.0))?,
                provider: row.1,
                auth_type: row.2,
                status: row.3.parse().unwrap_or(ConnectorStatus::Disconnected),
                scopes,
                rate_limit_per_minute: row.5,
                last_connected_at: row.6.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                created_at: DateTime::parse_from_rfc3339(&row.7)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            });
        }
        Ok(connectors)
    }

    // ---------------------------------------------------------
    // Email Drafts
    // ---------------------------------------------------------
    pub async fn create_email_draft(&self, draft: &EmailDraft) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"
            INSERT INTO email_drafts (
                id, connector_id, thread_id, to_recipients_json, cc_recipients_json,
                bcc_recipients_json, subject, body, status, operation_id, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                draft.id.to_string(),
                draft.connector_id.to_string(),
                draft.thread_id,
                serde_json::to_string(&draft.to_recipients)?,
                serde_json::to_string(&draft.cc_recipients)?,
                serde_json::to_string(&draft.bcc_recipients)?,
                draft.subject,
                draft.body,
                draft.status.to_string(),
                draft.operation_id,
                draft.created_at.to_rfc3339(),
                draft.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub async fn get_email_draft_by_operation_id(&self, op_id: &str) -> StorageResult<Option<EmailDraft>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, connector_id, thread_id, to_recipients_json, cc_recipients_json,
                   bcc_recipients_json, subject, body, status, operation_id, created_at, updated_at
            FROM email_drafts WHERE operation_id = ?1
            "#,
        )?;
        let mut rows = stmt.query(params![op_id])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let conn_id_str: String = row.get(1)?;
            let to_json: String = row.get(3)?;
            let cc_json: String = row.get(4)?;
            let bcc_json: String = row.get(5)?;
            let status_str: String = row.get(8)?;
            let created_str: String = row.get(10)?;
            let updated_str: String = row.get(11)?;

            Ok(Some(EmailDraft {
                id: Uuid::parse_str(&id_str).map_err(|_| StorageError::NotFound(id_str))?,
                connector_id: Uuid::parse_str(&conn_id_str).map_err(|_| StorageError::NotFound(conn_id_str))?,
                thread_id: row.get(2)?,
                to_recipients: serde_json::from_str(&to_json).unwrap_or_default(),
                cc_recipients: serde_json::from_str(&cc_json).unwrap_or_default(),
                bcc_recipients: serde_json::from_str(&bcc_json).unwrap_or_default(),
                subject: row.get(6)?,
                body: row.get(7)?,
                status: status_str.parse().unwrap_or(DraftStatus::Draft),
                operation_id: row.get(9)?,
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_email_draft_status(&self, id: Uuid, status: DraftStatus) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE email_drafts SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.to_string(), Utc::now().to_rfc3339(), id.to_string()],
        )?;
        Ok(())
    }

    // ---------------------------------------------------------
    // Business Approvals
    // ---------------------------------------------------------
    pub async fn create_business_approval(&self, approval: &BusinessApproval) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"
            INSERT INTO business_approvals (
                id, workflow_id, task_id, action_type, target_resource, payload_preview,
                status, approved_by, decision_reason, created_at, resolved_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                approval.id.to_string(),
                approval.workflow_id.map(|u| u.to_string()),
                approval.task_id.map(|u| u.to_string()),
                approval.action_type,
                approval.target_resource,
                approval.payload_preview,
                approval.status.to_string(),
                approval.approved_by,
                approval.decision_reason,
                approval.created_at.to_rfc3339(),
                approval.resolved_at.map(|dt| dt.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    pub async fn resolve_business_approval(
        &self,
        id: Uuid,
        status: ApprovalStatus,
        approved_by: &str,
        decision_reason: Option<&str>,
    ) -> StorageResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"
            UPDATE business_approvals
            SET status = ?1, approved_by = ?2, decision_reason = ?3, resolved_at = ?4
            WHERE id = ?5
            "#,
            params![
                status.to_string(),
                approved_by,
                decision_reason,
                Utc::now().to_rfc3339(),
                id.to_string(),
            ],
        )?;
        Ok(())
    }

    pub async fn list_pending_approvals(&self) -> StorageResult<Vec<BusinessApproval>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, workflow_id, task_id, action_type, target_resource, payload_preview,
                   status, approved_by, decision_reason, created_at, resolved_at
            FROM business_approvals WHERE status = 'Pending' ORDER BY created_at ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let wf_str: Option<String> = row.get(1)?;
            let task_str: Option<String> = row.get(2)?;
            let status_str: String = row.get(6)?;
            let created_str: String = row.get(9)?;
            let resolved_str: Option<String> = row.get(10)?;
            Ok((
                id_str,
                wf_str,
                task_str,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                status_str,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                created_str,
                resolved_str,
            ))
        })?;

        let mut approvals = Vec::new();
        for r in rows {
            let row = r?;
            approvals.push(BusinessApproval {
                id: Uuid::parse_str(&row.0).map_err(|_| StorageError::NotFound(row.0))?,
                workflow_id: row.1.and_then(|s| Uuid::parse_str(&s).ok()),
                task_id: row.2.and_then(|s| Uuid::parse_str(&s).ok()),
                action_type: row.3,
                target_resource: row.4,
                payload_preview: row.5,
                status: row.6.parse().unwrap_or(ApprovalStatus::Pending),
                approved_by: row.7,
                decision_reason: row.8,
                created_at: DateTime::parse_from_rfc3339(&row.9)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                resolved_at: row.10.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
            });
        }
        Ok(approvals)
    }
}
