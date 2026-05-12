interface TxHashLinkProps {
  hash: string | null;
  label?: string;
}

export function TxHashLink({ hash, label }: TxHashLinkProps) {
  if (!hash) return <span className="mono" style={{ color: "var(--text-muted)" }}>—</span>;

  const truncated = label || `${hash.slice(0, 8)}...${hash.slice(-4)}`;
  const explorerUrl = `https://explorer.solana.com/tx/${hash}?cluster=devnet`;

  return (
    <a
      href={explorerUrl}
      target="_blank"
      rel="noopener noreferrer"
      className="tx-link"
      title={hash}
    >
      {truncated}
      <span className="tx-link-icon">↗</span>
    </a>
  );
}
