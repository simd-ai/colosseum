import type { ProviderInfo } from "@/lib/types";
import { StatusBadge } from "./StatusBadge";
import { TxHashLink } from "./TxHashLink";

interface ProviderTableProps {
  providers: ProviderInfo[];
}

export function ProviderTable({ providers }: ProviderTableProps) {
  if (providers.length === 0) {
    return (
      <div className="table-container">
        <div className="empty-state">
          <div className="empty-state-icon">🖥️</div>
          <div className="empty-state-title">No providers registered</div>
          <div className="empty-state-text">
            Providers will appear here once the mock agent starts
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
            <th>Name</th>
            <th>Pubkey</th>
            <th>GPU</th>
            <th>Status</th>
            <th>Jobs Done</th>
            <th>SCU Delivered</th>
            <th>On-Chain TX</th>
            <th>Registered</th>
          </tr>
        </thead>
        <tbody>
          {providers.map((p) => (
            <tr key={p.id}>
              <td style={{ color: "var(--text-primary)", fontWeight: 500 }}>
                {p.name}
              </td>
              <td>
                <span className="mono truncate" title={p.pubkey}>
                  {p.pubkey.slice(0, 8)}...{p.pubkey.slice(-4)}
                </span>
              </td>
              <td>
                <span className="gpu-badge">
                  {p.gpu_class} × {p.gpu_count}
                </span>
              </td>
              <td>
                <StatusBadge status={p.status} />
              </td>
              <td>{p.total_jobs_completed}</td>
              <td>{p.total_scu_delivered.toLocaleString()}</td>
              <td>
                <TxHashLink hash={p.onchain_tx} />
              </td>
              <td
                style={{ color: "var(--text-muted)", fontSize: "0.8rem" }}
                suppressHydrationWarning
              >
                {new Date(p.created_at).toLocaleDateString()}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
