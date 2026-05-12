import type { NextConfig } from "next";

const SCHEDULER_ORIGIN =
  process.env.SCHEDULER_ORIGIN || "http://127.0.0.1:8080";

// Next 16 blocks dev-only assets (HMR WebSocket, etc.) for any origin other
// than the one `next dev` was started on. When accessing the dashboard via a
// public IP / remote hostname, list it here (comma-separated) or the HMR
// client fails to init and the page never hydrates, leaving DEMO_STATS on
// screen. Example:  ALLOWED_DEV_ORIGINS=136.116.105.159,my.dev.host
const allowedDevOrigins = (process.env.ALLOWED_DEV_ORIGINS || "")
  .split(",")
  .map((s) => s.trim())
  .filter(Boolean);

const nextConfig: NextConfig = {
  allowedDevOrigins,
  async rewrites() {
    return [
      {
        source: "/api/:path*",
        destination: `${SCHEDULER_ORIGIN}/api/:path*`,
      },
    ];
  },
};

export default nextConfig;
