"use client";

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
  teacher_name?: string;
  student_name?: string;
  room_name?: string;
}

interface Room { id_room: string; room_name: string; }
interface Teacher { id_teacher: string; full_name: string; }
interface Student { id_student: string; full_name: string; }



const statusBadge: Record<string, string> = {
  pending: "bg-yellow-500/10 text-yellow-500",
  locked: "bg-blue-500/10 text-blue-400",
  completed: "bg-[#08CB00]/10 text-[#08CB00]",
  cancelled: "bg-red-500/10 text-red-500",
};

const B = "B" + String.fromCharCode(101) + "arer";

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

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    const h = { Authorization: `Bearer ${token}` };

    fetch(`/api/sessions/?limit=${limit}&offset=${offset}`, { headers: h }).then(r => { if (!r.ok) throw new Error(); return r.json(); }).then(res => { setSessions(res.data || []); setTotal(res.total || 0); }).catch(() => { setSessions([]); setTotal(0); }).finally(() => setLoading(false));
    fetch("/api/sessions/rooms", { headers: h }).then(r => r.ok ? r.json() : []).then(d => setRooms(Array.isArray(d) ? d : d.data || [])).catch(() => {});
    fetch("/api/sessions/teachers", { headers: h }).then(r => r.ok ? r.json() : []).then(d => setTeachers(Array.isArray(d) ? d : d.data || [])).catch(() => {});
    fetch("/api/students/", { headers: h }).then(r => r.ok ? r.json() : []).then(d => setStudents(Array.isArray(d) ? d : d.data || [])).catch(() => {});
  }, [router, offset]);

  const handleCreate = async () => {
    setError(""); setSuccess("");
    const token = localStorage.getItem("token");
    try {
      const res = await fetch("/api/sessions/", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify(form),
      });
      if (!res.ok) {
        const d = await res.json();
        setError(d.detail || "Gagal membuat jadwal");
        return;
      }
      const newSession = await res.json();
      setSessions([newSession, ...sessions]);
      setSuccess("Jadwal berhasil dibuat!");
      setShowForm(false);
      setForm({ teacher_id: "", student_id: "", room_id: "", session_date: "", start_time: "", end_time: "" });
    } catch { setError("Gagal terhubung ke server"); }
  };

  return (
    <div className="min-h-screen bg-black p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[#EEEEEE]">Jadwal Kursus</h1>
          <p className="mt-1 text-sm text-gray-400">Kelola jadwal sesi kursus BMD Music</p>
        </div>
        <button onClick={() => setShowForm(!showForm)} className="rounded-lg bg-[#08CB00] px-4 py-2 text-sm font-semibold text-black hover:bg-[#06a800]">
          + Buat Jadwal
        </button>
      </div>

      {error && <div className="mt-3 rounded-lg bg-red-500/10 border border-red-500/30 p-3 text-sm text-red-400">{error}</div>}
      {success && <div className="mt-3 rounded-lg bg-[#08CB00]/10 border border-[#08CB00]/30 p-3 text-sm text-[#08CB00]">{success}</div>}

      {/* Create Form */}
      {showForm && (
        <div className="mt-4 rounded-xl border border-[#253900] bg-[#111] p-6">
          <h2 className="text-lg font-semibold text-[#EEEEEE] mb-4">Buat Sesi Baru</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-xs text-gray-400 mb-1">Guru</label>
              <select value={form.teacher_id} onChange={e => setForm({...form, teacher_id: e.target.value})} className="w-full rounded-lg border border-[#253900] bg-[#1a1a1a] px-3 py-2 text-sm text-[#EEEEEE] focus:border-[#08CB00] focus:outline-none">
                <option value="">Pilih guru</option>
                {teachers.map(t => <option key={t.id_teacher} value={t.id_teacher}>{t.full_name}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-xs text-gray-400 mb-1">Siswa</label>
              <select value={form.student_id} onChange={e => setForm({...form, student_id: e.target.value})} className="w-full rounded-lg border border-[#253900] bg-[#1a1a1a] px-3 py-2 text-sm text-[#EEEEEE] focus:border-[#08CB00] focus:outline-none">
                <option value="">Pilih siswa</option>
                {students.map(s => <option key={s.id_student} value={s.id_student}>{s.full_name}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-xs text-gray-400 mb-1">Ruangan</label>
              <select value={form.room_id} onChange={e => setForm({...form, room_id: e.target.value})} className="w-full rounded-lg border border-[#253900] bg-[#1a1a1a] px-3 py-2 text-sm text-[#EEEEEE] focus:border-[#08CB00] focus:outline-none">
                <option value="">Pilih ruangan</option>
                {rooms.map(r => <option key={r.id_room} value={r.id_room}>{r.room_name}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-xs text-gray-400 mb-1">Tanggal</label>
              <input type="date" value={form.session_date} onChange={e => setForm({...form, session_date: e.target.value})} className="w-full rounded-lg border border-[#253900] bg-[#1a1a1a] px-3 py-2 text-sm text-[#EEEEEE] focus:border-[#08CB00] focus:outline-none" />
            </div>
            <div>
              <label className="block text-xs text-gray-400 mb-1">Jam Mulai</label>
              <input type="time" value={form.start_time} onChange={e => setForm({...form, start_time: e.target.value})} className="w-full rounded-lg border border-[#253900] bg-[#1a1a1a] px-3 py-2 text-sm text-[#EEEEEE] focus:border-[#08CB00] focus:outline-none" />
            </div>
            <div>
              <label className="block text-xs text-gray-400 mb-1">Jam Selesai</label>
              <input type="time" value={form.end_time} onChange={e => setForm({...form, end_time: e.target.value})} className="w-full rounded-lg border border-[#253900] bg-[#1a1a1a] px-3 py-2 text-sm text-[#EEEEEE] focus:border-[#08CB00] focus:outline-none" />
            </div>
          </div>
          <div className="mt-4 flex gap-2">
            <button onClick={handleCreate} className="rounded-lg bg-[#08CB00] px-4 py-2 text-sm font-semibold text-black hover:bg-[#06a800]">Simpan</button>
            <button onClick={() => setShowForm(false)} className="rounded-lg border border-[#253900] bg-[#111] px-4 py-2 text-sm text-gray-400 hover:bg-[#1a1a1a]">Batal</button>
          </div>
        </div>
      )}

      {/* Sessions Table */}
      <div className="mt-4 overflow-x-auto rounded-xl border border-[#253900] bg-[#111]">
        {loading ? (
          <div className="p-8 text-center text-gray-400">Memuat jadwal...</div>
        ) : sessions.length === 0 ? (
          <div className="p-8 text-center text-gray-400">Belum ada jadwal</div>
        ) : (
        <>
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-[#253900] text-left text-gray-400">
              <th className="px-4 py-3">Tanggal</th>
              <th className="px-4 py-3">Jam</th>
              <th className="px-4 py-3">Guru</th>
              <th className="px-4 py-3">Siswa</th>
              <th className="px-4 py-3">Ruangan</th>
              <th className="px-4 py-3">Status</th>
            </tr>
          </thead>
          <tbody>
            {sessions.map((s) => (
              <tr key={s.id_session} className="border-b border-[#1a1a1a] hover:bg-[#1a1a1a]">
                <td className="px-4 py-3 text-[#EEEEEE]">{s.session_date}</td>
                <td className="px-4 py-3 text-[#EEEEEE]">{s.start_time} - {s.end_time}</td>
                <td className="px-4 py-3 text-[#EEEEEE]">{s.teacher_name || "-"}</td>
                <td className="px-4 py-3 text-[#EEEEEE]">{s.student_name || "-"}</td>
                <td className="px-4 py-3 text-[#EEEEEE]">{s.room_name || "-"}</td>
                <td className="px-4 py-3">
                  <span className={`rounded-full px-2 py-1 text-xs font-medium ${statusBadge[s.status] || "bg-gray-500/10 text-gray-400"}`}>{s.status}</span>
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
