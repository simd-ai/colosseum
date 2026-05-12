"use client";

import { useEffect, useState } from "react";
import { ProviderTable } from "../components/ProviderTable";
import type { ProviderInfo } from "@/lib/types";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "";

export default function ProvidersPage() {
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchProviders = async () => {
      try {
        const res = await fetch(`${API_URL}/api/v1/providers`);
        if (res.ok) setProviders(await res.json());
      } catch {
        // API offline
      } finally {
        setLoading(false);
      }
    };

    fetchProviders();
    const interval = setInterval(fetchProviders, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div>
      <div className="page-header">
        <h1 className="page-title">GPU Providers</h1>
        <p className="page-subtitle">
          Registered compute providers and their GPU resources
        </p>
      </div>

      {loading ? (
        <div className="loading-container">
          <div className="loading-spinner" />
        </div>
      ) : (
        <ProviderTable providers={providers} />
      )}
    </div>
  );
}
