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
  paid: "bg-[#08CB00]/10 text-[#08CB00]",
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
        <h1 className="text-2xl font-bold text-[#EEEEEE]">Tagihan & Pembayaran</h1>
        <p className="text-sm text-gray-400">Riwayat tagihan dan status pembayaran</p>
      </div>

      <div className="bg-[#111] rounded-xl border border-[#253900] overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-400">Memuat tagihan...</div>
        ) : invoices.length === 0 ? (
          <div className="p-8 text-center text-gray-400">Belum ada tagihan</div>
        ) : (
          <>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-[#0a0a0a] border-b border-[#253900]">
                <tr>
                  <th className="px-4 py-3 text-left font-medium text-gray-400">No. Invoice</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-400">Periode</th>
                  <th className="px-4 py-3 text-right font-medium text-gray-400">Jumlah</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-400">Jatuh Tempo</th>
                  <th className="px-4 py-3 text-center font-medium text-gray-400">Kode Unik</th>
                  <th className="px-4 py-3 text-center font-medium text-gray-400">Status</th>
                  <th className="px-4 py-3 text-center font-medium text-gray-400">Aksi</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[#1a1a1a]">
                {invoices.map(inv => (
                  <tr key={inv.id_invoice} className="hover:bg-[#1a1a1a]">
                    <td className="px-4 py-3 font-mono text-[#EEEEEE]">{inv.id_invoice}</td>
                    <td className="px-4 py-3 text-[#EEEEEE]">{getPeriod(inv)}</td>
                    <td className="px-4 py-3 text-right font-mono text-[#EEEEEE]">{fmtRp(getAmount(inv))}</td>
                    <td className="px-4 py-3 text-gray-400">{new Date(inv.due_date).toLocaleDateString("id-ID")}</td>
                    <td className="px-4 py-3 text-center font-mono text-[#EEEEEE]">{inv.unique_code || "-"}</td>
                    <td className="px-4 py-3 text-center">
                      <span className={`rounded-full px-2 py-1 text-xs font-medium ${STATUS_COLORS[inv.status] || STATUS_COLORS.unpaid}`}>
                        {STATUS_LABELS[inv.status] || inv.status}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-center">
                      {(inv.status === "issued" || inv.status === "unpaid") && (
                        <button onClick={() => alert(`Bayar invoice ${inv.id_invoice}\nKode unik: ${inv.unique_code}\nJumlah: ${fmtRp(getAmount(inv))}\n\nUpload bukti transfer via WhatsApp admin:\n0812-XXXX-XXXX`)} className="rounded bg-[#08CB00] px-3 py-1 text-xs font-semibold text-black hover:bg-[#06a800]">Bayar</button>
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
