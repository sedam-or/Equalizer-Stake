"use client";

import { useDashboard } from "@/components/DashboardContext";
import { parseApiError } from "@/lib/api-error";

import { useEffect, useState, useCallback } from "react";

const ALL_INSTRUMENTS = ["piano", "gitar", "biola", "drum", "vokal", "keyboard", "flute", "saxophone", "bass"];

interface Teacher {
  id_teacher: string;
  id_user: string;
  full_name: string;
  instruments_specialization: string[];
  employment_type: string;
  max_students_per_session: number;
  is_active: boolean;
  username: string | null;
}

export default function TeachersPage() {
  const [teachers, setTeachers] = useState<Teacher[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editId, setEditId] = useState<string | null>(null);
  const [form, setForm] = useState({
    id_user: "",
    full_name: "",
    instruments_specialization: [] as string[],
    employment_type: "tetap",
    max_students_per_session: 1,
  });
  const [error, setError] = useState("");
  const { refreshKey } = useDashboard();

  const token = typeof window !== "undefined" ? localStorage.getItem("access_token") : null;

  const fetchTeachers = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const res = await fetch("/api/v1/teachers", { headers: { Authorization: `Bearer ${token}` } });
      if (res.ok) setTeachers(await res.json());
    } catch { /* ignore */ }
    setLoading(false);
  }, [token]);

  useEffect(() => { fetchTeachers(); }, [fetchTeachers, refreshKey]);

  const handleInstrumentToggle = (inst: string) => {
    setForm((prev) => ({
      ...prev,
      instruments_specialization: prev.instruments_specialization.includes(inst)
        ? prev.instruments_specialization.filter((i) => i !== inst)
        : [...prev.instruments_specialization, inst],
    }));
  };

  const handleCreate = async () => {
    setError("");
    if (!form.full_name || form.instruments_specialization.length === 0) {
      setError("Nama dan minimal 1 instrumen wajib diisi");
      return;
    }
    try {
      const method = editId ? "PUT" : "POST";
      const url = editId ? `/api/v1/teachers/${editId}` : "/api/v1/teachers";
      const body = editId
        ? { full_name: form.full_name, instruments_specialization: form.instruments_specialization, employment_type: form.employment_type, max_students_per_session: form.max_students_per_session }
        : { ...form };
      const res = await fetch(url, {
        method,
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify(body),
      });
      if (!res.ok) {
        const data = await res.json();
        setError(parseApiError(data));
        return;
      }
      setShowForm(false);
      setEditId(null);
      setForm({ id_user: "", full_name: "", instruments_specialization: [], employment_type: "tetap", max_students_per_session: 1 });
      fetchTeachers();
    } catch { setError("Terjadi kesalahan"); }
  };

  const handleEdit = (t: Teacher) => {
    setEditId(t.id_teacher);
    setForm({
      id_user: t.id_user,
      full_name: t.full_name,
      instruments_specialization: t.instruments_specialization,
      employment_type: t.employment_type,
      max_students_per_session: t.max_students_per_session,
    });
    setShowForm(true);
  };

  const handleDelete = async (id: string) => {
    if (!confirm("Yakin ingin menonaktifkan guru ini?")) return;
    await fetch(`/api/v1/teachers/${id}`, { method: "DELETE", headers: { Authorization: `Bearer ${token}` } });
    fetchTeachers();
  };

  return (
    <div className="p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="font-heading text-2xl font-bold text-gray-900">Guru Musik</h1>
          <p className="text-sm text-gray-500">Kelola data guru dan spesialisasi instrumen</p>
        </div>
        <button onClick={() => { setEditId(null); setForm({ id_user: "", full_name: "", instruments_specialization: [], employment_type: "tetap", max_students_per_session: 1 }); setShowForm(true); }} className="rounded-lg bg-[#2563EB] px-4 py-2 text-sm font-semibold text-white hover:bg-[#1D4ED8]">
          + Tambah Guru
        </button>
      </div>

      {/* Form modal */}
      {showForm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30">
          <div className="w-full max-w-md rounded-xl bg-white p-6 shadow-xl">
            <h2 className="text-lg font-bold text-gray-900">{editId ? "Edit Guru" : "Tambah Guru"}</h2>
            {error && <p className="mt-2 text-sm text-red-600">{error}</p>}
            <div className="mt-4 space-y-3">
              {!editId && (
                <div>
                  <label className="mb-1 block text-sm font-medium text-gray-700">ID User (UUID) *</label>
                  <input type="text" value={form.id_user} onChange={(e) => setForm({ ...form, id_user: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm" placeholder="UUID user dengan role teacher" />
                </div>
              )}
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Nama Lengkap *</label>
                <input type="text" value={form.full_name} onChange={(e) => setForm({ ...form, full_name: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm" />
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Instrumen *</label>
                <div className="flex flex-wrap gap-2">
                  {ALL_INSTRUMENTS.map((inst) => (
                    <button key={inst} onClick={() => handleInstrumentToggle(inst)} className={`rounded-full px-3 py-1 text-xs font-medium capitalize ${form.instruments_specialization.includes(inst) ? "bg-[#2563EB] text-white" : "bg-gray-100 text-gray-700"}`}>
                      {inst}
                    </button>
                  ))}
                </div>
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Tipe</label>
                <select value={form.employment_type} onChange={(e) => setForm({ ...form, employment_type: e.target.value })} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm">
                  <option value="tetap">Tetap</option>
                  <option value="freelance">Freelance</option>
                </select>
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Kapasitas/Sesi</label>
                <input type="number" min={1} max={10} value={form.max_students_per_session} onChange={(e) => setForm({ ...form, max_students_per_session: parseInt(e.target.value) || 1 })} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm" />
              </div>
            </div>
            <div className="mt-4 flex justify-end gap-2">
              <button onClick={() => { setShowForm(false); setEditId(null); }} className="rounded-lg border border-gray-300 px-4 py-2 text-sm">Batal</button>
              <button onClick={handleCreate} className="rounded-lg bg-[#2563EB] px-4 py-2 text-sm font-semibold text-white hover:bg-[#1D4ED8]">{editId ? "Simpan" : "Tambah"}</button>
            </div>
          </div>
        </div>
      )}

      {/* Table */}
      {loading ? (
        <div className="mt-6 p-8 text-center text-gray-500">Memuat...</div>
      ) : teachers.length === 0 ? (
        <div className="mt-6 rounded-xl border border-gray-200 bg-white p-8 text-center text-gray-500">Belum ada guru</div>
      ) : (
        <div className="mt-6 overflow-x-auto rounded-xl border border-gray-200 bg-white">
          <table className="w-full text-left text-sm">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-4 py-3 font-medium text-gray-600">Nama</th>
                <th className="px-4 py-3 font-medium text-gray-600">Username</th>
                <th className="px-4 py-3 font-medium text-gray-600">Instrumen</th>
                <th className="px-4 py-3 font-medium text-gray-600">Tipe</th>
                <th className="px-4 py-3 font-medium text-gray-600">Kapasitas</th>
                <th className="px-4 py-3 font-medium text-gray-600">Status</th>
                <th className="px-4 py-3 font-medium text-gray-600">Aksi</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {teachers.map((t) => (
                <tr key={t.id_teacher} className="hover:bg-gray-50">
                  <td className="px-4 py-3 font-medium text-gray-900">{t.full_name}</td>
                  <td className="px-4 py-3 text-gray-600">{t.username || "-"}</td>
                  <td className="px-4 py-3">
                    <div className="flex flex-wrap gap-1">
                      {t.instruments_specialization.map((inst) => (
                        <span key={inst} className="rounded-full bg-blue-50 px-2 py-0.5 text-xs font-medium text-[#2563EB] capitalize">{inst}</span>
                      ))}
                    </div>
                  </td>
                  <td className="px-4 py-3 capitalize text-gray-600">{t.employment_type}</td>
                  <td className="px-4 py-3 text-gray-600">{t.max_students_per_session}</td>
                  <td className="px-4 py-3">
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${t.is_active ? "bg-green-50 text-green-700" : "bg-red-50 text-red-700"}`}>
                      {t.is_active ? "Aktif" : "Nonaktif"}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex gap-2">
                      <button onClick={() => handleEdit(t)} className="text-xs text-[#2563EB] hover:underline">Edit</button>
                      <button onClick={() => handleDelete(t.id_teacher)} className="text-xs text-red-500 hover:underline">Nonaktifkan</button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
