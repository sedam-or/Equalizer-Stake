"use client";
import { useDashboard } from "@/components/DashboardContext";
import { parseApiError } from "@/lib/api-error";
/**
 * Override Attendance (Owner only)
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface Session {
  id_session: string;
  student_name?: string;
  teacher_name?: string;
  date?: string;
  time_start?: string;
  status?: string;
}

export default function OverridePage() {
  const router = useRouter();
  const [sessions, setSessions] = useState<Session[]>([]);
  const [loadingSessions, setLoadingSessions] = useState(true);
  const { refreshKey, triggerRefresh } = useDashboard();
  const [form, setForm] = useState({
    session_id: "",
    status: "present",
    reason: "",
  });
  const [msg, setMsg] = useState("");
  const [err, setErr] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  const getToken = () => localStorage.getItem("access_token");

  useEffect(() => {
    const fetchSessions = async () => {
      const token = getToken();
      if (!token) { router.push("/login"); return; }
      try {
        const res = await fetch(`/api/sessions/?limit=${limit}&offset=${offset}`, {
          headers: { Authorization: `Bearer ${token}` },
        });
        if (!res.ok) throw new Error("Gagal memuat sesi");
        const data = await res.json();
        setSessions(data.data || data.sessions || []);
        setTotal(data.total || 0);
      } catch {
        setSessions([]);
      } finally {
        setLoadingSessions(false);
      }
    };
    fetchSessions();
  }, [router, offset]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setMsg(""); setErr("");
    if (!form.session_id.trim()) { setErr("ID Sesi wajib diisi"); return; }
    if (!form.reason.trim()) { setErr("Alasan override wajib diisi"); return; }
    const token = getToken();
    if (!token) { router.push("/login"); return; }
    setSubmitting(true);
    try {
      const res = await fetch("/api/v1/override/", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify({
          session_id: form.session_id,
          status: form.status,
          reason: form.reason,
        }),
      });
      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.detail || "Gagal menyimpan override");
      }
      setMsg(`Override kehadiran sesi ${form.session_id} berhasil disimpan`);
      setForm({ session_id: "", status: "present", reason: "" });
    } catch (e: unknown) {
      setErr(e instanceof Error ? e.message : "Error");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Override Kehadiran</h1>
        <p className="text-sm text-gray-500">Ubah status kehadiran sesi secara manual (Owner only)</p>
      </div>

      {msg && <div className="p-3 bg-[#2563EB]/10 border border-gray-200 rounded-lg text-[#2563EB] text-sm">{msg}</div>}
      {err && <div className="p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 text-sm">{err}</div>}

      <form onSubmit={handleSubmit} className="bg-white rounded-xl border border-gray-200 p-6 space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-500 mb-1">ID Sesi</label>
          <input
            type="text"
            value={form.session_id}
            onChange={e => setForm({ ...form, session_id: e.target.value })}
            placeholder="Contoh: sess_abc123"
            className="w-full px-3 py-2 bg-[#F8FAFC] border border-gray-200 rounded-lg text-sm text-gray-900 focus:border-[#2563EB] focus:outline-none"
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-500 mb-1">Status Kehadiran</label>
          <select
            value={form.status}
            onChange={e => setForm({ ...form, status: e.target.value })}
            className="w-full px-3 py-2 bg-[#F8FAFC] border border-gray-200 rounded-lg text-sm text-gray-900 focus:border-[#2563EB] focus:outline-none"
          >
            <option value="present">Hadir</option>
            <option value="absent">Tidak Hadir</option>
            <option value="late">Terlambat</option>
            <option value="excused">Izin</option>
          </select>
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-500 mb-1">Alasan Override</label>
          <textarea
            value={form.reason}
            onChange={e => setForm({ ...form, reason: e.target.value })}
            placeholder="Jelaskan alasan override..."
            rows={3}
            className="w-full px-3 py-2 bg-[#F8FAFC] border border-gray-200 rounded-lg text-sm text-gray-900 focus:border-[#2563EB] focus:outline-none resize-none"
          />
        </div>
        <button
          type="submit"
          disabled={submitting}
          className="w-full px-4 py-2 bg-[#2563EB] text-black text-sm font-semibold rounded-lg hover:bg-[#1d4ed8] disabled:opacity-50"
        >
          {submitting ? "Menyimpan..." : "Simpan Override"}
        </button>
      </form>

      {/* Recent sessions */}
      <div className="bg-white rounded-xl border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-3">Sesi Terbaru</h2>
        {loadingSessions ? (
          <p className="text-sm text-gray-500">Memuat sesi...</p>
        ) : sessions.length === 0 ? (
          <p className="text-sm text-gray-500">Tidak ada sesi ditemukan</p>
        ) : (
          <>
          <div className="space-y-2">
            {sessions.slice(0, 10).map((s) => (
              <div
                key={s.id_session}
                className="flex items-center justify-between rounded-lg border border-gray-200 bg-[#F8FAFC] p-3 cursor-pointer hover:border-[#2563EB] transition-colors"
                onClick={() => setForm({ ...form, session_id: s.id_session })}
              >
                <div>
                  <p className="text-sm font-mono text-gray-900">{s.id_session}</p>
                  <p className="text-xs text-gray-500">
                    {s.student_name || "-"} {s.teacher_name ? `· ${s.teacher_name}` : ""}
                    {s.date ? ` · ${s.date}` : ""}
                    {s.time_start ? ` ${s.time_start}` : ""}
                  </p>
                </div>
                {s.status && (
                  <span className="text-xs text-gray-500">{s.status}</span>
                )}
              </div>
            ))}
          </div>
          <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
          </>
        )}
      </div>
    </div>
  );
}
