"use client";
/**
 * T-042: Holiday Management (Admin)
 * UC-05: Kelola Libur & Minggu ke-5
 * - List holidays from system_settings
 * - Add/remove holidays
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

interface Holiday {
  date: string;
  name: string;
}

export default function HolidaysPage() {
  const router = useRouter();
  const [holidays, setHolidays] = useState<Holiday[]>([]);
  const [loading, setLoading] = useState(true);
  const [form, setForm] = useState({ date: "", name: "" });
  const [err, setErr] = useState("");
  const [msg, setMsg] = useState("");

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    fetch("/api/settings/holidays", { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : [])
      .then(data => {
        if (Array.isArray(data)) setHolidays(data);
        else if (data.holidays) setHolidays(data.holidays);
        else setHolidays([]);
      })
      .catch(() => setHolidays([]))
      .finally(() => setLoading(false));
  }, [router]);

  const handleAdd = async () => {
    setErr(""); setMsg("");
    if (!form.date || !form.name.trim()) { setErr("Tanggal dan nama libur wajib diisi"); return; }
    const token = localStorage.getItem("token");
    const updated = [...holidays, { date: form.date, name: form.name }];
    const res = await fetch("/api/settings/holidays", {
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
      setErr(data.detail || "Gagal menambahkan libur");
    }
  };

  const handleRemove = async (idx: number) => {
    const updated = holidays.filter((_, i) => i !== idx);
    const token = localStorage.getItem("token");
    await fetch("/api/settings/holidays", {
      method: "PUT",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
      body: JSON.stringify({ holidays: updated }),
    });
    setHolidays(updated);
  };

  return (
    <div className="max-w-3xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-[#000000]">Libur Nasional</h1>
        <p className="text-sm text-gray-500">Kelola hari libur untuk penjadwalan (UC-05)</p>
      </div>

      {msg && <div className="p-3 bg-green-50 border border-green-200 rounded-lg text-green-700 text-sm">{msg}</div>}
      {err && <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-600 text-sm">{err}</div>}

      {/* Add Form */}
      <div className="bg-white rounded-xl p-4 border border-gray-200 flex gap-3 items-end">
        <div className="flex-1">
          <label className="block text-xs text-gray-500 mb-1">Tanggal</label>
          <input type="date" value={form.date} onChange={e => setForm({ ...form, date: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm" />
        </div>
        <div className="flex-1">
          <label className="block text-xs text-gray-500 mb-1">Nama Libur</label>
          <input placeholder="Contoh: Idul Fitri" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm" />
        </div>
        <button onClick={handleAdd}
          className="px-4 py-2 bg-[#08CB00] text-black text-sm font-medium rounded-lg hover:bg-[#06a800]">
          Tambah
        </button>
      </div>

      {/* Holiday List */}
      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-500">Memuat...</div>
        ) : holidays.length === 0 ? (
          <div className="p-8 text-center text-gray-400">Belum ada hari libur</div>
        ) : (
          <div className="divide-y divide-gray-100">
            {holidays.map((h, idx) => (
              <div key={idx} className="flex items-center justify-between px-4 py-3 hover:bg-gray-50">
                <div>
                  <span className="text-sm font-medium text-gray-900">{h.name}</span>
                  <span className="text-xs text-gray-500 ml-3">{new Date(h.date).toLocaleDateString("id-ID", { weekday: "long", day: "numeric", month: "long", year: "numeric" })}</span>
                </div>
                <button onClick={() => handleRemove(idx)} className="text-xs text-red-500 hover:underline">Hapus</button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
