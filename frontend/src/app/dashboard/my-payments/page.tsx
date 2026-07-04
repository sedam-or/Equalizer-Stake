"use client";
/**
 * T-065: My Payments (Guardian) — connected to real API
 * GET /api/my/payments
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface Invoice {
  id_invoice: string;
  billing_period?: string;
  period?: string;
  amount_total?: number;
  amount?: number;
  due_date: string;
  status: string;
  unique_code?: string;
  payments?: { id_payment: string; amount: number; method: string; is_verified: boolean; payment_date: string }[];
}

const STATUS_COLORS: Record<string, string> = {
  paid: "bg-[#2563EB]/10 text-[#2563EB]",
  unpaid: "bg-yellow-500/10 text-yellow-500",
  overdue: "bg-red-500/10 text-red-400",
};

const STATUS_LABELS: Record<string, string> = {
  paid: "Lunas",
  unpaid: "Belum Dibayar",
  overdue: "Jatuh Tempo",
};

const fmtRp = (n: number) => `Rp ${n.toLocaleString("id-ID")}`;

const getPeriod = (inv: Invoice) => inv.billing_period || inv.period || "-";
const getAmount = (inv: Invoice) => inv.amount_total ?? inv.amount ?? 0;

export default function MyPaymentsPage() {
  const router = useRouter();
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [loading, setLoading] = useState(true);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    fetch(`/api/my/payments?limit=${limit}&offset=${offset}`, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : Promise.reject())
      .then(res => {
        setInvoices(res.data || (Array.isArray(res) ? res : []));
        setTotal(res.total || 0);
      })
      .catch(() => setInvoices([]))
      .finally(() => setLoading(false));
  }, [router, offset]);

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Tagihan & Pembayaran</h1>
        <p className="text-sm text-gray-500">Riwayat tagihan dan status pembayaran</p>
      </div>

      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-500">Memuat tagihan...</div>
        ) : invoices.length === 0 ? (
          <div className="p-8 text-center text-gray-500">Belum ada tagihan</div>
        ) : (
          <>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-gray-50 border-b border-gray-200">
                <tr>
                  <th className="px-4 py-3 text-left font-medium text-gray-500">No. Invoice</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-500">Periode</th>
                  <th className="px-4 py-3 text-right font-medium text-gray-500">Jumlah</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-500">Jatuh Tempo</th>
                  <th className="px-4 py-3 text-center font-medium text-gray-500">Kode Unik</th>
                  <th className="px-4 py-3 text-center font-medium text-gray-500">Status</th>
                  <th className="px-4 py-3 text-center font-medium text-gray-500">Aksi</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[#1a1a1a]">
                {invoices.map(inv => (
                  <tr key={inv.id_invoice} className="hover:bg-gray-50">
                    <td className="px-4 py-3 font-mono text-gray-900">{inv.id_invoice}</td>
                    <td className="px-4 py-3 text-gray-900">{getPeriod(inv)}</td>
                    <td className="px-4 py-3 text-right font-mono text-gray-900">{fmtRp(getAmount(inv))}</td>
                    <td className="px-4 py-3 text-gray-500">{new Date(inv.due_date).toLocaleDateString("id-ID")}</td>
                    <td className="px-4 py-3 text-center font-mono text-gray-900">{inv.unique_code || "-"}</td>
                    <td className="px-4 py-3 text-center">
                      <span className={`rounded-full px-2 py-1 text-xs font-medium ${STATUS_COLORS[inv.status] || STATUS_COLORS.unpaid}`}>
                        {STATUS_LABELS[inv.status] || inv.status}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-center">
                      {(inv.status === "issued" || inv.status === "unpaid") && (
                        <button onClick={() => alert(`Bayar invoice ${inv.id_invoice}\nKode unik: ${inv.unique_code}\nJumlah: ${fmtRp(getAmount(inv))}\n\nUpload bukti transfer via WhatsApp admin:\n0812-XXXX-XXXX`)} className="rounded bg-[#2563EB] px-3 py-1 text-xs font-semibold text-black hover:bg-[#1d4ed8]">Bayar</button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
          </>
        )}
      </div>
    </div>
  );
}
