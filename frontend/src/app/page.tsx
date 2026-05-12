"use client";

import { useEffect, useState } from "react";
import { StatsCards } from "./components/StatsCards";
import { JobTable } from "./components/JobTable";
import { ProviderTable } from "./components/ProviderTable";
import type { DashboardStats, JobInfo, ProviderInfo } from "@/lib/types";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "";

// Demo fallback data shown when the API is offline
const DEMO_STATS: DashboardStats = {
  total_providers: 3,
  active_providers: 2,
  total_jobs: 12,
  active_jobs: 2,
  settled_jobs: 8,
  total_scu_settled: 284_500,
  total_receipts: 10,
  verified_receipts: 9,
};

const DEMO_PROVIDERS: ProviderInfo[] = [
  {
    id: "a1b2c3d4-0000-0000-0000-000000000001",
    pubkey: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
    name: "MockProvider-Alpha",
    gpu_class: "A100",
    gpu_count: 4,
    max_scu_per_epoch: 1000000,
    status: "active",
    total_jobs_completed: 5,
    total_scu_delivered: 142250,
    onchain_tx: "4hXdFyqNfVJfQ2vKjTPYMRaLNz7bBfYxVdwMqGH5xGRn",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: "a1b2c3d4-0000-0000-0000-000000000002",
    pubkey: "9mNzFkR3cUj8PTgbHNhfC4tQdJ5FbeS2Af7mkUL6yrQW",
    name: "MockProvider-Beta",
    gpu_class: "H100",
    gpu_count: 8,
    max_scu_per_epoch: 2000000,
    status: "active",
    total_jobs_completed: 3,
    total_scu_delivered: 98000,
    onchain_tx: "2bRfKLjqPdH8nYwS5ZcXfGvTq3pM7uJxN1eD9kWoAsBm",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
];

const DEMO_JOBS: JobInfo[] = [
  {
    id: "b2c3d4e5-0000-0000-0000-000000000001",
    client_pubkey: "3rTk8WDnkZPF7juYh1QfmsACyGLvSqLPQxLcf4JKeMrR",
    provider_id: "a1b2c3d4-0000-0000-0000-000000000001",
    gpu_class: "A100",
    gpu_count: 4,
    max_duration_sec: 120,
    budget_scu: 50000,
    status: "settled",
    assigned_at: new Date().toISOString(),
    completed_at: new Date().toISOString(),
    escrow_tx: "5cYjKdpRm2wVn8SbGfTqZ3hX7uLxN4eH9kWoAsBmDfgP",
    settlement_tx: "6dZkLeqSn3xWo9TcHgUrA4iY8vMyO5fI0lXpBtCnEghQ",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: "b2c3d4e5-0000-0000-0000-000000000002",
    client_pubkey: "3rTk8WDnkZPF7juYh1QfmsACyGLvSqLPQxLcf4JKeMrR",
    provider_id: null,
    gpu_class: "H100",
    gpu_count: 8,
    max_duration_sec: 300,
    budget_scu: 120000,
    status: "pending",
    assigned_at: null,
    completed_at: null,
    escrow_tx: null,
    settlement_tx: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
];

export default function OverviewPage() {
  const [stats, setStats] = useState<DashboardStats>(DEMO_STATS);
  const [providers, setProviders] = useState<ProviderInfo[]>(DEMO_PROVIDERS);
  const [jobs, setJobs] = useState<JobInfo[]>(DEMO_JOBS);
  const [isLive, setIsLive] = useState(false);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [statsRes, providersRes, jobsRes] = await Promise.all([
          fetch(`${API_URL}/api/v1/stats`),
          fetch(`${API_URL}/api/v1/providers`),
          fetch(`${API_URL}/api/v1/jobs`),
        ]);
        if (statsRes.ok) {
          setStats(await statsRes.json());
          setIsLive(true);
        }
        if (providersRes.ok) setProviders(await providersRes.json());
        if (jobsRes.ok) setJobs(await jobsRes.json());
      } catch {
        // API offline — keep demo data
        setIsLive(false);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div>
      <div className="page-header">
        <h1 className="page-title">
          Dashboard Overview
          {!isLive && (
            <span
              style={{
                fontSize: "0.7rem",
                background: "rgba(245, 158, 11, 0.15)",
                color: "#fbbf24",
                padding: "0.2rem 0.6rem",
                borderRadius: "999px",
                marginLeft: "0.75rem",
                verticalAlign: "middle",
                fontWeight: 600,
              }}
            >
              DEMO DATA
            </span>
          )}
        </h1>
        <p className="page-subtitle">
          Real-time GPU compute settlement metrics on Solana Devnet
        </p>
      </div>

      <StatsCards stats={stats} />

      <div style={{ marginBottom: "2rem" }}>
        <h2 style={{ fontSize: "1.1rem", fontWeight: 600, marginBottom: "1rem", color: "var(--text-primary)" }}>
          Recent Providers
        </h2>
        <ProviderTable providers={providers.slice(0, 5)} />
      </div>

      <div>
        <h2 style={{ fontSize: "1.1rem", fontWeight: 600, marginBottom: "1rem", color: "var(--text-primary)" }}>
          Recent Jobs
        </h2>
        <JobTable jobs={jobs.slice(0, 5)} />
      </div>
    </div>
  );
}
