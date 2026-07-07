"use client";

import { useEffect, useState, useRef } from "react";
import { useRouter } from "next/navigation";
import { toast } from "@/components/Toast";
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
  paid: "bg-emerald-100 text-emerald-700",
  issued: "bg-amber-100 text-amber-700",
  overdue: "bg-red-100 text-red-700",
  pending: "bg-amber-100 text-amber-700",
};

const STATUS_LABELS: Record<string, string> = {
  paid: "Lunas",
  issued: "Belum Dibayar",
  overdue: "Jatuh Tempo",
  pending: "Menunggu Verifikasi",
};

const fmtRp = (n: number) => `Rp ${(n ?? 0).toLocaleString("id-ID")}`;
const getPeriod = (inv: Invoice) => inv.billing_period || inv.period || "-";
const getAmount = (inv: Invoice) => inv.amount_total ?? inv.amount ?? 0;

export default function MyPaymentsPage() {
  const router = useRouter();
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [loading, setLoading] = useState(true);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const [selected, setSelected] = useState<Invoice | null>(null);
  const [proofFile, setProofFile] = useState<File | null>(null);
  const [proofPreview, setProofPreview] = useState<string | null>(null);
  const [method, setMethod] = useState("transfer");
  const [submitting, setSubmitting] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);
  const limit = 20;

  const fetchInvoices = () => {
    const token = localStorage.getItem("access_token");
    if (!token) { router.push("/login"); return; }
    fetch(`/api/my/payments?limit=${limit}&offset=${offset}`, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : Promise.reject())
      .then(res => { setInvoices(res.data || (Array.isArray(res) ? res : [])); setTotal(res.total || 0); })
      .catch(() => { setInvoices([]); toast("error", "Gagal memuat data pembayaran"); })
      .finally(() => setLoading(false));
  };

  useEffect(() => { fetchInvoices(); }, [offset]);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    if (file.size > 5 * 1024 * 1024) { toast("warning", "Ukuran file maksimal 5MB"); return; }
    if (!file.type.startsWith("image/")) { toast("warning", "File harus berupa gambar"); return; }
    setProofFile(file);
    const reader = new FileReader();
    reader.onload = (ev) => setProofPreview(ev.target?.result as string);
    reader.readAsDataURL(file);
  };

  const handleSubmitPayment = async () => {
    if (!selected || !proofFile) return;
    setSubmitting(true);
    try {
      const token = localStorage.getItem("access_token");
      const base64 = proofPreview || "";
      const res = await fetch("/api/v1/payments", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify({
          id_invoice: selected.id_invoice,
          amount: getAmount(selected),
          method,
          proof_base64: base64,
        }),
      });
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        throw new Error(data.detail || "Gagal mengirim pembayaran");
      }
      toast("success", "Pembayaran berhasil dikirim! Menunggu verifikasi admin.");
      setSelected(null);
      setProofFile(null);
      setProofPreview(null);
      fetchInvoices();
    } catch (err) {
      toast("error", err instanceof Error ? err.message : "Terjadi kesalahan");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="font-heading text-2xl font-bold text-[#0F172A]">Pembayaran</h1>
        <p className="text-sm text-gray-500">Lihat tagihan dan upload bukti pembayaran</p>
      </div>

      {/* Info box */}
      <div className="rounded-xl border border-blue-200 bg-blue-50 p-4">
        <h3 className="text-sm font-semibold text-blue-800">💡 Cara Membayar</h3>
        <ol className="mt-1 space-y-1 text-sm text-blue-700">
          <li>1. Transfer ke rekening <strong>BCA 1234567890</strong> (a.n. BMD Music School)</li>
          <li>2. Jumlah transfer = Tagihan + Kode Unik (contoh: Rp 500.847)</li>
          <li>3. Klik "Bayar" → upload bukti transfer → Submit</li>
          <li>4. Admin akan memverifikasi dalam 1×24 jam</li>
        </ol>
      </div>

      {loading ? (
        <div className="space-y-2">{[1, 2, 3].map(i => <div key={i} className="h-16 animate-pulse rounded-lg bg-gray-100" />)}</div>
      ) : invoices.length === 0 ? (
        <div className="rounded-xl border border-dashed border-gray-300 bg-gray-50 p-12 text-center">
          <p className="text-gray-500">Belum ada tagihan.</p>
        </div>
      ) : (
        <div className="space-y-3">
          {invoices.map((inv, idx) => (
            <div key={inv.id_invoice || idx} className="rounded-xl border border-gray-200 bg-white p-4 shadow-sm">
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-heading text-sm font-bold text-[#0F172A]">Tagihan {getPeriod(inv)}</div>
                  <div className="text-xs text-gray-500">Jatuh tempo: {inv.due_date}</div>
                </div>
                <div className="text-right">
                  <div className="font-heading text-lg font-bold text-[#0F172A]">{fmtRp(getAmount(inv))}</div>
                  <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${STATUS_COLORS[inv.status] || "bg-gray-100 text-gray-700"}`}>
                    {STATUS_LABELS[inv.status] || inv.status}
                  </span>
                </div>
              </div>

              {/* Payment history */}
              {inv.payments && inv.payments.length > 0 && (
                <div className="mt-3 border-t border-gray-100 pt-3">
                  {inv.payments.map((p, pi) => (
                    <div key={pi} className="flex items-center justify-between text-xs">
                      <span className="text-gray-500">{p.payment_date?.slice(0, 10)} — {p.method}</span>
                      <span className={p.is_verified ? "text-emerald-600 font-medium" : "text-amber-600"}>
                        {p.is_verified ? "✓ Terverifikasi" : "⏳ Menunggu"}
                      </span>
                    </div>
                  ))}
                </div>
              )}

              {/* Action */}
              {(inv.status === "issued" || inv.status === "pending") && (
                <div className="mt-3 border-t border-gray-100 pt-3">
                  {inv.unique_code && (
                    <p className="mb-2 text-xs text-gray-500">
                      Kode unik: <span className="font-mono font-bold text-[#2563EB]">{inv.unique_code}</span> — transfer Rp {fmtRp(getAmount(inv) + parseInt(inv.unique_code || "0"))}
                    </p>
                  )}
                  <button onClick={() => setSelected(inv)} className="rounded-lg bg-[#2563EB] px-4 py-2 text-sm font-semibold text-white hover:bg-[#1D4ED8]">
                    💳 Bayar Sekarang
                  </button>
                </div>
              )}
            </div>
          ))}
          <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
        </div>
      )}

      {/* Payment Modal */}
      {selected && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4" onClick={() => setSelected(null)}>
          <div className="w-full max-w-md rounded-xl border border-gray-200 bg-white p-6 shadow-xl" onClick={e => e.stopPropagation()}>
            <h2 className="font-heading text-lg font-bold text-[#0F172A]">Upload Bukti Pembayaran</h2>

            {/* Invoice summary */}
            <div className="mt-4 rounded-lg bg-gray-50 p-4">
              <div className="flex justify-between text-sm">
                <span className="text-gray-500">Tagihan</span>
                <span className="font-medium text-[#0F172A]">{getPeriod(selected)}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-500">Jumlah</span>
                <span className="font-bold text-[#0F172A]">{fmtRp(getAmount(selected))}</span>
              </div>
              {selected.unique_code && (
                <div className="flex justify-between text-sm">
                  <span className="text-gray-500">Transfer (dengan kode unik)</span>
                  <span className="font-bold text-[#2563EB]">{fmtRp(getAmount(selected) + parseInt(selected.unique_code || "0"))}</span>
                </div>
              )}
              <div className="flex justify-between text-sm">
                <span className="text-gray-500">Kode Unik</span>
                <span className="font-mono font-bold text-[#2563EB]">{selected.unique_code || "-"}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-500">Jatuh Tempo</span>
                <span className="text-[#0F172A]">{selected.due_date}</span>
              </div>
            </div>

            {/* Payment method */}
            <div className="mt-4">
              <label className="mb-1 block text-sm font-medium text-gray-700">Metode Pembayaran</label>
              <select value={method} onChange={e => setMethod(e.target.value)} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm">
                <option value="transfer">Transfer Bank</option>
                <option value="cash">Tunai</option>
                <option value="ewallet">E-Wallet</option>
              </select>
            </div>

            {/* Upload proof */}
            <div className="mt-4">
              <label className="mb-1 block text-sm font-medium text-gray-700">Bukti Transfer *</label>
              <input ref={fileRef} type="file" accept="image/*" onChange={handleFileChange} className="hidden" />
              <button onClick={() => fileRef.current?.click()} className="w-full rounded-lg border-2 border-dashed border-gray-300 p-4 text-center transition-colors hover:border-[#2563EB] hover:bg-blue-50/50">
                {proofPreview ? (
                  <img src={proofPreview} alt="Preview" className="mx-auto h-32 rounded-lg object-contain" />
                ) : (
                  <>
                    <span className="text-2xl">📷</span>
                    <p className="mt-1 text-sm text-gray-500">Klik untuk upload bukti transfer</p>
                    <p className="text-xs text-gray-400">JPG/PNG, maks 5MB</p>
                  </>
                )}
              </button>
            </div>

            {/* Submit */}
            <div className="mt-6 flex gap-2">
              <button onClick={handleSubmitPayment} disabled={!proofFile || submitting}
                className={`flex-1 rounded-lg py-2.5 text-sm font-semibold ${proofFile && !submitting ? "bg-[#2563EB] text-white hover:bg-[#1D4ED8]" : "bg-gray-200 text-gray-400 cursor-not-allowed"}`}>
                {submitting ? "Mengirim..." : "✓ Kirim Pembayaran"}
              </button>
              <button onClick={() => { setSelected(null); setProofFile(null); setProofPreview(null); }} className="rounded-lg border border-gray-300 px-4 py-2.5 text-sm font-semibold text-gray-700 hover:bg-gray-50">
                Batal
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
