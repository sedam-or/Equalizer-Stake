"use client";
/**
 * T-043: My Payments (Guardian)
 * Invoice + payment history
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

interface Invoice {
  id_invoice: string;
  status: string;
  amount_total: number;
  billing_period: string;
  due_date: string;
  payments: Array<{
    id_payment: string;
    amount: number;
    method: string;
    is_verified: boolean;
    payment_date: string;
  }>;
}

const STATUS_COLORS: Record<string, string> = {
  issued: "bg-blue-100 text-blue-700",
  paid: "bg-green-100 text-green-700",
  overdue: "bg-red-100 text-red-700",
  partial: "bg-yellow-100 text-yellow-700",
};

export default function MyPaymentsPage() {
  const router = useRouter();
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    fetch("/api/my/payments", { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : [])
      .then(setInvoices)
      .finally(() => setLoading(false));
  }, [router]);

  const fmtRp = (n: number) => `Rp ${n.toLocaleString("id-ID")}`;

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-[#000000]">Tagihan & Pembayaran</h1>
        <p className="text-sm text-gray-500">Riwayat tagihan dan pembayaran Anda</p>
      </div>

      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-500">Memuat...</div>
        ) : invoices.length === 0 ? (
          <div className="p-8 text-center text-gray-400">Belum ada tagihan</div>
        ) : (
          <div className="divide-y divide-gray-100">
            {invoices.map(inv => (
              <div key={inv.id_invoice} className="px-4 py-4 hover:bg-gray-50">
                <div className="flex items-center justify-between mb-2">
                  <div>
                    <p className="font-medium text-gray-900">Tagihan {inv.billing_period}</p>
                    <p className="text-xs text-gray-500">Jatuh tempo: {new Date(inv.due_date).toLocaleDateString("id-ID")}</p>
                  </div>
                  <div className="text-right">
                    <p className="font-bold text-gray-900">{fmtRp(inv.amount_total)}</p>
                    <span className={`px-2 py-0.5 rounded text-xs font-medium ${STATUS_COLORS[inv.status] || "bg-gray-100"}`}>
                      {inv.status}
                    </span>
                  </div>
                </div>
                {inv.payments.length > 0 && (
                  <div className="mt-2 space-y-1">
                    {inv.payments.map(p => (
                      <div key={p.id_payment} className="flex items-center gap-2 text-xs text-gray-500">
                        <span className={p.is_verified ? "text-green-600" : "text-yellow-600"}>
                          {p.is_verified ? "✓" : "⏳"}
                        </span>
                        <span>{fmtRp(p.amount)}</span>
                        <span className="capitalize">{p.method}</span>
                        <span>{new Date(p.payment_date).toLocaleDateString("id-ID")}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
