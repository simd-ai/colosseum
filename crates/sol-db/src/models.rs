use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row for a GPU provider.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProviderRow {
    pub id: Uuid,
    pub pubkey: String,
    pub name: String,
    pub gpu_class: String,
    pub gpu_count: i16,
    pub max_scu_per_epoch: i64,
    pub status: String,
    pub total_jobs_completed: i64,
    pub total_scu_delivered: i64,
    pub onchain_tx: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database row for a compute job.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct JobRow {
    pub id: Uuid,
    pub client_pubkey: String,
    pub provider_id: Option<Uuid>,
    pub gpu_class: String,
    pub gpu_count: i16,
    pub max_duration_sec: i32,
    pub budget_scu: i64,
    pub status: String,
    pub assigned_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub escrow_tx: Option<String>,
    pub settlement_tx: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database row for a compute receipt.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ReceiptRow {
    pub id: Uuid,
    pub job_id: Uuid,
    pub provider_pubkey: String,
    pub gpu_class: String,
    pub gpu_count: i16,
    pub execution_duration_sec: i32,
    pub scu_amount: i64,
    pub result_hash: String,
    pub provider_signature: String,
    pub verified: bool,
    pub verifier_pubkey: Option<String>,
    pub verification_tx: Option<String>,
    pub onchain_tx: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Database row for transaction tracking.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TransactionRow {
    pub id: Uuid,
    pub tx_hash: String,
    pub tx_type: String,
    pub reference_id: Uuid,
    pub status: String,
    pub retry_count: i32,
    pub error_message: Option<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
