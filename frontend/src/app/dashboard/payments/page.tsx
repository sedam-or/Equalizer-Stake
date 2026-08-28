"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import { StatusBadge } from "@/components/dashboard";
import Pagination from "@/components/Pagination";

interface Payment {
  id_payment: string;
  id_invoice: string;
  amount: number;
  method: string;
  payment_date: string;
  proof_url?: string;
  unique_code_used?: string;
  is_verified: boolean;
  verified_by?: string;
  verified_at?: string;
  notes?: string;
  // Joined data
  guardian_name?: string;
  student_name?: string;
  billing_period?: string;
  invoice_status?: string;
  invoice_amount?: number;
}

const Bearer = "Bearer";
const rupiah = (n: number) => `Rp ${(n ?? 0).toLocaleString("id-ID")}`;

export default function PaymentsPage() {
  const router = useRouter();
  const [payments, setPayments] = useState<Payment[]>([]);
  const [loading, setLoading] = useState(true);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const [selected, setSelected] = useState<Payment | null>(null);
  const [filter, setFilter] = useState("all");
  const limit = 20;

  const fetchPayments = () => {
    const token = localStorage.getItem("access_token");
    if (!token) { router.push("/login"); return; }
    setLoading(true);
    fetch(`/api/v1/payments?limit=${limit}&offset=${offset}&status=${filter}`, {
      headers: { Authorization: `${Bearer} ${token}` },
    })
      .then(r => r.json())
      .then(d => { setPayments(d.data || []); setTotal(d.total || 0); })
      .catch(() => { setPayments([]); setTotal(0); })
      .finally(() => setLoading(false));
  };

  useEffect(() => { fetchPayments(); }, [offset, filter]);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 className="font-heading text-2xl font-bold text-[#0F172A]">Pembayaran</h1>
          <p className="text-sm text-gray-500">Daftar semua pembayaran yang tercatat dalam sistem</p>
        </div>
        <div className="flex gap-2">
          {["all", "verified", "pending"].map(f => (
            <button key={f} onClick={() => { setFilter(f); setOffset(0); }}
              className={`rounded-lg px-4 py-2 text-sm font-medium transition-colors ${filter === f ? "bg-[#2563EB] text-white" : "border border-gray-300 text-gray-600 hover:bg-gray-50"}`}>
              {f === "all" ? "Semua" : f === "verified" ? "Terverifikasi" : "Pending"}
            </button>
          ))}
        </div>
      </div>

      {/* Info Box */}
      <div className="rounded-xl border border-blue-200 bg-blue-50 p-4">
        <h3 className="text-sm font-semibold text-blue-800">💡 Cara Kerja Pembayaran</h3>
        <p className="mt-1 text-sm text-blue-700">
          1. Admin menerbitkan tagihan (Billing) → 2. Wali murid mentransfer + upload bukti → 3. Admin verifikasi kode unik → 4. Status berubah menjadi "Terverifikasi"
        </p>
      </div>

      {/* Table */}
      {loading ? (
        <div className="space-y-2">{[1, 2, 3, 4, 5].map(i => <div key={i} className="h-12 animate-pulse rounded-lg bg-gray-100" />)}</div>
      ) : payments.length === 0 ? (
        <div className="rounded-xl border border-dashed border-gray-300 bg-gray-50 p-12 text-center">
          <p className="text-gray-500">Belum ada data pembayaran.</p>
        </div>
      ) : (
        <div className="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-100 bg-gray-50">
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Tanggal</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Guardian</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Periode</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Jumlah</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Metode</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Status</th>
                </tr>
              </thead>
              <tbody>
                {payments.map((p, idx) => (
                  <tr key={p.id_payment || idx} onClick={() => setSelected(p)}
                    className="cursor-pointer border-b border-gray-50 transition-colors hover:bg-blue-50/50">
                    <td className="px-4 py-3 text-sm text-gray-900">{p.payment_date?.slice(0, 10) || "-"}</td>
                    <td className="px-4 py-3 text-sm font-medium text-gray-900">{p.guardian_name || "-"}</td>
                    <td className="px-4 py-3 text-sm text-gray-600">{p.billing_period || "-"}</td>
                    <td className="px-4 py-3 text-sm font-semibold text-[#0F172A]">{rupiah(p.amount)}</td>
                    <td className="px-4 py-3 text-sm text-gray-600 capitalize">{p.method || "-"}</td>
                    <td className="px-4 py-3">
                      <StatusBadge status={p.is_verified ? "active" : "pending"} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
        </div>
      )}

      {/* Detail Popup */}
      {selected && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4" onClick={() => setSelected(null)}>
          <div className="w-full max-w-lg rounded-xl border border-gray-200 bg-white p-6 shadow-xl" onClick={e => e.stopPropagation()}>
            {/* Header */}
            <div className="flex items-center justify-between">
              <h2 className="font-heading text-lg font-bold text-[#0F172A]">Detail Pembayaran</h2>
              <button onClick={() => setSelected(null)} className="text-gray-400 hover:text-gray-600">✕</button>
            </div>

            {/* Status */}
            <div className="mt-4 flex items-center gap-3 rounded-lg bg-gray-50 p-4">
              <div className={`flex h-12 w-12 items-center justify-center rounded-full text-2xl ${selected.is_verified ? "bg-emerald-100" : "bg-amber-100"}`}>
                {selected.is_verified ? "✓" : "⏳"}
              </div>
              <div>
                <div className="font-heading text-lg font-bold text-[#0F172A]">{rupiah(selected.amount)}</div>
                <div className="text-sm text-gray-500">{selected.is_verified ? "Terverifikasi" : "Menunggu Verifikasi"}</div>
              </div>
            </div>

            {/* Details */}
            <div className="mt-6 space-y-3">
              <div className="flex justify-between border-b border-gray-100 pb-3">
                <span className="text-sm text-gray-500">Guardian</span>
                <span className="text-sm font-medium text-[#0F172A]">{selected.guardian_name || "-"}</span>
              </div>
              <div className="flex justify-between border-b border-gray-100 pb-3">
                <span className="text-sm text-gray-500">Periode Tagihan</span>
                <span className="text-sm font-medium text-[#0F172A]">{selected.billing_period || "-"}</span>
              </div>
              <div className="flex justify-between border-b border-gray-100 pb-3">
                <span className="text-sm text-gray-500">Tanggal Bayar</span>
                <span className="text-sm font-medium text-[#0F172A]">{selected.payment_date?.slice(0, 10) || "-"}</span>
              </div>
              <div className="flex justify-between border-b border-gray-100 pb-3">
                <span className="text-sm text-gray-500">Metode</span>
                <span className="text-sm font-medium text-[#0F172A] capitalize">{selected.method || "-"}</span>
              </div>
              <div className="flex justify-between border-b border-gray-100 pb-3">
                <span className="text-sm text-gray-500">Kode Unik</span>
                <span className="font-mono text-sm font-bold text-[#2563EB]">{selected.unique_code_used || "-"}</span>
              </div>
              {selected.proof_url && (
                <div className="flex justify-between border-b border-gray-100 pb-3">
                  <span className="text-sm text-gray-500">Bukti Transfer</span>
                  <a href={selected.proof_url} target="_blank" rel="noopener noreferrer" className="text-sm font-medium text-[#2563EB] hover:underline">Lihat →</a>
                </div>
              )}
              {selected.verified_at && (
                <div className="flex justify-between border-b border-gray-100 pb-3">
                  <span className="text-sm text-gray-500">Diverifikasi</span>
                  <span className="text-sm font-medium text-[#0F172A]">{selected.verified_at?.slice(0, 16)}</span>
                </div>
              )}
              {selected.notes && (
                <div className="flex justify-between pb-3">
                  <span className="text-sm text-gray-500">Catatan</span>
                  <span className="text-sm text-gray-700">{selected.notes}</span>
                </div>
              )}
            </div>

            {/* Close */}
            <div className="mt-6">
              <button onClick={() => setSelected(null)} className="w-full rounded-lg border border-gray-300 py-2.5 text-sm font-semibold text-gray-700 hover:bg-gray-50">
                Tutup
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
