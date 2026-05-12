import type { ReceiptInfo } from "@/lib/types";
import { StatusBadge } from "./StatusBadge";
import { TxHashLink } from "./TxHashLink";

interface ReceiptTableProps {
  receipts: ReceiptInfo[];
}

export function ReceiptTable({ receipts }: ReceiptTableProps) {
  if (receipts.length === 0) {
    return (
      <div className="table-container">
        <div className="empty-state">
          <div className="empty-state-icon">📝</div>
          <div className="empty-state-title">No receipts yet</div>
          <div className="empty-state-text">
            Compute receipts will appear after jobs are processed
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
            <th>Provider</th>
            <th>GPU</th>
            <th>Duration</th>
            <th>SCU</th>
            <th>Verified</th>
            <th>Result Hash</th>
            <th>On-Chain TX</th>
          </tr>
        </thead>
        <tbody>
          {receipts.map((r) => (
            <tr key={r.id}>
              <td>
                <span className="mono truncate" title={r.job_id}>
                  {r.job_id.slice(0, 8)}...
                </span>
              </td>
              <td>
                <span className="mono truncate" title={r.provider_pubkey}>
                  {r.provider_pubkey.slice(0, 8)}...
                </span>
              </td>
              <td>
                <span className="gpu-badge">
                  {r.gpu_class} × {r.gpu_count}
                </span>
              </td>
              <td style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: "0.8rem" }}>
                {r.execution_duration_sec}s
              </td>
              <td style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: "0.8rem", color: "var(--accent-emerald)" }}>
                {r.scu_amount.toLocaleString()}
              </td>
              <td>
                <StatusBadge status={r.verified ? "verified" : "pending"} />
              </td>
              <td>
                <span className="mono truncate" title={r.result_hash}>
                  {r.result_hash.slice(0, 12)}...
                </span>
              </td>
              <td>
                <TxHashLink hash={r.onchain_tx} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
