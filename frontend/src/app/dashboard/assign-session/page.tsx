"use client";
import { useDashboard } from "@/components/DashboardContext";
import { parseApiError } from "@/lib/api-error";

import { useEffect, useState, useCallback } from "react";

interface Student { id_student: string; full_name: string; instrument: string; }
interface Teacher { id_teacher: string; full_name: string; instruments: string[]; }
interface Room { id_room: string; room_name: string; room_type: string; }

export default function AssignSessionPage() {
  const [students, setStudents] = useState<Student[]>([]);
  const [teachers, setTeachers] = useState<Teacher[]>([]);
  const [rooms, setRooms] = useState<Room[]>([]);
  const [loading, setLoading] = useState(true);
  const { refreshKey, triggerRefresh } = useDashboard();
  const [form, setForm] = useState({ id_student: "", id_teacher: "", id_room: "", session_date: "", start_time: "", end_time: "" });
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const token = typeof window !== "undefined" ? localStorage.getItem("access_token") : null;

  const fetchData = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const [sRes, tRes, rRes] = await Promise.all([
        fetch("/api/v1/students/", { headers: { Authorization: `Bearer ${token}` } }),
        fetch("/api/v1/sessions/teachers", { headers: { Authorization: `Bearer ${token}` } }),
        fetch("/api/v1/sessions/rooms", { headers: { Authorization: `Bearer ${token}` } }),
      ]);
      if (sRes.ok) { const d = await sRes.json(); setStudents(d.data || []); }
      if (tRes.ok) setTeachers(await tRes.json());
      if (rRes.ok) setRooms(await rRes.json());
    } catch { /* ignore */ }
    setLoading(false);
  }, [token]);

  useEffect(() => { fetchData(); }, [fetchData]);

  const handleAssign = async () => {
    setError(""); setSuccess("");
    if (!form.id_student || !form.id_teacher || !form.id_room || !form.session_date || !form.start_time || !form.end_time) {
      setError("Semua field wajib diisi"); return;
    }
    try {
      const res = await fetch("/api/v1/sessions/", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify(form),
      });
      if (!res.ok) {
        const data = await res.json();
        setError(parseApiError(data));
        return;
      }
      setSuccess("Sesi berhasil diassign!");
      setForm({ id_student: "", id_teacher: "", id_room: "", session_date: "", start_time: "", end_time: "" });
    } catch { setError("Terjadi kesalahan"); }
  };

  return (
    <div className="p-6">
      <h1 className="font-heading text-2xl font-bold text-gray-900">Assign Sesi</h1>
      <p className="text-sm text-gray-500">Assign siswa ke sesi kursus (guru + ruangan + jadwal)</p>

      {loading ? <div className="mt-6 p-8 text-center text-gray-500">Memuat...</div> : (
        <div className="mt-6 max-w-xl rounded-xl border border-gray-200 bg-white p-6">
          {error && <div className="mb-4 rounded-lg bg-red-50 p-3 text-sm text-red-600">{error}</div>}
          {success && <div className="mb-4 rounded-lg bg-green-50 p-3 text-sm text-green-700">{success}</div>}

          <div className="space-y-4">
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Siswa *</label>
              <select value={form.id_student} onChange={(e) => setForm({ ...form, id_student: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm">
                <option value="">Pilih siswa</option>
                {students.map((s) => <option key={s.id_student} value={s.id_student}>{s.full_name} ({s.instrument})</option>)}
              </select>
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Guru *</label>
              <select value={form.id_teacher} onChange={(e) => setForm({ ...form, id_teacher: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm">
                <option value="">Pilih guru</option>
                {teachers.map((t) => <option key={t.id_teacher} value={t.id_teacher}>{t.full_name} ({t.instruments.join(", ")})</option>)}
              </select>
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Ruangan *</label>
              <select value={form.id_room} onChange={(e) => setForm({ ...form, id_room: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm">
                <option value="">Pilih ruangan</option>
                {rooms.map((r) => <option key={r.id_room} value={r.id_room}>{r.room_name} ({r.room_type})</option>)}
              </select>
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Tanggal *</label>
              <input type="date" value={form.session_date} onChange={(e) => setForm({ ...form, session_date: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" />
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Jam Mulai *</label>
                <input type="time" value={form.start_time} onChange={(e) => setForm({ ...form, start_time: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" />
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Jam Selesai *</label>
                <input type="time" value={form.end_time} onChange={(e) => setForm({ ...form, end_time: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" />
              </div>
            </div>
          </div>

          <button onClick={handleAssign} className="mt-6 w-full rounded-lg bg-[#2563EB] py-2.5 text-sm font-semibold text-white hover:bg-[#1D4ED8]">Assign Sesi</button>
        </div>
      )}
    </div>
  );
}
