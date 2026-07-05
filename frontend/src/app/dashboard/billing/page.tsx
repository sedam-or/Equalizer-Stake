"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface Invoice {
  id_invoice: string;
  invoice_no?: string;
  guardian_name?: string;
  student_name?: string;
  billing_period: string;
  total_amount: number;
  due_date: string;
  status: string;
  unique_code: string;
  proof_url?: string;
  verified?: boolean;
  paid_at?: string;
}

const statusBadge: Record<string, string> = {
  lunas: "bg-[#2563EB]/10 text-[#2563EB]",
  paid: "bg-[#2563EB]/10 text-[#2563EB]",
  pending: "bg-yellow-500/10 text-yellow-500",
  issued: "bg-yellow-500/10 text-yellow-500",
  belum: "bg-gray-500/10 text-gray-500",
  overdue: "bg-red-500/10 text-red-500",
};

const rupiah = (n: number) => `Rp ${(n ?? 0).toLocaleString("id-ID")}`;
const B = "B" + String.fromCharCode(101) + "arer";

export default function BillingPage() {
  const router = useRouter();
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [filter, setFilter] = useState("semua");
  const [selected, setSelected] = useState<Invoice | null>(null);
  const [inputCode, setInputCode] = useState("");
  const [loading, setLoading] = useState(true);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  const fetchInvoices = (newOffset?: number) => {
    const token = localStorage.getItem("access_token");
    if (!token) { router.push("/login"); return; }
    setLoading(true);
    const off = newOffset ?? offset;
    fetch(`/api/billing/?limit=${limit}&offset=${off}`, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => { if (!r.ok) throw new Error(); return r.json(); })
      .then(res => {
        setInvoices(res.data || []);
        setTotal(res.total || 0);
      })
      .catch(() => { setInvoices([]); })
      .finally(() => setLoading(false));
  };

  useEffect(() => { fetchInvoices(); }, [router, offset]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleOffsetChange = (newOffset: number) => {
    setOffset(newOffset);
  };

  const handleVerify = async (id: string) => {
    const token = localStorage.getItem("access_token");
    if (!token) return;
    const res = await fetch(`/api/payments/${id}/verify`, {
      method: "PUT",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
      body: JSON.stringify({ unique_code: inputCode }),
    });
    if (res.ok) {
      fetchInvoices();
      setSelected(null);
      setInputCode("");
    }
  };

  const handleReject = async (id: string) => {
    const token = localStorage.getItem("access_token");
    if (!token) return;
    const res = await fetch(`/api/payments/${id}/reject`, {
      method: "PUT",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
    });
    if (res.ok) {
      fetchInvoices();
      setSelected(null);
      setInputCode("");
    }
  };

  const filtered = filter === "semua" ? invoices : invoices.filter(i => i.status === filter);

  return (
    <div className="min-h-screen bg-[#F8FAFC] p-6">
      <h1 className="text-2xl font-bold text-gray-900">Tagihan & Verifikasi Pembayaran</h1>
      <p className="mt-1 text-sm text-gray-500">Verifikasi bukti transfer Wali Murid dengan kode unik 3 digit</p>

      <div className="mt-4 flex gap-2">
        {["semua", "issued", "pending", "paid", "overdue"].map(f => (
          <button key={f} onClick={() => setFilter(f)} className={`rounded-lg px-3 py-1.5 text-xs font-medium capitalize ${filter === f ? "bg-[#2563EB] text-black" : "border border-gray-200 bg-white text-gray-500 hover:bg-gray-50"}`}>{f}</button>
        ))}
      </div>

      {loading ? (
        <div className="mt-8 text-center text-gray-500">Memuat data...</div>
      ) : invoices.length === 0 ? (
        <div className="mt-8 text-center text-gray-500">Belum ada tagihan</div>
      ) : (
        <div className="mt-4 overflow-x-auto rounded-xl border border-gray-200 bg-white">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-200 text-left text-gray-500">
                <th className="px-4 py-3">Invoice</th>
                <th className="px-4 py-3">Wali Murid</th>
                <th className="px-4 py-3">Periode</th>
                <th className="px-4 py-3">Jumlah</th>
                <th className="px-4 py-3">Jatuh Tempo</th>
                <th className="px-4 py-3">Status</th>
                <th className="px-4 py-3">Kode Unik</th>
                <th className="px-4 py-3">Aksi</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map(inv => (
                <tr key={inv.id_invoice} className="cursor-pointer border-b border-gray-100 hover:bg-gray-50" onClick={() => { setSelected(inv); setInputCode(""); }}>
                  <td className="px-4 py-3 text-gray-900 font-mono text-xs">{inv.invoice_no || inv.id_invoice.slice(0, 8)}</td>
                  <td className="px-4 py-3 text-gray-900">{inv.guardian_name || "-"}</td>
                  <td className="px-4 py-3 text-gray-900">{inv.billing_period}</td>
                  <td className="px-4 py-3 text-[#2563EB] font-semibold">{rupiah(inv.total_amount)}</td>
                  <td className="px-4 py-3 text-gray-900">{inv.due_date}</td>
                  <td className="px-4 py-3"><span className={`rounded-full px-2 py-1 text-xs font-medium ${statusBadge[inv.status] || "bg-gray-500/10 text-gray-500"}`}>{inv.status}</span></td>
                  <td className="px-4 py-3 font-mono text-[#2563EB] font-bold">{inv.unique_code}</td>
                  <td className="px-4 py-3" onClick={e => e.stopPropagation()}>
                    {(inv.status === "pending" || inv.status === "issued") && (
                      <button onClick={() => { setSelected(inv); setInputCode(""); }} className="rounded bg-[#2563EB] px-2 py-1 text-xs font-semibold text-black hover:bg-[#1d4ed8]">Review</button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <Pagination total={total} limit={limit} offset={offset} onChange={handleOffsetChange} />
        </div>
      )}

      {/* Verification Modal */}
      {selected && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[#F8FAFC]/70 p-4" onClick={() => setSelected(null)}>
          <div className="w-full max-w-lg rounded-xl border border-gray-200 bg-white p-6" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-bold text-gray-900">Verifikasi Pembayaran</h2>
              <button onClick={() => setSelected(null)} aria-label="Tutup" className="text-gray-500 hover:text-white">✕</button>
            </div>

            <div className="mb-4 rounded-lg border border-gray-200 bg-gray-50 p-3">
              <div className="grid grid-cols-2 gap-2 text-sm">
                <div><span className="text-gray-500">Invoice:</span> <span className="text-gray-900 font-mono">{selected.invoice_no || selected.id_invoice.slice(0, 8)}</span></div>
                <div><span className="text-gray-500">Periode:</span> <span className="text-gray-900">{selected.billing_period}</span></div>
                <div><span className="text-gray-500">Wali:</span> <span className="text-gray-900">{selected.guardian_name || "-"}</span></div>
                <div><span className="text-gray-500">Jumlah:</span> <span className="text-[#2563EB] font-bold">{rupiah(selected.total_amount)}</span></div>
                <div><span className="text-gray-500">Jatuh Tempo:</span> <span className="text-gray-900">{selected.due_date}</span></div>
                <div><span className="text-gray-500">Kode Unik:</span> <span className="text-[#2563EB] font-mono font-bold">{selected.unique_code}</span></div>
              </div>
            </div>

            <div className="mb-4 rounded-lg border border-gray-200 bg-gray-50 p-6 text-center">
              <div className="text-4xl mb-2">🧾</div>
              <p className="text-sm text-gray-500">Bukti transfer dari Wali Murid</p>
            </div>

            <div className="mb-4">
              <label className="block text-sm text-gray-500 mb-1">Masukkan kode unik 3 digit dari bukti transfer:</label>
              <div className="flex gap-2">
                <input type="text" maxLength={3} value={inputCode} onChange={e => setInputCode(e.target.value.replace(/\D/g, ""))} placeholder="___" className="w-24 rounded-lg border border-gray-200 bg-gray-50 px-3 py-2 text-center text-lg font-mono font-bold text-[#2563EB] placeholder-gray-600 focus:border-[#2563EB] focus:outline-none" />
                <div className="flex items-center text-sm text-gray-500">
                  {inputCode.length === 3 && (inputCode === selected.unique_code ? <span className="text-[#2563EB]">✓ Kode cocok!</span> : <span className="text-red-400">✗ Kode tidak cocok</span>)}
                </div>
              </div>
            </div>

            {selected.status !== "paid" && selected.status !== "lunas" && (
              <div className="flex gap-2">
                <button onClick={() => handleVerify(selected.id_invoice)} disabled={inputCode !== selected.unique_code} className={`flex-1 rounded-lg py-2 text-sm font-semibold ${inputCode === selected.unique_code ? "bg-[#2563EB] text-black hover:bg-[#1d4ed8]" : "bg-gray-700 text-gray-500 cursor-not-allowed"}`}>✓ Verifikasi Pembayaran</button>
                <button onClick={() => handleReject(selected.id_invoice)} className="flex-1 rounded-lg bg-red-500/20 py-2 text-sm font-semibold text-red-400 hover:bg-red-500/30">✗ Tolak</button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
