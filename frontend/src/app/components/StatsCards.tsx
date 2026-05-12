import type { DashboardStats } from "@/lib/types";

interface StatsCardsProps {
  stats: DashboardStats;
}

const cards = [
  { key: "total_providers" as const, label: "Total Providers", icon: "🖥️" },
  { key: "active_providers" as const, label: "Active Providers", icon: "✅" },
  { key: "total_jobs" as const, label: "Total Jobs", icon: "⚡" },
  { key: "active_jobs" as const, label: "Active Jobs", icon: "🔄" },
  { key: "settled_jobs" as const, label: "Settled Jobs", icon: "💰" },
  { key: "total_scu_settled" as const, label: "SCU Settled", icon: "📐" },
  { key: "total_receipts" as const, label: "Total Receipts", icon: "📝" },
  { key: "verified_receipts" as const, label: "Verified Receipts", icon: "✔️" },
];

export function StatsCards({ stats }: StatsCardsProps) {
  return (
    <div className="stats-grid">
      {cards.map((card) => (
        <div key={card.key} className="stat-card">
          <div className="stat-card-icon">{card.icon}</div>
          <div className="stat-card-label">{card.label}</div>
          <div className="stat-card-value">
            {(stats[card.key] ?? 0).toLocaleString()}
          </div>
        </div>
      ))}
    </div>
  );
}
