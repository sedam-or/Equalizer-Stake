"use client";
import { useDashboard } from "@/components/DashboardContext";
import { parseApiError } from "@/lib/api-error";
/**
 * T-042: Holiday Management (Admin)
 * UC-05: Kelola Libur & Minggu ke-5
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface Holiday {
  date: string;
  name: string;
  type: string;
}

export default function HolidaysPage() {
  const router = useRouter();
  const [holidays, setHolidays] = useState<Holiday[]>([]);
  const [loading, setLoading] = useState(true);
  const { refreshKey, triggerRefresh } = useDashboard();
  const [form, setForm] = useState({ date: "", name: "" });
  const [err, setErr] = useState("");
  const [msg, setMsg] = useState("");
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  useEffect(() => {
    const token = localStorage.getItem("access_token");
    if (!token) { router.push("/login"); return; }
    fetch(`/api/holidays/?limit=${limit}&offset=${offset}`, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : { data: [], total: 0 })
      .then(res => {
        const data = res.data || res.holidays || (Array.isArray(res) ? res : []);
        setHolidays(data);
        setTotal(res.total || 0);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [router, offset]);

  const handleAdd = async () => {
    setErr(""); setMsg("");
    if (!form.date || !form.name.trim()) { setErr("Tanggal dan nama libur wajib diisi"); return; }
    const token = localStorage.getItem("access_token");
    const updated = [...holidays, { date: form.date, name: form.name, type: "Custom" }];
    const res = await fetch("/api/v1/holidays/", {
      method: "PUT",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
      body: JSON.stringify({ holidays: updated }),
    });
    if (res.ok) {
      setHolidays(updated);
      setMsg("Libur berhasil ditambahkan");
      setForm({ date: "", name: "" });
    } else {
      const data = await res.json();
      setErr(parseApiError(data));
    }
  };

  const handleRemove = async (idx: number) => {
    const updated = holidays.filter((_, i) => i !== idx);
    const token = localStorage.getItem("access_token");
    await fetch("/api/v1/holidays/", {
      method: "PUT",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
      body: JSON.stringify({ holidays: updated }),
    });
    setHolidays(updated);
  };

  return (
    <div className="max-w-3xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Libur Nasional</h1>
        <p className="text-sm text-gray-500">Kelola hari libur untuk penjadwalan</p>
      </div>

      {msg && <div className="p-3 bg-[#2563EB]/10 border border-gray-200 rounded-lg text-[#2563EB] text-sm">{msg}</div>}
      {err && <div className="p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 text-sm">{err}</div>}

      {/* Add Form */}
      <div className="bg-white rounded-xl p-4 border border-gray-200 flex gap-3 items-end">
        <div className="flex-1">
          <label className="block text-xs text-gray-500 mb-1">Tanggal</label>
          <input type="date" value={form.date} onChange={e => setForm({ ...form, date: e.target.value })}
            className="w-full px-3 py-2 bg-[#F8FAFC] border border-gray-200 rounded-lg text-sm text-gray-900 focus:border-[#2563EB] focus:outline-none" />
        </div>
        <div className="flex-1">
          <label className="block text-xs text-gray-500 mb-1">Nama Libur</label>
          <input placeholder="Contoh: Idul Fitri" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })}
            className="w-full px-3 py-2 bg-[#F8FAFC] border border-gray-200 rounded-lg text-sm text-gray-900 focus:border-[#2563EB] focus:outline-none" />
        </div>
        <button onClick={handleAdd}
          className="px-4 py-2 bg-[#2563EB] text-black text-sm font-semibold rounded-lg hover:bg-[#1d4ed8]">
          Tambah
        </button>
      </div>

      {/* Holiday List */}
      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-500">Memuat...</div>
        ) : holidays.length === 0 ? (
          <div className="p-8 text-center text-gray-500">Belum ada hari libur</div>
        ) : (
          <>
          <div className="divide-y divide-[#1a1a1a]">
            {holidays.map((h, idx) => (
              <div key={idx} className="flex items-center justify-between px-4 py-3 hover:bg-gray-50">
                <div className="flex items-center gap-4">
                  <span className="rounded-full px-2 py-1 text-xs font-medium bg-[#2563EB]/10 text-[#2563EB]">{h.type || "Nasional"}</span>
                  <div>
                    <span className="text-sm font-medium text-gray-900">{h.name}</span>
                    <span className="text-xs text-gray-500 ml-3">{new Date(h.date).toLocaleDateString("id-ID", { weekday: "long", day: "numeric", month: "long", year: "numeric" })}</span>
                  </div>
                </div>
                <button onClick={() => handleRemove(idx)} className="text-xs text-red-400 hover:underline">Hapus</button>
              </div>
            ))}
          </div>
          <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
          </>
        )}
      </div>
    </div>
  );
}
