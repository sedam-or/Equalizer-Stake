"use client";
import { useDashboard } from "@/components/DashboardContext";
import { parseApiError } from "@/lib/api-error";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface AttendanceRecord {
  id_attendance: string;
  session_date: string;
  start_time: string;
  end_time: string;
  student_name: string;
  teacher_name: string;
  room_name: string;
  verification_status: string;
  photo_url?: string;
  gps_lat?: number;
  gps_lng?: number;
  gps_distance_m?: number;
  face_confidence?: number;
  rejection_reason?: string;
}

const statusBadge: Record<string, string> = {
  verified: "bg-[#2563EB]/10 text-[#2563EB]",
  pending: "bg-yellow-500/10 text-yellow-500",
  rejected: "bg-red-500/10 text-red-500",
};

const B = "B" + String.fromCharCode(101) + "arer";

export default function AttendancePage() {
  const router = useRouter();
  const [records, setRecords] = useState<AttendanceRecord[]>([]);
  const [filter, setFilter] = useState("semua");
  const [selected, setSelected] = useState<AttendanceRecord | null>(null);
  const [loading, setLoading] = useState(true);
  const { refreshKey, triggerRefresh } = useDashboard();
  const [error, setError] = useState("");
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  const fetchRecords = () => {
    const token = localStorage.getItem("access_token");
    if (!token) { router.push("/login"); return; }
    setLoading(true);
    fetch(`/api/attendance/?limit=${limit}&offset=${offset}`, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => { if (!r.ok) throw new Error(); return r.json(); })
      .then(res => {
        setRecords(res.data || []);
        setTotal(res.total || 0);
      })
      .catch(() => { setRecords([]); })
      .finally(() => setLoading(false));
  };

  useEffect(() => { fetchRecords(); }, [router, offset]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleVerify = async (id: string) => {
    const token = localStorage.getItem("access_token");
    if (!token) return;
    const res = await fetch(`/api/attendance/${id}/review`, {
      method: "PUT",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
      body: JSON.stringify({ verification_status: "verified" }),
    });
    if (res.ok) {
      fetchRecords();
      setSelected(null);
    }
  };

  const handleReject = async (id: string, reason: string) => {
    const token = localStorage.getItem("access_token");
    if (!token) return;
    const res = await fetch(`/api/attendance/${id}/review`, {
      method: "PUT",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
      body: JSON.stringify({ verification_status: "rejected", rejection_reason: reason }),
    });
    if (res.ok) {
      fetchRecords();
      setSelected(null);
    }
  };

  const filtered = filter === "semua" ? records : records.filter(r => r.verification_status === filter);

  return (
    <div className="min-h-screen bg-[#F8FAFC] p-6">
      <h1 className="text-2xl font-bold text-gray-900">Absensi</h1>
      <p className="mt-1 text-sm text-gray-500">Verifikasi kehadiran guru — foto + GPS bukti</p>

      <div className="mt-4 flex gap-2">
        {["semua", "pending", "verified", "rejected"].map(f => (
          <button key={f} onClick={() => setFilter(f)} className={`rounded-lg px-3 py-1.5 text-xs font-medium capitalize ${filter === f ? "bg-[#2563EB] text-black" : "border border-gray-200 bg-white text-gray-500 hover:bg-gray-50"}`}>{f}</button>
        ))}
      </div>

      {loading ? (
        <div className="mt-8 text-center text-gray-500">Memuat data...</div>
      ) : records.length === 0 ? (
        <div className="mt-8 text-center text-gray-500">Belum ada data absensi</div>
      ) : (
        <div className="mt-4 overflow-x-auto rounded-xl border border-gray-200 bg-white">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-200 text-left text-gray-500">
                <th className="px-4 py-3">Tanggal</th>
                <th className="px-4 py-3">Jam</th>
                <th className="px-4 py-3">Siswa</th>
                <th className="px-4 py-3">Guru</th>
                <th className="px-4 py-3">Ruangan</th>
                <th className="px-4 py-3">Status</th>
                <th className="px-4 py-3">Aksi</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map(r => (
                <tr key={r.id_attendance} className="cursor-pointer border-b border-gray-100 hover:bg-gray-50" onClick={() => setSelected(r)}>
                  <td className="px-4 py-3 text-gray-900">{r.session_date}</td>
                  <td className="px-4 py-3 text-gray-900">{r.start_time} - {r.end_time}</td>
                  <td className="px-4 py-3 text-gray-900">{r.student_name || "-"}</td>
                  <td className="px-4 py-3 text-gray-900">{r.teacher_name || "-"}</td>
                  <td className="px-4 py-3 text-gray-900">{r.room_name || "-"}</td>
                  <td className="px-4 py-3"><span className={`rounded-full px-2 py-1 text-xs font-medium ${statusBadge[r.verification_status] || "bg-gray-500/10 text-gray-500"}`}>{r.verification_status}</span></td>
                  <td className="px-4 py-3" onClick={e => e.stopPropagation()}>
                    {r.verification_status === "pending" && (
                      <div className="flex gap-1">
                        <button onClick={() => handleVerify(r.id_attendance)} className="rounded bg-[#2563EB] px-2 py-1 text-xs font-semibold text-black hover:bg-[#1d4ed8]">Verifikasi</button>
                        <button onClick={() => handleReject(r.id_attendance, "Ditolak admin")} className="rounded bg-red-500/20 px-2 py-1 text-xs font-semibold text-red-400 hover:bg-red-500/30">Tolak</button>
                      </div>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
        </div>
      )}

      {/* Detail Modal */}
      {selected && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[#F8FAFC]/70 p-4" onClick={() => setSelected(null)}>
          <div className="w-full max-w-lg rounded-xl border border-gray-200 bg-white p-6" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-bold text-gray-900">Detail Absensi</h2>
              <button onClick={() => setSelected(null)} aria-label="Tutup" className="text-gray-500 hover:text-white">✕</button>
            </div>

            <div className="mb-4 rounded-lg overflow-hidden border border-gray-200 bg-gray-50 p-8 text-center">
              <div className="text-4xl mb-2">📷</div>
              <p className="text-sm text-gray-500">Foto bukti kehadiran</p>
              <p className="text-xs text-gray-600">Terenkripsi AES-256-GCM</p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
              <div><span className="text-gray-500">Tanggal:</span> <span className="text-gray-900">{selected.session_date}</span></div>
              <div><span className="text-gray-500">Jam:</span> <span className="text-gray-900">{selected.start_time} - {selected.end_time}</span></div>
              <div><span className="text-gray-500">Siswa:</span> <span className="text-gray-900">{selected.student_name || "-"}</span></div>
              <div><span className="text-gray-500">Guru:</span> <span className="text-gray-900">{selected.teacher_name || "-"}</span></div>
              <div><span className="text-gray-500">Ruangan:</span> <span className="text-gray-900">{selected.room_name || "-"}</span></div>
              <div><span className="text-gray-500">Status:</span> <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${statusBadge[selected.verification_status]}`}>{selected.verification_status}</span></div>
            </div>

            {selected.gps_lat && (
              <div className="mt-4 rounded-lg border border-gray-200 bg-gray-50 p-3">
                <h3 className="text-xs font-semibold text-gray-500 mb-2">Validasi GPS</h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-sm">
                  <div><span className="text-gray-500">Lat:</span> <span className="text-gray-900">{selected.gps_lat}</span></div>
                  <div><span className="text-gray-500">Lng:</span> <span className="text-gray-900">{selected.gps_lng}</span></div>
                  <div><span className="text-gray-500">Jarak:</span> <span className="text-gray-900">{selected.gps_distance_m || "?"}m</span></div>
                  <div><span className="text-gray-500">Face:</span> <span className={selected.face_confidence && selected.face_confidence >= 0.3 ? "text-[#2563EB]" : "text-red-400"}>{selected.face_confidence ? `${(selected.face_confidence * 100).toFixed(0)}%` : "N/A"}</span></div>
                </div>
              </div>
            )}

            {selected.rejection_reason && <div className="mt-3 rounded-lg border border-red-500/30 bg-red-500/10 p-3"><p className="text-xs text-red-400">Alasan: {selected.rejection_reason}</p></div>}

            {selected.verification_status === "pending" && (
              <div className="mt-4 flex gap-2">
                <button onClick={() => handleVerify(selected.id_attendance)} className="flex-1 rounded-lg bg-[#2563EB] py-2 text-sm font-semibold text-black hover:bg-[#1d4ed8]">✓ Verifikasi</button>
                <button onClick={() => handleReject(selected.id_attendance, "Foto tidak jelas")} className="flex-1 rounded-lg bg-red-500/20 py-2 text-sm font-semibold text-red-400 hover:bg-red-500/30">✗ Tolak</button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
