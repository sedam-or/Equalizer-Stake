"use client";

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
  verified: "bg-[#08CB00]/10 text-[#08CB00]",
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
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  const fetchRecords = () => {
    const token = localStorage.getItem("token");
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
    const token = localStorage.getItem("token");
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
    const token = localStorage.getItem("token");
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
    <div className="min-h-screen bg-black p-6">
      <h1 className="text-2xl font-bold text-[#EEEEEE]">Absensi</h1>
      <p className="mt-1 text-sm text-gray-400">Verifikasi kehadiran guru — foto + GPS bukti</p>

      <div className="mt-4 flex gap-2">
        {["semua", "pending", "verified", "rejected"].map(f => (
          <button key={f} onClick={() => setFilter(f)} className={`rounded-lg px-3 py-1.5 text-xs font-medium capitalize ${filter === f ? "bg-[#08CB00] text-black" : "border border-[#253900] bg-[#111] text-gray-400 hover:bg-[#1a1a1a]"}`}>{f}</button>
        ))}
      </div>

      {loading ? (
        <div className="mt-8 text-center text-gray-500">Memuat data...</div>
      ) : records.length === 0 ? (
        <div className="mt-8 text-center text-gray-500">Belum ada data absensi</div>
      ) : (
        <div className="mt-4 overflow-x-auto rounded-xl border border-[#253900] bg-[#111]">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[#253900] text-left text-gray-400">
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
                <tr key={r.id_attendance} className="cursor-pointer border-b border-[#1a1a1a] hover:bg-[#1a1a1a]" onClick={() => setSelected(r)}>
                  <td className="px-4 py-3 text-[#EEEEEE]">{r.session_date}</td>
                  <td className="px-4 py-3 text-[#EEEEEE]">{r.start_time} - {r.end_time}</td>
                  <td className="px-4 py-3 text-[#EEEEEE]">{r.student_name || "-"}</td>
                  <td className="px-4 py-3 text-[#EEEEEE]">{r.teacher_name || "-"}</td>
                  <td className="px-4 py-3 text-[#EEEEEE]">{r.room_name || "-"}</td>
                  <td className="px-4 py-3"><span className={`rounded-full px-2 py-1 text-xs font-medium ${statusBadge[r.verification_status] || "bg-gray-500/10 text-gray-400"}`}>{r.verification_status}</span></td>
                  <td className="px-4 py-3" onClick={e => e.stopPropagation()}>
                    {r.verification_status === "pending" && (
                      <div className="flex gap-1">
                        <button onClick={() => handleVerify(r.id_attendance)} className="rounded bg-[#08CB00] px-2 py-1 text-xs font-semibold text-black hover:bg-[#06a800]">Verifikasi</button>
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
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4" onClick={() => setSelected(null)}>
          <div className="w-full max-w-lg rounded-xl border border-[#253900] bg-[#111] p-6" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-bold text-[#EEEEEE]">Detail Absensi</h2>
              <button onClick={() => setSelected(null)} aria-label="Tutup" className="text-gray-400 hover:text-white">✕</button>
            </div>

            <div className="mb-4 rounded-lg overflow-hidden border border-[#253900] bg-[#1a1a1a] p-8 text-center">
              <div className="text-4xl mb-2">📷</div>
              <p className="text-sm text-gray-400">Foto bukti kehadiran</p>
              <p className="text-xs text-gray-600">Terenkripsi AES-256-GCM</p>
            </div>

            <div className="grid grid-cols-2 gap-3 text-sm">
              <div><span className="text-gray-400">Tanggal:</span> <span className="text-[#EEEEEE]">{selected.session_date}</span></div>
              <div><span className="text-gray-400">Jam:</span> <span className="text-[#EEEEEE]">{selected.start_time} - {selected.end_time}</span></div>
              <div><span className="text-gray-400">Siswa:</span> <span className="text-[#EEEEEE]">{selected.student_name || "-"}</span></div>
              <div><span className="text-gray-400">Guru:</span> <span className="text-[#EEEEEE]">{selected.teacher_name || "-"}</span></div>
              <div><span className="text-gray-400">Ruangan:</span> <span className="text-[#EEEEEE]">{selected.room_name || "-"}</span></div>
              <div><span className="text-gray-400">Status:</span> <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${statusBadge[selected.verification_status]}`}>{selected.verification_status}</span></div>
            </div>

            {selected.gps_lat && (
              <div className="mt-4 rounded-lg border border-[#253900] bg-[#0a0a0a] p-3">
                <h3 className="text-xs font-semibold text-gray-400 mb-2">Validasi GPS</h3>
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div><span className="text-gray-500">Lat:</span> <span className="text-[#EEEEEE]">{selected.gps_lat}</span></div>
                  <div><span className="text-gray-500">Lng:</span> <span className="text-[#EEEEEE]">{selected.gps_lng}</span></div>
                  <div><span className="text-gray-500">Jarak:</span> <span className="text-[#EEEEEE]">{selected.gps_distance_m || "?"}m</span></div>
                  <div><span className="text-gray-500">Face:</span> <span className={selected.face_confidence && selected.face_confidence >= 0.3 ? "text-[#08CB00]" : "text-red-400"}>{selected.face_confidence ? `${(selected.face_confidence * 100).toFixed(0)}%` : "N/A"}</span></div>
                </div>
              </div>
            )}

            {selected.rejection_reason && <div className="mt-3 rounded-lg border border-red-500/30 bg-red-500/10 p-3"><p className="text-xs text-red-400">Alasan: {selected.rejection_reason}</p></div>}

            {selected.verification_status === "pending" && (
              <div className="mt-4 flex gap-2">
                <button onClick={() => handleVerify(selected.id_attendance)} className="flex-1 rounded-lg bg-[#08CB00] py-2 text-sm font-semibold text-black hover:bg-[#06a800]">✓ Verifikasi</button>
                <button onClick={() => handleReject(selected.id_attendance, "Foto tidak jelas")} className="flex-1 rounded-lg bg-red-500/20 py-2 text-sm font-semibold text-red-400 hover:bg-red-500/30">✗ Tolak</button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
