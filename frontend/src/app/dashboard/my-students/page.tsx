"use client";
import { parseApiError } from "@/lib/api-error";

import { useEffect, useState, useCallback } from "react";

const INSTRUMENTS = ["piano", "gitar", "biola", "drum", "vokal", "keyboard", "flute", "saxophone", "bass"];
const LEVELS = ["pemula", "dasar", "menengah", "lanjut"];

interface Student {
  id_student: string;
  id_guardian: string;
  full_name: string;
  instrument: string;
  level: string;
  enrollment_status: string;
  enrollment_date: string;
}

export default function MyStudentsPage() {
  const [students, setStudents] = useState<Student[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editId, setEditId] = useState<string | null>(null);
  const [form, setForm] = useState({ full_name: "", date_of_birth: "", instrument: "piano", level: "pemula" });
  const [error, setError] = useState("");

  const token = typeof window !== "undefined" ? localStorage.getItem("access_token") : null;

  const fetchStudents = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const res = await fetch("/api/v1/students/", { headers: { Authorization: `Bearer ${token}` } });
      if (res.ok) {
        const data = await res.json();
        setStudents(data.data || []);
      }
    } catch { /* ignore */ }
    setLoading(false);
  }, [token]);

  useEffect(() => { fetchStudents(); }, [fetchStudents]);

  const handleSave = async () => {
    setError("");
    if (!form.full_name.trim()) { setError("Nama anak wajib diisi"); return; }
    try {
      const method = editId ? "PATCH" : "POST";
      const url = editId ? `/api/v1/students/${editId}` : "/api/v1/students/";
      const body = editId
        ? { full_name: form.full_name, instrument: form.instrument, level: form.level }
        : { full_name: form.full_name, instrument: form.instrument, level: form.level, enrollment_date: new Date().toISOString().split("T")[0] };
      const res = await fetch(url, {
        method,
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify(body),
      });
      if (!res.ok) {
        const data = await res.json();
        setError(data.detail || "Gagal menyimpan");
        return;
      }
      setShowForm(false);
      setEditId(null);
      setForm({ full_name: "", date_of_birth: "", instrument: "piano", level: "pemula" });
      fetchStudents();
    } catch { setError("Terjadi kesalahan"); }
  };

  const handleEdit = (s: Student) => {
    setEditId(s.id_student);
    setForm({ full_name: s.full_name, date_of_birth: "", instrument: s.instrument, level: s.level });
    setShowForm(true);
  };

  const handleDelete = async (id: string) => {
    if (!confirm("Yakin ingin menghapus data anak ini?")) return;
    await fetch(`/api/v1/students/${id}`, { method: "DELETE", headers: { Authorization: `Bearer ${token}` } });
    fetchStudents();
  };

  const openAdd = () => {
    setEditId(null);
    setForm({ full_name: "", date_of_birth: "", instrument: "piano", level: "pemula" });
    setShowForm(true);
  };

  return (
    <div className="p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="font-heading text-2xl font-bold text-gray-900">Anak Saya</h1>
          <p className="text-sm text-gray-500">Kelola data anak yang terdaftar kursus</p>
        </div>
        <button onClick={openAdd} className="rounded-lg bg-[#2563EB] px-4 py-2 text-sm font-semibold text-white hover:bg-[#1D4ED8]">
          + Tambah Anak
        </button>
      </div>

      {/* Form modal */}
      {showForm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30">
          <div className="w-full max-w-md rounded-xl bg-white p-6 shadow-xl">
            <h2 className="text-lg font-bold text-gray-900">{editId ? "Edit Anak" : "Tambah Anak"}</h2>
            {error && <p className="mt-2 text-sm text-red-600">{error}</p>}
            <div className="mt-4 space-y-3">
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Nama Lengkap *</label>
                <input type="text" value={form.full_name} onChange={(e) => setForm({ ...form, full_name: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm" placeholder="Nama anak" />
              </div>
              {!editId && (
                <div>
                  <label className="mb-1 block text-sm font-medium text-gray-700">Tanggal Lahir</label>
                  <input type="date" value={form.date_of_birth} onChange={(e) => setForm({ ...form, date_of_birth: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm" />
                </div>
              )}
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Instrumen *</label>
                <select value={form.instrument} onChange={(e) => setForm({ ...form, instrument: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm capitalize">
                  {INSTRUMENTS.map((i) => <option key={i} value={i}>{i}</option>)}
                </select>
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Level</label>
                <select value={form.level} onChange={(e) => setForm({ ...form, level: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm capitalize">
                  {LEVELS.map((l) => <option key={l} value={l}>{l}</option>)}
                </select>
              </div>
            </div>
            <div className="mt-4 flex justify-end gap-2">
              <button onClick={() => { setShowForm(false); setEditId(null); }} className="rounded-lg border border-gray-300 px-4 py-2 text-sm">Batal</button>
              <button onClick={handleSave} className="rounded-lg bg-[#2563EB] px-4 py-2 text-sm font-semibold text-white hover:bg-[#1D4ED8]">{editId ? "Simpan" : "Tambah"}</button>
            </div>
          </div>
        </div>
      )}

      {/* List */}
      {loading ? (
        <div className="mt-6 p-8 text-center text-gray-500">Memuat...</div>
      ) : students.length === 0 ? (
        <div className="mt-6 rounded-xl border border-gray-200 bg-white p-8 text-center">
          <span className="text-4xl">👶</span>
          <p className="mt-2 text-gray-500">Belum ada data anak</p>
          <button onClick={openAdd} className="mt-4 rounded-lg bg-[#2563EB] px-4 py-2 text-sm font-semibold text-white hover:bg-[#1D4ED8]">+ Tambah Anak Pertama</button>
        </div>
      ) : (
        <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {students.map((s) => (
            <div key={s.id_student} className="rounded-xl border border-gray-200 bg-white p-5">
              <div className="flex items-start justify-between">
                <div>
                  <h3 className="font-heading text-lg font-bold text-gray-900">{s.full_name}</h3>
                  <p className="mt-1 text-sm capitalize text-[#2563EB]">{s.instrument}</p>
                  <p className="text-xs capitalize text-gray-500">{s.level}</p>
                </div>
                <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${s.enrollment_status === "active" ? "bg-green-50 text-green-700" : "bg-yellow-50 text-yellow-700"}`}>
                  {s.enrollment_status}
                </span>
              </div>
              <div className="mt-4 flex gap-2">
                <button onClick={() => handleEdit(s)} className="rounded-lg border border-gray-300 px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50">Edit</button>
                <button onClick={() => handleDelete(s.id_student)} className="rounded-lg border border-red-200 px-3 py-1.5 text-xs font-medium text-red-600 hover:bg-red-50">Hapus</button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
