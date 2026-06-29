"use client";
/**
 * T-042: Payment Verification (Admin)
 * UC-08: Verifikasi Pembayaran Manual
 * - List payments with filters
 * - Verify payment with notes
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

interface Payment {
  id_payment: string;
  id_invoice: string;
  amount: number;
  method: string;
  payment_date: string;
  is_verified: boolean;
  verified_at: string | null;
  notes: string | null;
  unique_code_used: string;
}

const METHOD_LABELS: Record<string, string> = { transfer: "Transfer", cash: "Tunai", dp: "DP" };

export default function PaymentsPage() {
  const router = useRouter();
  const [payments, setPayments] = useState<Payment[]>([]);
  const [loading, setLoading] = useState(true);
  const [verifiedFilter, setVerifiedFilter] = useState<string>("");
  const [verifyModal, setVerifyModal] = useState<string | null>(null);
  const [verifyNotes, setVerifyNotes] = useState("");
  const [msg, setMsg] = useState("");

  const fetchPayments = async () => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    const params = new URLSearchParams({ limit: "200" });
    if (verifiedFilter === "verified") params.set("verified", "true");
    if (verifiedFilter === "unverified") params.set("verified", "false");
    const res = await fetch(`/api/payments?${params}`, { headers: { Authorization: `Bearer ${token}` } });
    if (res.ok) { const d = await res.json(); setPayments(Array.isArray(d) ? d : []); }
    setLoading(false);
  };

  useEffect(() => { fetchPayments(); }, [verifiedFilter]); // eslint-disable-line

  const handleVerify = async (id: string) => {
    const token = localStorage.getItem("token");
    const res = await fetch(`/api/payments/${id}/verify`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
      body: JSON.stringify({ notes: verifyNotes || undefined }),
    });
    const data = await res.json();
    if (res.ok) {
      setMsg("Pembayaran berhasil diverifikasi");
      setVerifyModal(null);
      setVerifyNotes("");
      fetchPayments();
    } else {
      setMsg(data.detail || "Gagal verifikasi");
    }
  };

  const fmtRp = (n: number) => `Rp ${n.toLocaleString("id-ID")}`;

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-[#000000]">Verifikasi Pembayaran</h1>
        <p className="text-sm text-gray-500">Kelola verifikasi pembayaran manual (UC-08)</p>
      </div>

      {msg && <div className="p-3 bg-green-50 border border-green-200 rounded-lg text-green-700 text-sm">{msg}</div>}

      {/* Filter */}
      <div className="flex gap-2">
        {["", "unverified", "verified"].map(v => (
          <button key={v} onClick={() => setVerifiedFilter(v)}
            className={`px-3 py-1 rounded-lg text-xs font-medium ${verifiedFilter === v ? "bg-[#08CB00] text-black" : "bg-gray-100 text-gray-600 hover:bg-gray-200"}`}>
            {v === "" ? "Semua" : v === "unverified" ? "Belum Diverifikasi" : "Sudah Diverifikasi"}
          </button>
        ))}
        <span className="text-xs text-gray-400 ml-auto self-center">{payments.length} pembayaran</span>
      </div>

      {/* Payment List */}
      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-500">Memuat...</div>
        ) : payments.length === 0 ? (
          <div className="p-8 text-center text-gray-400">Tidak ada pembayaran</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-gray-50 border-b border-gray-200">
                <tr>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Tanggal</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Jumlah</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Metode</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Kode Unik</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Status</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Aksi</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-100">
                {payments.map(p => (
                  <tr key={p.id_payment} className="hover:bg-gray-50">
                    <td className="px-4 py-2.5 text-gray-700">{new Date(p.payment_date).toLocaleDateString("id-ID")}</td>
                    <td className="px-4 py-2.5 font-medium text-gray-900">{fmtRp(p.amount)}</td>
                    <td className="px-4 py-2.5 text-gray-600">{METHOD_LABELS[p.method] || p.method}</td>
                    <td className="px-4 py-2.5 font-mono text-xs text-gray-500">{p.unique_code_used}</td>
                    <td className="px-4 py-2.5">
                      {p.is_verified ? (
                        <span className="px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-700">Terverifikasi</span>
                      ) : (
                        <span className="px-2 py-0.5 rounded text-xs font-medium bg-yellow-100 text-yellow-700">Menunggu</span>
                      )}
                    </td>
                    <td className="px-4 py-2.5">
                      {!p.is_verified && (
                        <button onClick={() => setVerifyModal(p.id_payment)}
                          className="px-3 py-1 bg-[#08CB00] text-black text-xs font-medium rounded hover:bg-[#06a800]">
                          Verifikasi
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Verify Modal */}
      {verifyModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl p-6 w-full max-w-md space-y-4">
            <h3 className="font-semibold">Verifikasi Pembayaran</h3>
            <textarea placeholder="Catatan (opsional)" value={verifyNotes} onChange={e => setVerifyNotes(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm" rows={3} />
            <div className="flex gap-2">
              <button onClick={() => handleVerify(verifyModal)}
                className="px-4 py-2 bg-[#08CB00] text-black text-sm font-medium rounded-lg hover:bg-[#06a800]">
                Konfirmasi
              </button>
              <button onClick={() => { setVerifyModal(null); setVerifyNotes(""); }}
                className="px-4 py-2 bg-gray-200 text-gray-700 text-sm rounded-lg hover:bg-gray-300">
                Batal
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
