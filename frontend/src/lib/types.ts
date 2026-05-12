// ─── SolGrid API Types ──────────────────────────────────────────────

export interface ProviderInfo {
  id: string;
  pubkey: string;
  name: string;
  gpu_class: string;
  gpu_count: number;
  max_scu_per_epoch: number;
  status: string;
  total_jobs_completed: number;
  total_scu_delivered: number;
  onchain_tx: string | null;
  created_at: string;
  updated_at: string;
}

export interface JobInfo {
  id: string;
  client_pubkey: string;
  provider_id: string | null;
  gpu_class: string;
  gpu_count: number;
  max_duration_sec: number;
  budget_scu: number;
  status: string;
  assigned_at: string | null;
  completed_at: string | null;
  escrow_tx: string | null;
  settlement_tx: string | null;
  created_at: string;
  updated_at: string;
}

export interface ReceiptInfo {
  id: string;
  job_id: string;
  provider_pubkey: string;
  gpu_class: string;
  gpu_count: number;
  execution_duration_sec: number;
  scu_amount: number;
  result_hash: string;
  provider_signature: string;
  verified: boolean;
  verifier_pubkey: string | null;
  verification_tx: string | null;
  onchain_tx: string | null;
  created_at: string;
}

export interface DashboardStats {
  total_providers: number;
  active_providers: number;
  total_jobs: number;
  active_jobs: number;
  settled_jobs: number;
  total_scu_settled: number;
  total_receipts: number;
  verified_receipts: number;
}

export type JobStatus =
  | "pending"
  | "assigned"
  | "computing"
  | "receipt_submitted"
  | "verified"
  | "settled"
  | "failed";

export const STATUS_COLORS: Record<string, string> = {
  active: "#10b981",
  inactive: "#6b7280",
  suspended: "#ef4444",
  pending: "#f59e0b",
  assigned: "#3b82f6",
  computing: "#8b5cf6",
  receipt_submitted: "#a855f7",
  verified: "#06b6d4",
  settled: "#10b981",
  failed: "#ef4444",
};
