"use client";

import { useEffect, useState } from "react";
import { ReceiptTable } from "../components/ReceiptTable";
import type { ReceiptInfo } from "@/lib/types";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "";

export default function ReceiptsPage() {
  const [receipts, setReceipts] = useState<ReceiptInfo[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchReceipts = async () => {
      try {
        const res = await fetch(`${API_URL}/api/v1/receipts`);
        if (res.ok) setReceipts(await res.json());
      } catch {
        // API offline
      } finally {
        setLoading(false);
      }
    };

    fetchReceipts();
    const interval = setInterval(fetchReceipts, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div>
      <div className="page-header">
        <h1 className="page-title">Compute Receipts</h1>
        <p className="page-subtitle">
          Verified compute receipts with on-chain proof and payout status
        </p>
      </div>

      {loading ? (
        <div className="loading-container">
          <div className="loading-spinner" />
        </div>
      ) : (
        <ReceiptTable receipts={receipts} />
      )}
    </div>
  );
}
