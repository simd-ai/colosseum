"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const navItems = [
  { href: "/", label: "Overview", icon: "📊" },
  { href: "/providers", label: "Providers", icon: "🖥️" },
  { href: "/jobs", label: "Jobs", icon: "⚡" },
  { href: "/receipts", label: "Receipts", icon: "📝" },
];

export function Navbar() {
  const pathname = usePathname();

  return (
    <nav className="navbar">
      <Link href="/" className="navbar-brand">
        <div className="navbar-logo">SG</div>
        <div>
          <div className="navbar-title">SolGrid</div>
          <div className="navbar-subtitle">GPU Compute Settlement</div>
        </div>
      </Link>

      <ul className="navbar-links">
        {navItems.map((item) => (
          <li key={item.href}>
            <Link
              href={item.href}
              className={`navbar-link ${pathname === item.href ? "active" : ""}`}
            >
              <span>{item.icon}</span>
              {item.label}
            </Link>
          </li>
        ))}
      </ul>

      <div className="navbar-network">
        <span className="navbar-network-dot" />
        Solana Devnet
      </div>
    </nav>
  );
}
