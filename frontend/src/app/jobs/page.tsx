"use client";

import { useEffect, useState } from "react";
import { JobTable } from "../components/JobTable";
import type { JobInfo } from "@/lib/types";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "";

export default function JobsPage() {
  const [jobs, setJobs] = useState<JobInfo[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchJobs = async () => {
      try {
        const res = await fetch(`${API_URL}/api/v1/jobs`);
        if (res.ok) setJobs(await res.json());
      } catch {
        // API offline
      } finally {
        setLoading(false);
      }
    };

    fetchJobs();
    const interval = setInterval(fetchJobs, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div>
      <div className="page-header">
        <h1 className="page-title">Compute Jobs</h1>
        <p className="page-subtitle">
          CFD workload jobs with escrow and settlement tracking
        </p>
      </div>

      {loading ? (
        <div className="loading-container">
          <div className="loading-spinner" />
        </div>
      ) : (
        <JobTable jobs={jobs} />
      )}
    </div>
  );
}
