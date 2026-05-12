import type { ProviderInfo, JobInfo, ReceiptInfo, DashboardStats } from "./types";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "";

async function fetchApi<T>(path: string): Promise<T> {
  const res = await fetch(`${API_URL}${path}`, {
    cache: "no-store",
    headers: { "Content-Type": "application/json" },
  });
  if (!res.ok) {
    throw new Error(`API error: ${res.status} ${res.statusText}`);
  }
  return res.json();
}

export async function getStats(): Promise<DashboardStats> {
  return fetchApi<DashboardStats>("/api/v1/stats");
}

export async function getProviders(): Promise<ProviderInfo[]> {
  return fetchApi<ProviderInfo[]>("/api/v1/providers");
}

export async function getJobs(): Promise<JobInfo[]> {
  return fetchApi<JobInfo[]>("/api/v1/jobs");
}

export async function getReceipts(): Promise<ReceiptInfo[]> {
  return fetchApi<ReceiptInfo[]>("/api/v1/receipts");
}

export async function createJob(data: {
  client_pubkey: string;
  gpu_class: string;
  gpu_count: number;
  max_duration_sec: number;
  budget_scu: number;
}): Promise<JobInfo> {
  const res = await fetch(`${API_URL}/api/v1/jobs`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
  if (!res.ok) throw new Error(`Failed to create job: ${res.statusText}`);
  return res.json();
}

export async function registerProvider(data: {
  pubkey: string;
  name: string;
  gpu_class: string;
  gpu_count: number;
  max_scu_per_epoch: number;
}): Promise<ProviderInfo> {
  const res = await fetch(`${API_URL}/api/v1/providers/register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
  if (!res.ok) throw new Error(`Failed to register: ${res.statusText}`);
  return res.json();
}
