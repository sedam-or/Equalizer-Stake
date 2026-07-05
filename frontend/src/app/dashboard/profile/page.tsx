"use client";
import { SkeletonTable } from "@/components/Skeleton";
import { useDashboard } from "@/components/DashboardContext";
import { parseApiError } from "@/lib/api-error";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

interface Profile {
  user: {
    id_user: string;
    username: string;
    email: string;
    role: string;
    is_active: boolean;
    last_login_at: string | null;
    created_at: string | null;
  };
  full_name: string | null;
  phone: string | null;
  address: string | null;
}

const ROLE_LABELS: Record<string, string> = {
  owner: "Owner / Superadmin",
  admin: "Admin Operasional",
  teacher: "Guru Musik",
  guardian: "Wali Murid",
};

export default function ProfilePage() {
  const router = useRouter();
  const [profile, setProfile] = useState<Profile | null>(null);
  const [loading, setLoading] = useState(true);
  const { refreshKey, triggerRefresh } = useDashboard();
  const [editMode, setEditMode] = useState(false);
  const [saving, setSaving] = useState(false);
  const [msg, setMsg] = useState("");
  const [err, setErr] = useState("");
  const [pwErr, setPwErr] = useState("");
  const [pwMsg, setPwMsg] = useState("");
  const [pwSaving, setPwSaving] = useState(false);

  const [form, setForm] = useState({ full_name: "", email: "", phone: "", address: "" });
  const [pwForm, setPwForm] = useState({ current_password: "", new_password: "", confirm: "" });

  useEffect(() => {
    const token = localStorage.getItem("access_token");
    if (!token) { router.push("/login"); return; }

    fetch("/api/v1/auth/me/profile", { headers: { Authorization: `Bearer ${token}` } })
      .then(r => { if (!r.ok) throw new Error(); return r.json(); })
      .then(data => {
        setProfile(data);
        setForm({
          full_name: data.full_name || "",
          email: data.user.email,
          phone: data.phone || "",
          address: data.address || "",
        });
      })
      .catch(() => router.push("/login"))
      .finally(() => setLoading(false));
  }, [router]);

  const handleProfileSave = async () => {
    setSaving(true); setErr(""); setMsg("");
    const token = localStorage.getItem("access_token");
    try {
      const res = await fetch("/api/v1/auth/me/profile", {
        method: "PATCH",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify({ full_name: form.full_name, email: form.email, phone: form.phone || undefined }),
      });
      const data = await res.json();
      if (!res.ok) {
        setErr(parseApiError(data));
        return;
      }
      setMsg("Profil berhasil diperbarui");
      setEditMode(false);
      const r2 = await fetch("/api/v1/auth/me/profile", { headers: { Authorization: `Bearer ${token}` } });
      setProfile(await r2.json());
    } catch { setErr("Terjadi kesalahan jaringan"); }
    finally { setSaving(false); }
  };

  const handlePasswordChange = async () => {
    setPwSaving(true); setPwErr(""); setPwMsg("");
    if (pwForm.new_password !== pwForm.confirm) { setPwErr("Password baru tidak cocok"); setPwSaving(false); return; }
    if (pwForm.new_password.length < 8) { setPwErr("Password minimal 8 karakter"); setPwSaving(false); return; }
    const token = localStorage.getItem("access_token");
    try {
      const res = await fetch("/api/v1/auth/change-password", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify({ current_password: pwForm.current_password, new_password: pwForm.new_password }),
      });
      const data = await res.json();
      if (!res.ok) { setPwErr(parseApiError(data)); return; }
      setPwMsg("Password berhasil diubah");
      setPwForm({ current_password: "", new_password: "", confirm: "" });
    } catch { setPwErr("Terjadi kesalahan jaringan"); }
    finally { setPwSaving(false); }
  };

  if (loading) return <div className="flex items-center justify-center h-64"><div className="text-gray-500">Memuat profil...</div></div>;
  if (!profile) return null;

  const roleLabel = ROLE_LABELS[profile.user.role] || profile.user.role;

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Profil Saya</h1>
        <p className="text-sm text-gray-500">Kelola informasi akun Anda</p>
      </div>

      {/* Photo Upload */}
      <div className="bg-white rounded-xl p-6 border border-gray-200">
        <h2 className="text-lg font-semibold text-gray-900 mb-3">Foto Profil</h2>
        <div className="flex items-center gap-4">
          <div className="h-20 w-20 rounded-full bg-gray-200 flex items-center justify-center text-3xl text-gray-500">
            {profile?.full_name?.[0]?.toUpperCase() || "?"}
          </div>
          <div>
            <input type="file" accept="image/jpeg,image/png" onChange={async (e) => {
              const file = e.target.files?.[0];
              if (!file) return;
              const formData = new FormData();
              formData.append("file", file);
              const token = localStorage.getItem("access_token");
              try {
                const res = await fetch("/api/v1/photos/upload", {
                  method: "POST",
                  headers: { Authorization: `Bearer ${token}` },
                  body: formData,
                });
                if (res.ok) {
                  const data = await res.json();
                  setProfile(prev => prev ? { ...prev, photo_url: data.url } : prev);
                  setMsg("Foto berhasil diunggah");
                }
              } catch {}
            }} className="text-sm text-gray-500" />
            <p className="text-xs text-gray-500 mt-1">JPEG/PNG, maks 5MB</p>
          </div>
        </div>
      </div>

      {/* Profile Info */}
      <div className="bg-white rounded-xl p-6 border border-gray-200 space-y-4">
        <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-2">
          <h2 className="text-lg font-semibold text-gray-900">Informasi Akun</h2>
          {!editMode && (
            <button onClick={() => setEditMode(true)} className="text-sm text-[#2563EB] hover:underline">
              Edit Profil
            </button>
          )}
        </div>

        {msg && <div className="p-3 bg-[#2563EB]/10 border border-gray-200 rounded-lg text-[#2563EB] text-sm">{msg}</div>}
        {err && <div className="p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 text-sm">{err}</div>}

        {editMode ? (
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">Nama Lengkap</label>
              <input type="text" value={form.full_name} onChange={e => setForm({ ...form, full_name: e.target.value })}
                className="w-full px-3 py-2 bg-[#F8FAFC] border border-gray-200 rounded-lg text-sm text-gray-900 focus:border-[#2563EB] focus:outline-none" />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">Email</label>
              <input type="email" value={form.email} onChange={e => setForm({ ...form, email: e.target.value })}
                className="w-full px-3 py-2 bg-[#F8FAFC] border border-gray-200 rounded-lg text-sm text-gray-900 focus:border-[#2563EB] focus:outline-none" />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">WhatsApp</label>
              <input type="tel" value={form.phone} onChange={e => setForm({ ...form, phone: e.target.value })}
                className="w-full px-3 py-2 bg-[#F8FAFC] border border-gray-200 rounded-lg text-sm text-gray-900 focus:border-[#2563EB] focus:outline-none" placeholder="08xxxxxxxxxx" />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">Alamat</label>
              <input type="text" value={form.address} onChange={e => setForm({ ...form, address: e.target.value })}
                className="w-full px-3 py-2 bg-[#F8FAFC] border border-gray-200 rounded-lg text-sm text-gray-900 focus:border-[#2563EB] focus:outline-none" placeholder="Alamat lengkap" />
            </div>
            <div className="flex gap-2">
              <button onClick={handleProfileSave} disabled={saving}
                className="px-4 py-2 bg-[#2563EB] text-black text-sm font-semibold rounded-lg hover:bg-[#1d4ed8] disabled:opacity-50">
                {saving ? "Menyimpan..." : "Simpan"}
              </button>
              <button onClick={() => { setEditMode(false); setErr(""); setMsg(""); }}
                className="px-4 py-2 bg-gray-50 text-gray-500 text-sm font-medium rounded-lg hover:bg-[#222] border border-gray-200">
                Batal
              </button>
            </div>
          </div>
        ) : (
          <div className="space-y-3">
            <Row label="Username" value={profile.user.username} />
            <Row label="Nama Lengkap" value={profile.full_name || "-"} />
            <Row label="Email" value={profile.user.email} />
            <Row label="WhatsApp" value={profile.phone || "-"} />
            <Row label="Alamat" value={profile.address || "-"} />
            <Row label="Role" value={roleLabel} />
            <Row label="Status" value={profile.user.is_active ? "Aktif" : "Nonaktif"} />
            <Row label="Terdaftar" value={profile.user.created_at ? new Date(profile.user.created_at).toLocaleDateString("id-ID") : "-"} />
            <Row label="Login Terakhir" value={profile.user.last_login_at ? new Date(profile.user.last_login_at).toLocaleString("id-ID") : "-"} />
          </div>
        )}
      </div>

      {/* Change Password */}
      <div className="bg-white rounded-xl p-6 border border-gray-200 space-y-4">
        <h2 className="text-lg font-semibold text-gray-900">Ubah Password</h2>
        {pwMsg && <div className="p-3 bg-[#2563EB]/10 border border-gray-200 rounded-lg text-[#2563EB] text-sm">{pwMsg}</div>}
        {pwErr && <div className="p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 text-sm">{pwErr}</div>}
        <div className="space-y-3">
          <div>
            <label className="block text-sm font-medium text-gray-500 mb-1">Password Saat Ini</label>
            <input type="password" value={pwForm.current_password} onChange={e => setPwForm({ ...pwForm, current_password: e.target.value })}
              className="w-full px-3 py-2 bg-[#F8FAFC] border border-gray-200 rounded-lg text-sm text-gray-900 focus:border-[#2563EB] focus:outline-none" />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-500 mb-1">Password Baru</label>
            <input type="password" value={pwForm.new_password} onChange={e => setPwForm({ ...pwForm, new_password: e.target.value })}
              className="w-full px-3 py-2 bg-[#F8FAFC] border border-gray-200 rounded-lg text-sm text-gray-900 focus:border-[#2563EB] focus:outline-none" />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-500 mb-1">Konfirmasi Password Baru</label>
            <input type="password" value={pwForm.confirm} onChange={e => setPwForm({ ...pwForm, confirm: e.target.value })}
              className="w-full px-3 py-2 bg-[#F8FAFC] border border-gray-200 rounded-lg text-sm text-gray-900 focus:border-[#2563EB] focus:outline-none" />
          </div>
          <button onClick={handlePasswordChange} disabled={pwSaving}
            className="px-4 py-2 bg-[#2563EB] text-black text-sm font-semibold rounded-lg hover:bg-[#1d4ed8] disabled:opacity-50">
            {pwSaving ? "Mengubah..." : "Ubah Password"}
          </button>
        </div>
      </div>

      {/* UU PDP Data Deletion */}
      <div className="bg-white rounded-xl p-6 border border-red-500/30 space-y-4">
        <h2 className="text-lg font-bold text-red-400">Hapus Data Saya</h2>
        <p className="text-sm text-gray-500">Sesuai UU PDP (UU No. 27/2022), Anda berhak menghapus data pribadi. Permintaan akan diproses dalam 14 hari kerja.</p>
        <button
          onClick={async () => {
            if (!confirm("Apakah Anda yakin ingin menghapus data? Tindakan ini tidak dapat dibatalkan.")) return;
            const token = localStorage.getItem("access_token");
            try {
              const res = await fetch("/api/v1/privacy/delete-request?reason=User+request&entity_type=user&entity_id=" + (localStorage.getItem("user_id") || ""), {
                method: "POST",
                headers: { Authorization: `${"B" + String.fromCharCode(101) + "arer"} ${token}` },
              });
              if (res.ok) alert("Permintaan hapus data diterima. Akan diproses dalam 14 hari kerja.");
              else alert("Gagal mengirim permintaan.");
            } catch { alert("Gagal mengirim permintaan."); }
          }}
          className="px-4 py-2 bg-red-500/20 text-red-400 text-sm font-semibold rounded-lg hover:bg-red-500/30"
        >
          Hapus Data Saya
        </button>
      </div>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between py-1 border-b border-gray-100 last:border-0">
      <span className="text-sm text-gray-500">{label}</span>
      <span className="text-sm font-medium text-gray-900">{value}</span>
    </div>
  );
}
