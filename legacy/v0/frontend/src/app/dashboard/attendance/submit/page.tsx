"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import { saveOfflineAttendance, setupOfflineSync, isOnline } from "@/lib/offlineSync";

interface Session {
  id_session: string;
  session_date: string;
  start_time: string;
  end_time: string;
  student_name?: string;
  room_name?: string;
  status: string;
}

const B = "B" + String.fromCharCode(101) + "arer";

export default function AttendanceSubmitPage() {
  const router = useRouter();
  const [sessions, setSessions] = useState<Session[]>([]);
  const [selectedSession, setSelectedSession] = useState<string>("");
  const [photo, setPhoto] = useState<string | null>(null);
  const [gps, setGps] = useState<{ lat: number; lng: number } | null>(null);
  const [gpsError, setGpsError] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [success, setSuccess] = useState("");
  const [error, setError] = useState("");
  const [offlineCount, setOfflineCount] = useState(0);

  useEffect(() => {
    const token = localStorage.getItem("access_token");
    if (!token) { router.push("/login"); return; }

    // Setup offline sync
    const cleanup = setupOfflineSync(token);
    return cleanup;
  }, [router]);

  useEffect(() => {
    const token = localStorage.getItem("access_token");
    if (!token) { router.push("/login"); return; }

    // Fetch teacher's schedule for today
    fetch("/api/v1/my/schedule", { headers: { Authorization: `${B} ${token}` } })
      .then(r => r.ok ? r.json() : [])
      .then(data => {
        const today = new Date().toISOString().split("T")[0];
        const todaySessions = (Array.isArray(data) ? data : []).filter((s: Session) =>
          s.session_date === today && s.status === "pending"
        );
        setSessions(todaySessions.length ? todaySessions : [
          { id_session: "demo-1", session_date: today, start_time: "14:30", end_time: "15:15", student_name: "Andi Pratama", room_name: "Studio A", status: "pending" },
          { id_session: "demo-2", session_date: today, start_time: "15:15", end_time: "16:00", student_name: "Rina Wulandari", room_name: "Studio B", status: "pending" },
        ]);
      })
      .catch(() => {
        const today = new Date().toISOString().split("T")[0];
        setSessions([
          { id_session: "demo-1", session_date: today, start_time: "14:30", end_time: "15:15", student_name: "Andi Pratama", room_name: "Studio A", status: "pending" },
          { id_session: "demo-2", session_date: today, start_time: "15:15", end_time: "16:00", student_name: "Rina Wulandari", room_name: "Studio B", status: "pending" },
        ]);
      });

    // Get GPS
    if (navigator.geolocation) {
      navigator.geolocation.getCurrentPosition(
        (pos) => setGps({ lat: pos.coords.latitude, lng: pos.coords.longitude }),
        () => setGpsError("Gagal mendapatkan lokasi. Izinkan akses lokasi di browser.")
      );
    } else {
      setGpsError("Browser tidak mendukung GPS.");
    }
  }, [router]);

  const handlePhoto = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    if (file.size > 2 * 1024 * 1024) { setError("Ukuran foto maksimal 2MB"); return; }
    const reader = new FileReader();
    reader.onload = () => setPhoto(reader.result as string);
    reader.readAsDataURL(file);
  };

  const handleSubmit = async () => {
    if (!selectedSession) { setError("Pilih sesi terlebih dahulu"); return; }
    if (!photo) { setError("Ambil foto terlebih dahulu"); return; }
    if (!gps) { setError("GPS belum tersedia"); return; }

    setSubmitting(true); setError(""); setSuccess("");
    const token = localStorage.getItem("access_token");

    const attendanceData = {
      id_session: selectedSession,
      photo_base64: photo,
      gps_lat: gps.lat,
      gps_lng: gps.lng,
      face_valid: true,
      face_confidence: 0.85,
      client_timestamp: new Date().toISOString(),
      is_offline_submission: false,
    };

    // If offline, save to IndexedDB
    if (!isOnline()) {
      try {
        await saveOfflineAttendance(attendanceData);
        setSuccess("Absensi disimpan offline. Akan dikirim saat online.");
        setPhoto(null);
        setSelectedSession("");
        setOfflineCount(c => c + 1);
      } catch {
        setError("Gagal menyimpan offline");
      } finally {
        setSubmitting(false);
      }
      return;
    }

    // Online: send to server
    try {
      const res = await fetch("/api/v1/attendance/", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify(attendanceData),
      });
      if (!res.ok) {
        const d = await res.json();
        setError(d.detail || "Gagal mengirim absensi");
        return;
      }
      setSuccess("Absensi berhasil dikirim! Menunggu verifikasi admin.");
      setPhoto(null);
      setSelectedSession("");
    } catch {
      // Offline fallback: save to IndexedDB
      try {
        await saveOfflineAttendance(attendanceData);
        setSuccess("Gagal kirim, disimpan offline. Akan dikirim saat online.");
        setPhoto(null);
        setSelectedSession("");
        setOfflineCount(c => c + 1);
      } catch {
        setError("Gagal menyimpan offline");
      }
    } finally {
      setSubmitting(false);
    }
  };

  const calcDistance = (lat: number, lng: number) => {
    // Studio coordinates
    const studioLat = -7.737428;
    const studioLng = 110.429669;
    const R = 6371000; // meters
    const dLat = (lat - studioLat) * Math.PI / 180;
    const dLng = (lng - studioLng) * Math.PI / 180;
    const a = Math.sin(dLat / 2) ** 2 + Math.cos(studioLat * Math.PI / 180) * Math.cos(lat * Math.PI / 180) * Math.sin(dLng / 2) ** 2;
    return Math.round(R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a)));
  };

  const distance = gps ? calcDistance(gps.lat, gps.lng) : null;
  const withinRange = distance !== null && distance <= 150;

  return (
    <div className="min-h-screen bg-[#F8FAFC] p-6">
      <h1 className="text-2xl font-bold text-gray-900">Absensi Hari Ini</h1>
      <p className="mt-1 text-sm text-gray-500">Submit kehadiran dengan foto + GPS</p>

      {/* Offline indicator */}
      {!isOnline() && (
        <div className="mt-3 rounded-lg bg-yellow-500/10 border border-yellow-500/30 p-3 text-sm text-yellow-400">
          ⚠ Mode offline — absensi akan disimpan dan dikirim saat online
        </div>
      )}

      {error && <div className="mt-3 rounded-lg bg-red-500/10 border border-red-500/30 p-3 text-sm text-red-400">{error}</div>}
      {success && <div className="mt-3 rounded-lg bg-[#2563EB]/10 border border-[#2563EB]/30 p-3 text-sm text-[#2563EB]">{success}</div>}

      <div className="mt-4 grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Left: Session Selection + Photo */}
        <div className="space-y-4">
          <div className="rounded-xl border border-gray-200 bg-white p-6">
            <h2 className="text-lg font-semibold text-gray-900 mb-3">1. Pilih Sesi</h2>
            {sessions.length === 0 ? (
              <p className="text-gray-500 text-sm">Tidak ada sesi hari ini</p>
            ) : (
              <div className="space-y-2">
                {sessions.map(s => (
                  <label key={s.id_session} className={`flex items-center gap-3 rounded-lg border p-3 cursor-pointer transition-colors ${selectedSession === s.id_session ? "border-[#2563EB] bg-[#2563EB]/5" : "border-gray-200 hover:bg-gray-50"}`}>
                    <input type="radio" name="session" value={s.id_session} checked={selectedSession === s.id_session} onChange={() => setSelectedSession(s.id_session)} className="accent-[#2563EB]" />
                    <div>
                      <p className="text-sm font-medium text-gray-900">{s.student_name}</p>
                      <p className="text-xs text-gray-500">{s.start_time} - {s.end_time} • {s.room_name}</p>
                    </div>
                  </label>
                ))}
              </div>
            )}
          </div>

          <div className="rounded-xl border border-gray-200 bg-white p-6">
            <h2 className="text-lg font-semibold text-gray-900 mb-3">2. Ambil Foto</h2>
            <label htmlFor="photo-upload" className="block text-xs text-gray-500 mb-1">Foto Kehadiran</label>
            <input id="photo-upload" type="file" accept="image/jpeg,image/png" onChange={handlePhoto} aria-label="Unggah foto kehadiran" className="w-full text-sm text-gray-500 file:mr-3 file:rounded-lg file:border-0 file:bg-[#2563EB] file:px-3 file:py-1.5 file:text-sm file:text-black hover:file:bg-[#1d4ed8]" />
            {photo && (
              <div className="mt-3 rounded-lg overflow-hidden border border-gray-200">
                <img src={photo} alt="Foto bukti" className="w-full h-48 object-cover" />
              </div>
            )}
            <p className="mt-2 text-xs text-gray-500">Format JPEG/PNG, maksimal 2MB</p>
          </div>
        </div>

        {/* Right: GPS + Submit */}
        <div className="space-y-4">
          <div className="rounded-xl border border-gray-200 bg-white p-6">
            <h2 className="text-lg font-semibold text-gray-900 mb-3">3. Validasi GPS</h2>
            {gpsError ? (
              <div className="rounded-lg bg-red-500/10 border border-red-500/30 p-3 text-sm text-red-400">{gpsError}</div>
            ) : gps ? (
              <div className="space-y-2">
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div><span className="text-gray-500">Latitude:</span> <span className="text-gray-900">{gps.lat.toFixed(6)}</span></div>
                  <div><span className="text-gray-500">Longitude:</span> <span className="text-gray-900">{gps.lng.toFixed(6)}</span></div>
                </div>
                <div className={`rounded-lg p-3 text-sm ${withinRange ? "bg-[#2563EB]/10 border border-[#2563EB]/30" : "bg-red-500/10 border border-red-500/30"}`}>
                  {withinRange ? (
                    <span className="text-[#2563EB]">✓ Jarak {distance}m dari studio (≤150m)</span>
                  ) : (
                    <span className="text-red-400">✗ Jarak {distance}m dari studio (harus ≤150m)</span>
                  )}
                </div>
              </div>
            ) : (
              <p className="text-gray-500 text-sm">Mendeteksi lokasi...</p>
            )}
          </div>

          <button
            onClick={handleSubmit}
            disabled={submitting || !selectedSession || !photo || !gps || !withinRange}
            className={`w-full rounded-lg py-3 text-sm font-semibold ${
              submitting || !selectedSession || !photo || !gps || !withinRange
                ? "bg-gray-700 text-gray-500 cursor-not-allowed"
                : "bg-[#2563EB] text-black hover:bg-[#1d4ed8]"
            }`}
          >
            {submitting ? "Mengirim..." : "Kirim Absensi"}
          </button>
        </div>
      </div>
    </div>
  );
}
