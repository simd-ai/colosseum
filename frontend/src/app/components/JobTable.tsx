import type { JobInfo } from "@/lib/types";
import { StatusBadge } from "./StatusBadge";
import { TxHashLink } from "./TxHashLink";

interface JobTableProps {
  jobs: JobInfo[];
}

export function JobTable({ jobs }: JobTableProps) {
  if (jobs.length === 0) {
    return (
      <div className="table-container">
        <div className="empty-state">
          <div className="empty-state-icon">⚡</div>
          <div className="empty-state-title">No jobs yet</div>
          <div className="empty-state-text">
            Jobs will appear here once created via the API
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="table-container">
      <table className="data-table">
        <thead>
          <tr>
            <th>Job ID</th>
            <th>Client</th>
            <th>GPU</th>
            <th>Budget SCU</th>
            <th>Status</th>
            <th>Escrow TX</th>
            <th>Settlement TX</th>
            <th>Created</th>
          </tr>
        </thead>
        <tbody>
          {jobs.map((job) => (
            <tr key={job.id}>
              <td>
                <span className="mono truncate" title={job.id}>
                  {job.id.slice(0, 8)}...
                </span>
              </td>
              <td>
                <span className="mono truncate" title={job.client_pubkey}>
                  {job.client_pubkey.slice(0, 8)}...
                </span>
              </td>
              <td>
                <span className="gpu-badge">
                  {job.gpu_class} × {job.gpu_count}
                </span>
              </td>
              <td style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: "0.8rem" }}>
                {job.budget_scu.toLocaleString()}
              </td>
              <td>
                <StatusBadge status={job.status} />
              </td>
              <td>
                <TxHashLink hash={job.escrow_tx} />
              </td>
              <td>
                <TxHashLink hash={job.settlement_tx} />
              </td>
              <td
                style={{ color: "var(--text-muted)", fontSize: "0.8rem" }}
                suppressHydrationWarning
              >
                {new Date(job.created_at).toLocaleString()}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
