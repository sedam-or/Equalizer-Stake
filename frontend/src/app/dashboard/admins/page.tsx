"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

interface Admin {
  id_user: string;
  username: string;
  email: string;
  is_active: boolean;
  created_at: string;
}

export default function AdminsPage() {
  const router = useRouter();
  const [admins, setAdmins] = useState<Admin[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [showCreate, setShowCreate] = useState(false);
  const [createForm, setCreateForm] = useState({ username: "", email: "", password: "" });
  const [createError, setCreateError] = useState("");
  const [createLoading, setCreateLoading] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Admin | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState("");
  const [deleteError, setDeleteError] = useState("");

  const getToken = () => localStorage.getItem("token");

  const fetchAdmins = async () => {
    const token = getToken();
    if (!token) { router.push("/login"); return; }
    try {
      const res = await fetch("/api/admin-mgmt/admins", {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!res.ok) throw new Error("Gagal memuat data");
      const data = await res.json();
      setAdmins(data.admins);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Error");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchAdmins(); }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setCreateError("");
    setCreateLoading(true);
    const token = getToken();
    try {
      const res = await fetch("/api/admin-mgmt/admins", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify(createForm),
      });
      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.detail || "Gagal membuat admin");
      }
      setShowCreate(false);
      setCreateForm({ username: "", email: "", password: "" });
      fetchAdmins();
    } catch (e: unknown) {
      setCreateError(e instanceof Error ? e.message : "Error");
    } finally {
      setCreateLoading(false);
    }
  };

  const handleToggle = async (admin: Admin) => {
    const token = getToken();
    const action = admin.is_active ? "deactivate" : "activate";
    await fetch(`/api/admin-mgmt/admins/${admin.id_user}/${action}`, {
      method: "PUT",
      headers: { Authorization: `Bearer ${token}` },
    });
    fetchAdmins();
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    setDeleteError("");
    const token = getToken();
    try {
      const res = await fetch(`/api/admin-mgmt/admins/${deleteTarget.id_user}`, {
        method: "DELETE",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify({ confirm_username: deleteConfirm }),
      });
      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.detail || "Gagal menghapus admin");
      }
      setDeleteTarget(null);
      setDeleteConfirm("");
      fetchAdmins();
    } catch (e: unknown) {
      setDeleteError(e instanceof Error ? e.message : "Error");
    }
  };

  if (loading) return <div className="p-8 text-center text-gray-500">Memuat...</div>;

  return (
    <div>
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[#000000]">Kelola Admin</h1>
          <p className="mt-1 text-sm text-gray-500">Buat, nonaktifkan, atau hapus akun Admin</p>
        </div>
        <button
          onClick={() => setShowCreate(!showCreate)}
          className="rounded-lg bg-[#08CB00] px-4 py-2 text-sm font-semibold text-black hover:bg-[#0aad00]"
        >
          + Tambah Admin
        </button>
      </div>

      {error && <div className="mt-4 rounded-lg bg-red-50 p-3 text-sm text-red-600">{error}</div>}

      {/* Create form */}
      {showCreate && (
        <form onSubmit={handleCreate} className="mt-6 rounded-xl border border-gray-200 bg-white p-6">
          <h2 className="text-lg font-semibold text-[#000000]">Buat Admin Baru</h2>
          {createError && <div className="mt-2 rounded bg-red-50 p-2 text-sm text-red-600">{createError}</div>}
          <div className="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-3">
            <input
              type="text" placeholder="Username" required minLength={6}
              value={createForm.username}
              onChange={(e) => setCreateForm({ ...createForm, username: e.target.value })}
              className="rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-[#08CB00] focus:outline-none"
            />
            <input
              type="email" placeholder="Email" required
              value={createForm.email}
              onChange={(e) => setCreateForm({ ...createForm, email: e.target.value })}
              className="rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-[#08CB00] focus:outline-none"
            />
            <input
              type="password" placeholder="Password" required minLength={8}
              value={createForm.password}
              onChange={(e) => setCreateForm({ ...createForm, password: e.target.value })}
              className="rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-[#08CB00] focus:outline-none"
            />
          </div>
          <div className="mt-4 flex gap-2">
            <button
              type="submit" disabled={createLoading}
              className="rounded-lg bg-[#08CB00] px-4 py-2 text-sm font-semibold text-black hover:bg-[#0aad00] disabled:opacity-50"
            >
              {createLoading ? "Membuat..." : "Buat Admin"}
            </button>
            <button type="button" onClick={() => setShowCreate(false)} className="rounded-lg border px-4 py-2 text-sm text-gray-600 hover:bg-gray-50">
              Batal
            </button>
          </div>
        </form>
      )}

      {/* Admin list */}
      <div className="mt-6 space-y-3">
        {admins.length === 0 ? (
          <p className="text-sm text-gray-400">Belum ada akun Admin</p>
        ) : (
          admins.map((admin) => (
            <div key={admin.id_user} className="flex items-center justify-between rounded-xl border border-gray-200 bg-white p-4">
              <div>
                <p className="font-medium text-[#000000]">{admin.username}</p>
                <p className="text-sm text-gray-500">{admin.email}</p>
                <p className="text-xs text-gray-400">Dibuat: {new Date(admin.created_at).toLocaleDateString("id-ID")}</p>
              </div>
              <div className="flex items-center gap-2">
                <span className={`rounded-full px-2 py-1 text-xs font-medium ${admin.is_active ? "bg-green-100 text-green-700" : "bg-red-100 text-red-700"}`}>
                  {admin.is_active ? "Aktif" : "Nonaktif"}
                </span>
                <button
                  onClick={() => handleToggle(admin)}
                  className="rounded-lg border px-3 py-1.5 text-xs font-medium text-gray-600 hover:bg-gray-50"
                >
                  {admin.is_active ? "Nonaktifkan" : "Aktifkan"}
                </button>
                <button
                  onClick={() => setDeleteTarget(admin)}
                  className="rounded-lg border border-red-200 px-3 py-1.5 text-xs font-medium text-red-600 hover:bg-red-50"
                >
                  Hapus
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Delete confirmation modal */}
      {deleteTarget && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-full max-w-md rounded-xl bg-white p-6 shadow-xl">
            <h2 className="text-lg font-bold text-red-600">Hapus Admin</h2>
            <p className="mt-2 text-sm text-gray-600">
              Tindakan ini tidak dapat dibatalkan. Data PII akan dianonimkan. Ketik <strong>{deleteTarget.username}</strong> untuk konfirmasi.
            </p>
            {deleteError && <div className="mt-2 rounded bg-red-50 p-2 text-sm text-red-600">{deleteError}</div>}
            <input
              type="text"
              value={deleteConfirm}
              onChange={(e) => setDeleteConfirm(e.target.value)}
              placeholder={`Ketik "${deleteTarget.username}"`}
              className="mt-4 w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-red-500 focus:outline-none"
            />
            <div className="mt-4 flex gap-2">
              <button
                onClick={handleDelete}
                disabled={deleteConfirm !== deleteTarget.username}
                className="rounded-lg bg-red-600 px-4 py-2 text-sm font-semibold text-white hover:bg-red-700 disabled:opacity-50"
              >
                Hapus Permanen
              </button>
              <button
                onClick={() => { setDeleteTarget(null); setDeleteConfirm(""); setDeleteError(""); }}
                className="rounded-lg border px-4 py-2 text-sm text-gray-600 hover:bg-gray-50"
              >
                Batal
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
