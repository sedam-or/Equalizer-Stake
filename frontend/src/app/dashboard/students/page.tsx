"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { toast } from "@/components/Toast";
import { SkeletonTable } from "@/components/dashboard/Skeleton";
import Pagination from "@/components/Pagination";

interface Student {
  id_student: string;
  id_guardian: string;
  full_name: string;
  instrument: string;
  level: string;
  is_active: boolean;
  guardian_name?: string;
}

const INSTRUMENTS = ["piano", "gitar", "vokal", "biola", "drum", "keyboard", "flute", "saxophone", "bass"];
const INSTRUMENT_LABELS: Record<string, string> = {
  piano: "Piano", gitar: "Gitar", vokal: "Vokal", biola: "Biola",
  drum: "Drum", keyboard: "Keyboard", flute: "Flute", saxophone: "Saxophone", bass: "Bass",
};

export default function StudentsPage() {
  const router = useRouter();
  const [students, setStudents] = useState<Student[]>([]);
  const [loading, setLoading] = useState(true);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const [showForm, setShowForm] = useState(false);
  const [editStudent, setEditStudent] = useState<Student | null>(null);
  const [form, setForm] = useState({ full_name: "", instrument: "piano", level: "pemula" });
  const [submitting, setSubmitting] = useState(false);
  const limit = 20;

  const fetchStudents = () => {
    const token = localStorage.getItem("access_token");
    if (!token) { router.push("/login"); return; }
    setLoading(true);
    fetch(`/api/v1/students?limit=${limit}&offset=${offset}`, {
      headers: { Authorization: `Bearer ${token}` },
    })
      .then(r => r.json())
      .then(d => { setStudents(d.data || []); setTotal(d.total || 0); })
      .catch(() => { setStudents([]); toast("error", "Gagal memuat data student"); })
      .finally(() => setLoading(false));
  };

  useEffect(() => { fetchStudents(); }, [offset]);

  const handleCreate = async () => {
    setSubmitting(true);
    try {
      const token = localStorage.getItem("access_token");
      const res = await fetch("/api/v1/students", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify(form),
      });
      if (!res.ok) { const d = await res.json().catch(() => ({})); throw new Error(d.detail || "Gagal menambah student"); }
      toast("success", "Student berhasil ditambahkan");
      setShowForm(false);
      setForm({ full_name: "", instrument: "piano", level: "pemula" });
      fetchStudents();
    } catch (err) { toast("error", err instanceof Error ? err.message : "Terjadi kesalahan"); }
    finally { setSubmitting(false); }
  };

  const handleUpdate = async () => {
    if (!editStudent) return;
    setSubmitting(true);
    try {
      const token = localStorage.getItem("access_token");
      const res = await fetch(`/api/v1/students/${editStudent.id_student}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify(form),
      });
      if (!res.ok) { const d = await res.json().catch(() => ({})); throw new Error(d.detail || "Gagal mengupdate student"); }
      toast("success", "Student berhasil diupdate");
      setEditStudent(null);
      setForm({ full_name: "", instrument: "piano", level: "pemula" });
      fetchStudents();
    } catch (err) { toast("error", err instanceof Error ? err.message : "Terjadi kesalahan"); }
    finally { setSubmitting(false); }
  };

  const handleDelete = async (student: Student) => {
    if (!confirm(`Yakin ingin menghapus ${student.full_name}?`)) return;
    try {
      const token = localStorage.getItem("access_token");
      const res = await fetch(`/api/v1/students/${student.id_student}`, {
        method: "DELETE",
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!res.ok) { const d = await res.json().catch(() => ({})); throw new Error(d.detail || "Gagal menghapus student"); }
      toast("success", "Student berhasil dihapus");
      fetchStudents();
    } catch (err) { toast("error", err instanceof Error ? err.message : "Terjadi kesalahan"); }
  };

  const openEdit = (student: Student) => {
    setEditStudent(student);
    setForm({ full_name: student.full_name, instrument: student.instrument, level: student.level });
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="font-heading text-2xl font-bold text-[#0F172A]">Student</h1>
          <p className="text-sm text-gray-500">{total} student terdaftar</p>
        </div>
        <button onClick={() => { setShowForm(!showForm); setEditStudent(null); }} className="rounded-lg bg-[#2563EB] px-4 py-2 text-sm font-semibold text-white hover:bg-[#1D4ED8]">
          {showForm ? "Tutup" : "+ Tambah Student"}
        </button>
      </div>

      {/* Create Form */}
      {showForm && (
        <div className="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
          <h2 className="mb-4 font-heading text-lg font-bold text-[#0F172A]">Tambah Student Baru</h2>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Nama Lengkap</label>
              <input value={form.full_name} onChange={e => setForm({ ...form, full_name: e.target.value })} placeholder="Nama student" className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Instrumen</label>
              <select value={form.instrument} onChange={e => setForm({ ...form, instrument: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm">
                {INSTRUMENTS.map(i => <option key={i} value={i}>{INSTRUMENT_LABELS[i]}</option>)}
              </select>
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Level</label>
              <select value={form.level} onChange={e => setForm({ ...form, level: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm">
                <option value="pemula">Pemula</option>
                <option value="dasar">Dasar</option>
                <option value="menengah">Menengah</option>
                <option value="lanjut">Lanjut</option>
              </select>
            </div>
          </div>
          <div className="mt-4 flex gap-2">
            <button onClick={handleCreate} disabled={!form.full_name || submitting} className="rounded-lg bg-[#2563EB] px-5 py-2 text-sm font-semibold text-white hover:bg-[#1D4ED8] disabled:opacity-50">
              {submitting ? "Menyimpan..." : "Simpan"}
            </button>
            <button onClick={() => setShowForm(false)} className="rounded-lg border border-gray-300 px-5 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50">Batal</button>
          </div>
        </div>
      )}

      {/* Edit Modal */}
      {editStudent && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4" onClick={() => setEditStudent(null)}>
          <div className="w-full max-w-md rounded-xl border border-gray-200 bg-white p-6 shadow-xl" onClick={e => e.stopPropagation()}>
            <h2 className="font-heading text-lg font-bold text-[#0F172A]">Edit Student</h2>
            <div className="mt-4 space-y-3">
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Nama</label>
                <input value={form.full_name} onChange={e => setForm({ ...form, full_name: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" />
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Instrumen</label>
                <select value={form.instrument} onChange={e => setForm({ ...form, instrument: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm">
                  {INSTRUMENTS.map(i => <option key={i} value={i}>{INSTRUMENT_LABELS[i]}</option>)}
                </select>
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Level</label>
                <select value={form.level} onChange={e => setForm({ ...form, level: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm">
                  <option value="pemula">Pemula</option>
                  <option value="dasar">Dasar</option>
                  <option value="menengah">Menengah</option>
                  <option value="lanjut">Lanjut</option>
                </select>
              </div>
            </div>
            <div className="mt-6 flex gap-2">
              <button onClick={handleUpdate} disabled={submitting} className="rounded-lg bg-[#2563EB] px-5 py-2 text-sm font-semibold text-white hover:bg-[#1D4ED8]">
                {submitting ? "Menyimpan..." : "Simpan"}
              </button>
              <button onClick={() => setEditStudent(null)} className="rounded-lg border border-gray-300 px-5 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50">Batal</button>
            </div>
          </div>
        </div>
      )}

      {/* Table */}
      {loading ? (
        <SkeletonTable />
      ) : students.length === 0 ? (
        <div className="rounded-xl border border-dashed border-gray-300 bg-gray-50 p-12 text-center">
          <p className="text-gray-500">Belum ada student terdaftar.</p>
        </div>
      ) : (
        <div className="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-100 bg-gray-50">
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Nama</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Instrumen</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Level</th>
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Status</th>
                  <th className="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-gray-500">Aksi</th>
                </tr>
              </thead>
              <tbody>
                {students.map(s => (
                  <tr key={s.id_student} className="border-b border-gray-50 transition-colors hover:bg-gray-50">
                    <td className="px-4 py-3 text-sm font-medium text-[#0F172A]">{s.full_name}</td>
                    <td className="px-4 py-3 text-sm text-gray-600">{INSTRUMENT_LABELS[s.instrument] || s.instrument}</td>
                    <td className="px-4 py-3 text-sm text-gray-600 capitalize">{s.level}</td>
                    <td className="px-4 py-3">
                      <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${s.is_active ? "bg-emerald-100 text-emerald-700" : "bg-red-100 text-red-700"}`}>
                        {s.is_active ? "Aktif" : "Nonaktif"}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-right">
                      <button onClick={() => openEdit(s)} className="mr-2 text-xs font-medium text-[#2563EB] hover:underline">Edit</button>
                      <button onClick={() => handleDelete(s)} className="text-xs font-medium text-red-500 hover:underline">Hapus</button>
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
