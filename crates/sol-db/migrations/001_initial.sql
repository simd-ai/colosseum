-- SolGrid Database Schema
-- PostgreSQL 16+

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ─── Providers ──────────────────────────────────────────────────────

CREATE TABLE providers (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pubkey      VARCHAR(44) UNIQUE NOT NULL,
    name        VARCHAR(64) NOT NULL,
    gpu_class   VARCHAR(16) NOT NULL,
    gpu_count   SMALLINT NOT NULL CHECK (gpu_count >= 1),
    max_scu_per_epoch BIGINT NOT NULL CHECK (max_scu_per_epoch > 0),
    status      VARCHAR(16) NOT NULL DEFAULT 'active',
    total_jobs_completed BIGINT NOT NULL DEFAULT 0,
    total_scu_delivered  BIGINT NOT NULL DEFAULT 0,
    onchain_tx  VARCHAR(88),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_providers_status ON providers(status);
CREATE INDEX idx_providers_gpu_class ON providers(gpu_class);

-- ─── Jobs ───────────────────────────────────────────────────────────

CREATE TABLE jobs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_pubkey   VARCHAR(44) NOT NULL,
    provider_id     UUID REFERENCES providers(id),
    gpu_class       VARCHAR(16) NOT NULL,
    gpu_count       SMALLINT NOT NULL CHECK (gpu_count >= 1),
    max_duration_sec INT NOT NULL CHECK (max_duration_sec > 0),
    budget_scu      BIGINT NOT NULL CHECK (budget_scu > 0),
    status          VARCHAR(24) NOT NULL DEFAULT 'pending',
    assigned_at     TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    escrow_tx       VARCHAR(88),
    settlement_tx   VARCHAR(88),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_provider ON jobs(provider_id);
CREATE INDEX idx_jobs_client ON jobs(client_pubkey);

-- ─── Compute Receipts ───────────────────────────────────────────────

CREATE TABLE compute_receipts (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id                  UUID REFERENCES jobs(id) NOT NULL UNIQUE,
    provider_pubkey         VARCHAR(44) NOT NULL,
    gpu_class               VARCHAR(16) NOT NULL,
    gpu_count               SMALLINT NOT NULL,
    execution_duration_sec  INT NOT NULL,
    scu_amount              BIGINT NOT NULL,
    result_hash             VARCHAR(64) NOT NULL,
    provider_signature      VARCHAR(128) NOT NULL,
    verified                BOOLEAN NOT NULL DEFAULT FALSE,
    verifier_pubkey         VARCHAR(44),
    verification_tx         VARCHAR(88),
    onchain_tx              VARCHAR(88),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_receipts_job ON compute_receipts(job_id);
CREATE INDEX idx_receipts_verified ON compute_receipts(verified);

-- ─── Transaction Log ────────────────────────────────────────────────

CREATE TABLE transactions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_hash         VARCHAR(88) NOT NULL,
    tx_type         VARCHAR(32) NOT NULL,
    reference_id    UUID NOT NULL,
    status          VARCHAR(16) NOT NULL DEFAULT 'pending',
    retry_count     INT NOT NULL DEFAULT 0,
    error_message   TEXT,
    confirmed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tx_status ON transactions(status);
CREATE INDEX idx_tx_reference ON transactions(reference_id);
CREATE INDEX idx_tx_hash ON transactions(tx_hash);
