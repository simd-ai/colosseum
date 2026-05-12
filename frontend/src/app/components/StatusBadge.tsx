interface StatusBadgeProps {
  status: string;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  const normalized = status.toLowerCase().replace(/\s+/g, "_");
  return (
    <span className={`status-badge status-${normalized}`}>
      <span className="status-badge-dot" />
      {status.replace(/_/g, " ")}
    </span>
  );
}
