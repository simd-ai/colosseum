use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;

// ─── Provider Queries ───────────────────────────────────────────────

pub async fn insert_provider(
    pool: &PgPool,
    pubkey: &str,
    name: &str,
    gpu_class: &str,
    gpu_count: i16,
    max_scu_per_epoch: i64,
) -> Result<ProviderRow> {
    let row = sqlx::query_as::<_, ProviderRow>(
        r#"INSERT INTO providers (pubkey, name, gpu_class, gpu_count, max_scu_per_epoch)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING *"#,
    )
    .bind(pubkey)
    .bind(name)
    .bind(gpu_class)
    .bind(gpu_count)
    .bind(max_scu_per_epoch)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_provider(pool: &PgPool, id: Uuid) -> Result<Option<ProviderRow>> {
    let row = sqlx::query_as::<_, ProviderRow>("SELECT * FROM providers WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn get_provider_by_pubkey(pool: &PgPool, pubkey: &str) -> Result<Option<ProviderRow>> {
    let row = sqlx::query_as::<_, ProviderRow>("SELECT * FROM providers WHERE pubkey = $1")
        .bind(pubkey)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_providers(pool: &PgPool) -> Result<Vec<ProviderRow>> {
    let rows = sqlx::query_as::<_, ProviderRow>(
        "SELECT * FROM providers ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_active_providers_by_gpu(
    pool: &PgPool,
    gpu_class: &str,
    gpu_count: i16,
) -> Result<Vec<ProviderRow>> {
    let rows = sqlx::query_as::<_, ProviderRow>(
        r#"SELECT * FROM providers
           WHERE status = 'active' AND gpu_class = $1 AND gpu_count >= $2
           ORDER BY total_jobs_completed ASC"#,
    )
    .bind(gpu_class)
    .bind(gpu_count)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn update_provider_onchain_tx(pool: &PgPool, id: Uuid, tx_hash: &str) -> Result<()> {
    sqlx::query("UPDATE providers SET onchain_tx = $1, updated_at = NOW() WHERE id = $2")
        .bind(tx_hash)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn increment_provider_stats(
    pool: &PgPool,
    id: Uuid,
    scu_delivered: i64,
) -> Result<()> {
    sqlx::query(
        r#"UPDATE providers
           SET total_jobs_completed = total_jobs_completed + 1,
               total_scu_delivered = total_scu_delivered + $1,
               updated_at = NOW()
           WHERE id = $2"#,
    )
    .bind(scu_delivered)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

// ─── Job Queries ────────────────────────────────────────────────────

pub async fn insert_job(
    pool: &PgPool,
    client_pubkey: &str,
    gpu_class: &str,
    gpu_count: i16,
    max_duration_sec: i32,
    budget_scu: i64,
) -> Result<JobRow> {
    let row = sqlx::query_as::<_, JobRow>(
        r#"INSERT INTO jobs (client_pubkey, gpu_class, gpu_count, max_duration_sec, budget_scu)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING *"#,
    )
    .bind(client_pubkey)
    .bind(gpu_class)
    .bind(gpu_count)
    .bind(max_duration_sec)
    .bind(budget_scu)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_job(pool: &PgPool, id: Uuid) -> Result<Option<JobRow>> {
    let row = sqlx::query_as::<_, JobRow>("SELECT * FROM jobs WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_jobs(pool: &PgPool) -> Result<Vec<JobRow>> {
    let rows = sqlx::query_as::<_, JobRow>("SELECT * FROM jobs ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn assign_job(pool: &PgPool, job_id: Uuid, provider_id: Uuid) -> Result<JobRow> {
    let row = sqlx::query_as::<_, JobRow>(
        r#"UPDATE jobs
           SET provider_id = $1, status = 'assigned', assigned_at = NOW(), updated_at = NOW()
           WHERE id = $2
           RETURNING *"#,
    )
    .bind(provider_id)
    .bind(job_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update_job_status(pool: &PgPool, id: Uuid, status: &str) -> Result<()> {
    sqlx::query("UPDATE jobs SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_job_escrow_tx(pool: &PgPool, id: Uuid, tx_hash: &str) -> Result<()> {
    sqlx::query("UPDATE jobs SET escrow_tx = $1, updated_at = NOW() WHERE id = $2")
        .bind(tx_hash)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_job_settlement(pool: &PgPool, id: Uuid, tx_hash: &str) -> Result<()> {
    sqlx::query(
        r#"UPDATE jobs
           SET settlement_tx = $1, status = 'settled', completed_at = NOW(), updated_at = NOW()
           WHERE id = $2"#,
    )
    .bind(tx_hash)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_pending_jobs(pool: &PgPool) -> Result<Vec<JobRow>> {
    let rows = sqlx::query_as::<_, JobRow>(
        "SELECT * FROM jobs WHERE status = 'pending' ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ─── Receipt Queries ────────────────────────────────────────────────

pub async fn insert_receipt(
    pool: &PgPool,
    job_id: Uuid,
    provider_pubkey: &str,
    gpu_class: &str,
    gpu_count: i16,
    execution_duration_sec: i32,
    scu_amount: i64,
    result_hash: &str,
    provider_signature: &str,
) -> Result<ReceiptRow> {
    let row = sqlx::query_as::<_, ReceiptRow>(
        r#"INSERT INTO compute_receipts
           (job_id, provider_pubkey, gpu_class, gpu_count, execution_duration_sec,
            scu_amount, result_hash, provider_signature)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING *"#,
    )
    .bind(job_id)
    .bind(provider_pubkey)
    .bind(gpu_class)
    .bind(gpu_count)
    .bind(execution_duration_sec)
    .bind(scu_amount)
    .bind(result_hash)
    .bind(provider_signature)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_receipt_by_job(pool: &PgPool, job_id: Uuid) -> Result<Option<ReceiptRow>> {
    let row = sqlx::query_as::<_, ReceiptRow>(
        "SELECT * FROM compute_receipts WHERE job_id = $1",
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_receipts(pool: &PgPool) -> Result<Vec<ReceiptRow>> {
    let rows = sqlx::query_as::<_, ReceiptRow>(
        "SELECT * FROM compute_receipts ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn verify_receipt(
    pool: &PgPool,
    receipt_id: Uuid,
    verifier_pubkey: &str,
    verification_tx: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"UPDATE compute_receipts
           SET verified = TRUE, verifier_pubkey = $1, verification_tx = $2
           WHERE id = $3"#,
    )
    .bind(verifier_pubkey)
    .bind(verification_tx)
    .bind(receipt_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_receipt_onchain_tx(pool: &PgPool, id: Uuid, tx_hash: &str) -> Result<()> {
    sqlx::query("UPDATE compute_receipts SET onchain_tx = $1 WHERE id = $2")
        .bind(tx_hash)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ─── Transaction Queries ────────────────────────────────────────────

pub async fn insert_transaction(
    pool: &PgPool,
    tx_hash: &str,
    tx_type: &str,
    reference_id: Uuid,
) -> Result<TransactionRow> {
    let row = sqlx::query_as::<_, TransactionRow>(
        r#"INSERT INTO transactions (tx_hash, tx_type, reference_id)
           VALUES ($1, $2, $3)
           RETURNING *"#,
    )
    .bind(tx_hash)
    .bind(tx_type)
    .bind(reference_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update_transaction_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    error: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"UPDATE transactions
           SET status = $1, error_message = $2,
               confirmed_at = CASE WHEN $1 = 'confirmed' THEN NOW() ELSE confirmed_at END
           WHERE id = $3"#,
    )
    .bind(status)
    .bind(error)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn increment_tx_retry(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("UPDATE transactions SET retry_count = retry_count + 1 WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ─── Stats Queries ──────────────────────────────────────────────────

pub async fn get_dashboard_stats(pool: &PgPool) -> Result<sol_common::DashboardStats> {
    let total_providers: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM providers")
            .fetch_one(pool)
            .await?;
    let active_providers: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM providers WHERE status = 'active'")
            .fetch_one(pool)
            .await?;
    let total_jobs: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM jobs")
            .fetch_one(pool)
            .await?;
    let active_jobs: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM jobs WHERE status IN ('assigned', 'computing')")
            .fetch_one(pool)
            .await?;
    let settled_jobs: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM jobs WHERE status = 'settled'")
            .fetch_one(pool)
            .await?;
    let total_scu: (Option<i64>,) =
        sqlx::query_as("SELECT SUM(scu_amount) FROM compute_receipts WHERE verified = TRUE")
            .fetch_one(pool)
            .await?;
    let total_receipts: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM compute_receipts")
            .fetch_one(pool)
            .await?;
    let verified_receipts: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM compute_receipts WHERE verified = TRUE")
            .fetch_one(pool)
            .await?;

    Ok(sol_common::DashboardStats {
        total_providers: total_providers.0,
        active_providers: active_providers.0,
        total_jobs: total_jobs.0,
        active_jobs: active_jobs.0,
        settled_jobs: settled_jobs.0,
        total_scu_settled: total_scu.0.unwrap_or(0),
        total_receipts: total_receipts.0,
        verified_receipts: verified_receipts.0,
    })
}
