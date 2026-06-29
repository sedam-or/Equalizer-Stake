"use client";
/**
 * T-042: Students Management (Admin)
 * UC-06: Reservasi Studio (students are linked to guardians)
 * - List all students with filters
 * - Create new student
 * - Edit student status
 * - Soft delete
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

interface Student {
  id_student: string;
  id_guardian: string;
  full_name: string;
  instrument: string;
  level: string;
  enrollment_status: string;
  enrollment_date: string;
  eligible_for_exam: boolean;
}

const INSTRUMENTS = ["biola", "piano", "gitar", "drum", "keyboard", "flute", "saxophone", "vokal", "ensemble"];
const LEVELS = ["pemula", "dasar", "menengah", "lanjut"];
const STATUS_COLORS: Record<string, string> = {
  active: "bg-green-100 text-green-700",
  trial: "bg-blue-100 text-blue-700",
  cuti: "bg-yellow-100 text-yellow-700",
  inactive: "bg-gray-100 text-gray-500",
};

export default function StudentsPage() {
  const router = useRouter();
  const [students, setStudents] = useState<Student[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [filter, setFilter] = useState("");
  const [form, setForm] = useState({ full_name: "", instrument: "piano", level: "pemula", enrollment_date: new Date().toISOString().slice(0, 10), id_guardian: "" });
  const [err, setErr] = useState("");
  const [msg, setMsg] = useState("");

  const fetchStudents = async () => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    const params = new URLSearchParams({ limit: "200" });
    if (filter) params.set("status", filter);
    const res = await fetch(`/api/students?${params}`, { headers: { Authorization: `Bearer ${token}` } });
    if (res.ok) { const d = await res.json(); setStudents(Array.isArray(d) ? d : []); }
    setLoading(false);
  };

  useEffect(() => { fetchStudents(); }, [filter]); // eslint-disable-line

  const handleCreate = async () => {
    setErr(""); setMsg("");
    if (!form.full_name.trim() || !form.id_guardian.trim()) { setErr("Nama dan ID Guardian wajib diisi"); return; }
    const token = localStorage.getItem("token");
    const res = await fetch("/api/students", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
      body: JSON.stringify(form),
    });
    const data = await res.json();
    if (!res.ok) { setErr(data.detail || "Gagal menambahkan siswa"); return; }
    setMsg("Siswa berhasil ditambahkan");
    setShowCreate(false);
    setForm({ full_name: "", instrument: "piano", level: "pemula", enrollment_date: new Date().toISOString().slice(0, 10), id_guardian: "" });
    fetchStudents();
  };

  const handleStatusChange = async (id: string, status: string) => {
    const token = localStorage.getItem("token");
    await fetch(`/api/students/${id}`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
      body: JSON.stringify({ enrollment_status: status }),
    });
    fetchStudents();
  };

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[#000000]">Manajemen Siswa</h1>
          <p className="text-sm text-gray-500">{students.length} siswa terdaftar</p>
        </div>
        <button onClick={() => setShowCreate(!showCreate)}
          className="px-4 py-2 bg-[#08CB00] text-black text-sm font-medium rounded-lg hover:bg-[#06a800]">
          {showCreate ? "Batal" : "+ Tambah Siswa"}
        </button>
      </div>

      {msg && <div className="p-3 bg-green-50 border border-green-200 rounded-lg text-green-700 text-sm">{msg}</div>}
      {err && <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-600 text-sm">{err}</div>}

      {/* Create Form */}
      {showCreate && (
        <div className="bg-white rounded-xl p-6 border border-gray-200 space-y-3">
          <h3 className="font-semibold">Tambah Siswa Baru</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <input placeholder="Nama Lengkap" value={form.full_name} onChange={e => setForm({ ...form, full_name: e.target.value })}
              className="px-3 py-2 border border-gray-300 rounded-lg text-sm" />
            <input placeholder="ID Guardian (UUID)" value={form.id_guardian} onChange={e => setForm({ ...form, id_guardian: e.target.value })}
              className="px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono text-xs" />
            <select value={form.instrument} onChange={e => setForm({ ...form, instrument: e.target.value })}
              className="px-3 py-2 border border-gray-300 rounded-lg text-sm">
              {INSTRUMENTS.map(i => <option key={i} value={i}>{i}</option>)}
            </select>
            <select value={form.level} onChange={e => setForm({ ...form, level: e.target.value })}
              className="px-3 py-2 border border-gray-300 rounded-lg text-sm">
              {LEVELS.map(l => <option key={l} value={l}>{l}</option>)}
            </select>
            <input type="date" value={form.enrollment_date} onChange={e => setForm({ ...form, enrollment_date: e.target.value })}
              className="px-3 py-2 border border-gray-300 rounded-lg text-sm" />
          </div>
          <button onClick={handleCreate} className="px-4 py-2 bg-[#000000] text-white text-sm rounded-lg hover:bg-gray-800">Simpan</button>
        </div>
      )}

      {/* Filter */}
      <div className="flex gap-2">
        {["", "active", "trial", "cuti", "inactive"].map(s => (
          <button key={s} onClick={() => setFilter(s)}
            className={`px-3 py-1 rounded-lg text-xs font-medium ${filter === s ? "bg-[#08CB00] text-black" : "bg-gray-100 text-gray-600 hover:bg-gray-200"}`}>
            {s || "Semua"}
          </button>
        ))}
      </div>

      {/* Student List */}
      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-500">Memuat...</div>
        ) : students.length === 0 ? (
          <div className="p-8 text-center text-gray-400">Tidak ada siswa</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-gray-50 border-b border-gray-200">
                <tr>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Nama</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Instrument</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Level</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Status</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Ujian</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Aksi</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-100">
                {students.map(s => (
                  <tr key={s.id_student} className="hover:bg-gray-50">
                    <td className="px-4 py-2.5 font-medium text-gray-900">{s.full_name}</td>
                    <td className="px-4 py-2.5 text-gray-600 capitalize">{s.instrument}</td>
                    <td className="px-4 py-2.5 text-gray-600 capitalize">{s.level}</td>
                    <td className="px-4 py-2.5">
                      <span className={`px-2 py-0.5 rounded text-xs font-medium ${STATUS_COLORS[s.enrollment_status] || "bg-gray-100"}`}>
                        {s.enrollment_status}
                      </span>
                    </td>
                    <td className="px-4 py-2.5">{s.eligible_for_exam ? "✅" : "—"}</td>
                    <td className="px-4 py-2.5">
                      <select value={s.enrollment_status} onChange={e => handleStatusChange(s.id_student, e.target.value)}
                        className="text-xs border border-gray-300 rounded px-2 py-1">
                        <option value="active">Active</option>
                        <option value="trial">Trial</option>
                        <option value="cuti">Cuti</option>
                        <option value="inactive">Inactive</option>
                      </select>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
