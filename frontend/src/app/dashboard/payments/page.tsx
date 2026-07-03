"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface Payment {
  id: number;
  date: string;
  invoice: string;
  student: string;
  guardian: string;
  amount: number;
  method: string;
  status: "berhasil" | "pending" | "gagal";
}

const badge: Record<string, string> = {
  berhasil: "bg-[#2563EB]/10 text-[#2563EB]",
  pending: "bg-yellow-500/10 text-yellow-500",
  gagal: "bg-red-500/10 text-red-500",
};

const badgeLabel: Record<string, string> = {
  berhasil: "Berhasil",
  pending: "Pending",
  gagal: "Gagal",
};

const rupiah = (n: number) => `Rp ${n.toLocaleString("id-ID")}`;

export default function PaymentsPage() {
  const router = useRouter();
  const [payments, setPayments] = useState<Payment[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState("semua");
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    fetch(`/api/payments/?limit=${limit}&offset=${offset}`, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : { data: [], total: 0 })
      .then(res => {
        setPayments(res.data || []);
        setTotal(res.total || 0);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [router, offset]);

  const filtered = filter === "semua" ? payments : payments.filter((p) => p.status === filter);

  return (
    <div className="min-h-screen bg-black p-6">
      <h1 className="text-2xl font-bold text-[#EEEEEE]">Pembayaran</h1>
      <p className="mt-1 text-sm text-gray-400">Riwayat pembayaran siswa ({filtered.length} data)</p>

      <div className="mt-4 flex gap-2">
        {["semua", "berhasil", "pending", "gagal"].map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`rounded-lg px-3 py-1.5 text-xs font-medium capitalize ${
              filter === f ? "bg-[#2563EB] text-black" : "border border-[#CBD5E1] bg-[#111] text-gray-400 hover:bg-[#1a1a1a]"
            }`}
          >
            {f}
          </button>
        ))}
      </div>

      <div className="mt-4 overflow-x-auto rounded-xl border border-[#CBD5E1] bg-[#111]">
        {loading ? (
          <div className="p-8 text-center text-gray-400">Memuat data pembayaran...</div>
        ) : (
          <>
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-[#CBD5E1] text-left text-gray-400">
                  <th className="px-4 py-3">No</th>
                  <th className="px-4 py-3">Tanggal</th>
                  <th className="px-4 py-3">Invoice</th>
                  <th className="px-4 py-3">Siswa</th>
                  <th className="px-4 py-3">Wali</th>
                  <th className="px-4 py-3">Jumlah</th>
                  <th className="px-4 py-3">Metode</th>
                  <th className="px-4 py-3">Status</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((p, idx) => (
                  <tr key={p.id} className="border-b border-[#1a1a1a] hover:bg-[#1a1a1a]">
                    <td className="px-4 py-3 text-gray-400">{offset + idx + 1}</td>
                    <td className="px-4 py-3 text-[#EEEEEE]">{p.date}</td>
                    <td className="px-4 py-3 text-[#EEEEEE]">{p.invoice}</td>
                    <td className="px-4 py-3 text-[#EEEEEE]">{p.student}</td>
                    <td className="px-4 py-3 text-gray-400">{p.guardian}</td>
                    <td className="px-4 py-3 text-[#EEEEEE]">{rupiah(p.amount)}</td>
                    <td className="px-4 py-3 text-[#EEEEEE]">{p.method}</td>
                    <td className="px-4 py-3">
                      <span className={`rounded-full px-2 py-1 text-xs font-medium ${badge[p.status]}`}>
                        {badgeLabel[p.status]}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
          </>
        )}
      </div>
    </div>
  );
}
