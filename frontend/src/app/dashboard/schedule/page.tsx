"use client";

import { useDashboard } from "@/components/DashboardContext";
import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface Session {
  id_session: string;
  session_date: string;
  start_time: string;
  end_time: string;
  session_type: string;
  status: string;
  id_teacher?: string;
  id_student?: string;
  id_room?: string;
}

interface Room { id_room: string; room_name: string; }
interface Teacher { id_teacher: string; full_name: string; }
interface Student { id_student: string; full_name: string; }

const statusBadge: Record<string, string> = {
  pending: "bg-yellow-100 text-yellow-700",
  locked: "bg-blue-100 text-blue-700",
  completed: "bg-emerald-100 text-emerald-700",
  cancelled: "bg-red-100 text-red-700",
  rescheduled: "bg-purple-100 text-purple-700",
};

const Bearer = "Bearer";

export default function SchedulePage() {
  const router = useRouter();
  const [sessions, setSessions] = useState<Session[]>([]);
  const [rooms, setRooms] = useState<Room[]>([]);
  const [teachers, setTeachers] = useState<Teacher[]>([]);
  const [students, setStudents] = useState<Student[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState({ teacher_id: "", student_id: "", room_id: "", session_date: "", start_time: "", end_time: "" });
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const [loading, setLoading] = useState(true);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;
  const { refreshKey, triggerRefresh } = useDashboard();

  // Lookup maps
  const teacherMap = Object.fromEntries(teachers.map(t => [t.id_teacher, t.full_name]));
  const studentMap = Object.fromEntries(students.map(s => [s.id_student, s.full_name]));
  const roomMap = Object.fromEntries(rooms.map(r => [r.id_room, r.room_name]));

  const fetchAll = () => {
    const token = localStorage.getItem("access_token");
    if (!token) { router.push("/login"); return; }
    const h = { Authorization: `${Bearer} ${token}` };

    setLoading(true);
    fetch(`/api/v1/sessions/?limit=${limit}&offset=${offset}`, { headers: h })
      .then(r => { if (!r.ok) throw new Error(); return r.json(); })
      .then(data => { setSessions(Array.isArray(data) ? data : data.data || []); setTotal(data.total || (Array.isArray(data) ? data.length : 0)); })
      .catch(() => { setSessions([]); setTotal(0); })
      .finally(() => setLoading(false));

    fetch("/api/v1/sessions/rooms", { headers: h }).then(r => r.ok ? r.json() : []).then(d => setRooms(Array.isArray(d) ? d : d.data || [])).catch(() => {});
    fetch("/api/v1/sessions/teachers", { headers: h }).then(r => r.ok ? r.json() : []).then(d => setTeachers(Array.isArray(d) ? d : d.data || [])).catch(() => {});
    fetch("/api/v1/students", { headers: h }).then(r => r.ok ? r.json() : []).then(d => setStudents(Array.isArray(d) ? d : d.data || [])).catch(() => {});
  };

  useEffect(() => { fetchAll(); }, [offset, refreshKey]);

  const handleCreate = async () => {
    setError(""); setSuccess("");
    const token = localStorage.getItem("access_token");
    try {
      const res = await fetch("/api/v1/sessions/", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `${Bearer} ${token}` },
        body: JSON.stringify({
          id_teacher: form.teacher_id,
          id_student: form.student_id,
          id_room: form.room_id,
          session_date: form.session_date,
          start_time: form.start_time,
          end_time: form.end_time,
          session_type: "reguler",
        }),
      });
      if (!res.ok) { const d = await res.json().catch(() => ({})); throw new Error(d.detail || "Gagal membuat jadwal"); }
      setSuccess("Jadwal berhasil dibuat");
      setShowForm(false);
      setForm({ teacher_id: "", student_id: "", room_id: "", session_date: "", start_time: "", end_time: "" });
      triggerRefresh();
    } catch (err) { setError(err instanceof Error ? err.message : "Terjadi kesalahan"); }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="font-heading text-2xl font-bold text-[#0F172A]">Jadwal Kursus</h1>
          <p className="text-sm text-gray-500">Kelola jadwal sesi kursus</p>
        </div>
        <button onClick={() => setShowForm(!showForm)} className="rounded-lg bg-[#2563EB] px-4 py-2 text-sm font-semibold text-white hover:bg-[#1D4ED8]">
          {showForm ? "Tutup" : "+ Buat Jadwal"}
        </button>
      </div>

      {error && <div className="rounded-lg bg-red-50 p-3 text-sm text-red-600">{error}</div>}
      {success && <div className="rounded-lg bg-emerald-50 p-3 text-sm text-emerald-600">{success}</div>}

      {/* Form */}
      {showForm && (
        <div className="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
          <h2 className="mb-4 font-heading text-lg font-bold text-[#0F172A]">Jadwal Baru</h2>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Guru</label>
              <select value={form.teacher_id} onChange={e => setForm({ ...form, teacher_id: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm">
                <option value="">Pilih guru</option>
                {teachers.map(t => <option key={t.id_teacher} value={t.id_teacher}>{t.full_name}</option>)}
              </select>
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Siswa</label>
              <select value={form.student_id} onChange={e => setForm({ ...form, student_id: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm">
                <option value="">Pilih siswa</option>
                {students.map(s => <option key={s.id_student} value={s.id_student}>{s.full_name}</option>)}
              </select>
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Studio</label>
              <select value={form.room_id} onChange={e => setForm({ ...form, room_id: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm">
                <option value="">Pilih studio</option>
                {rooms.map(r => <option key={r.id_room} value={r.id_room}>{r.room_name}</option>)}
              </select>
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Tanggal</label>
              <input type="date" value={form.session_date} onChange={e => setForm({ ...form, session_date: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Jam Mulai</label>
              <input type="time" value={form.start_time} onChange={e => setForm({ ...form, start_time: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Jam Selesai</label>
              <input type="time" value={form.end_time} onChange={e => setForm({ ...form, end_time: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" />
            </div>
          </div>
          <div className="mt-4 flex gap-2">
            <button onClick={handleCreate} className="rounded-lg bg-[#2563EB] px-5 py-2 text-sm font-semibold text-white hover:bg-[#1D4ED8]">Simpan</button>
            <button onClick={() => setShowForm(false)} className="rounded-lg border border-gray-300 px-5 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50">Batal</button>
          </div>
        </div>
      )}

      {/* Table */}
      {loading ? (
        <div className="space-y-2">{[1, 2, 3, 4, 5].map(i => <div key={i} className="h-12 animate-pulse rounded-lg bg-gray-100" />)}</div>
      ) : sessions.length === 0 ? (
        <div className="rounded-xl border border-dashed border-gray-300 bg-gray-50 p-12 text-center">
          <p className="text-gray-500">Belum ada jadwal. Klik "Buat Jadwal" untuk memulai.</p>
        </div>
      ) : (
        <div className="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-100 bg-gray-50">
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Tanggal</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Waktu</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Guru</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Siswa</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Studio</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Status</th>
                </tr>
              </thead>
              <tbody>
                {sessions.map(s => (
                  <tr key={s.id_session} className="border-b border-gray-50 transition-colors hover:bg-gray-50">
                    <td className="px-4 py-3 text-sm text-gray-900">{s.session_date}</td>
                    <td className="px-4 py-3 text-sm text-gray-900">{String(s.start_time).slice(0, 5)} - {String(s.end_time).slice(0, 5)}</td>
                    <td className="px-4 py-3 text-sm text-gray-900">{teacherMap[s.id_teacher || ""] || s.id_teacher?.slice(0, 8) || "-"}</td>
                    <td className="px-4 py-3 text-sm text-gray-900">{studentMap[s.id_student || ""] || s.id_student?.slice(0, 8) || "-"}</td>
                    <td className="px-4 py-3 text-sm text-gray-900">{roomMap[s.id_room || ""] || s.id_room?.slice(0, 8) || "-"}</td>
                    <td className="px-4 py-3">
                      <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${statusBadge[s.status] || "bg-gray-100 text-gray-700"}`}>
                        {s.status}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
        </div>
      )}
    </div>
  );
}
