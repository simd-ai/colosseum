use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ─── Job Status Pipeline ────────────────────────────────────────────

/// Full lifecycle status for a compute job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Assigned,
    Computing,
    ReceiptSubmitted,
    Verified,
    Settled,
    Failed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Assigned => write!(f, "assigned"),
            JobStatus::Computing => write!(f, "computing"),
            JobStatus::ReceiptSubmitted => write!(f, "receipt_submitted"),
            JobStatus::Verified => write!(f, "verified"),
            JobStatus::Settled => write!(f, "settled"),
            JobStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(JobStatus::Pending),
            "assigned" => Ok(JobStatus::Assigned),
            "computing" => Ok(JobStatus::Computing),
            "receipt_submitted" => Ok(JobStatus::ReceiptSubmitted),
            "verified" => Ok(JobStatus::Verified),
            "settled" => Ok(JobStatus::Settled),
            "failed" => Ok(JobStatus::Failed),
            _ => Err(anyhow::anyhow!("invalid job status: {}", s)),
        }
    }
}

// ─── Provider Info ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
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
}

// ─── Job Spec ───────────────────────────────────────────────────────

/// Request to create a new compute job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobRequest {
    pub client_pubkey: String,
    pub gpu_class: String,
    pub gpu_count: i16,
    pub max_duration_sec: i32,
    pub budget_scu: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
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
}

// ─── Compute Receipt (off-chain representation) ─────────────────────

/// The canonical compute receipt format used across all services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeReceiptData {
    pub job_id: Uuid,
    pub provider_pubkey: String,
    pub gpu_class: String,
    pub gpu_count: u8,
    pub execution_duration_sec: u32,
    pub scu_amount: u64,
    pub result_hash: String,
    pub provider_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptInfo {
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

// ─── Provider Registration ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterProviderRequest {
    pub pubkey: String,
    pub name: String,
    pub gpu_class: String,
    pub gpu_count: i16,
    pub max_scu_per_epoch: i64,
}

// ─── Verification ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    pub receipt: ComputeReceiptData,
    pub job: JobInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub approved: bool,
    pub verifier_signature: String,
    pub reason: Option<String>,
}

// ─── Transaction Tracking ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxType {
    RegisterProvider,
    CreateEscrow,
    SubmitReceipt,
    ReleaseEscrow,
    RefundEscrow,
}

impl std::fmt::Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxType::RegisterProvider => write!(f, "register_provider"),
            TxType::CreateEscrow => write!(f, "create_escrow"),
            TxType::SubmitReceipt => write!(f, "submit_receipt"),
            TxType::ReleaseEscrow => write!(f, "release_escrow"),
            TxType::RefundEscrow => write!(f, "refund_escrow"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxRequest {
    pub tx_type: TxType,
    pub reference_id: Uuid,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxResult {
    pub tx_hash: String,
    pub tx_type: TxType,
    pub reference_id: Uuid,
    pub success: bool,
    pub error: Option<String>,
}

// ─── Dashboard Stats ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_providers: i64,
    pub active_providers: i64,
    pub total_jobs: i64,
    pub active_jobs: i64,
    pub settled_jobs: i64,
    pub total_scu_settled: i64,
    pub total_receipts: i64,
    pub verified_receipts: i64,
}

// ─── Telemetry (mock) ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuTelemetry {
    pub provider_pubkey: String,
    pub job_id: Uuid,
    pub gpu_index: u8,
    pub utilization_pct: f32,
    pub memory_used_mb: u32,
    pub memory_total_mb: u32,
    pub temperature_c: u32,
    pub power_draw_w: u32,
    pub timestamp: DateTime<Utc>,
}
