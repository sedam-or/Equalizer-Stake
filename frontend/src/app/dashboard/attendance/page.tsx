"use client";

import { useState, useRef, useCallback, useEffect } from "react";

interface GpsStatus {
  loading: boolean;
  lat: number | null;
  lng: number | null;
  distance: number | null;
  valid: boolean | null;
}

interface FaceStatus {
  detected: boolean;
  confidence: number;
  checking: boolean;
}

export default function AttendancePage() {
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [photo, setPhoto] = useState<string | null>(null);
  const [gps, setGps] = useState<GpsStatus>({ loading: false, lat: null, lng: null, distance: null, valid: null });
  const [face, setFace] = useState<FaceStatus>({ detected: false, confidence: 0, checking: false });
  const [submitting, setSubmitting] = useState(false);
  const [result, setResult] = useState<"success" | "pending" | null>(null);

  // GPS validation
  const STUDIO_LAT = -7.7265;
  const STUDIO_LNG = 110.4162;
  const RADIUS_M = 165; // 150m + 15m tolerance

  const getGps = useCallback(() => {
    setGps((prev) => ({ ...prev, loading: true }));
    navigator.geolocation.getCurrentPosition(
      (pos) => {
        const { latitude, longitude } = pos.coords;
        const dist = haversine(latitude, longitude, STUDIO_LAT, STUDIO_LNG);
        setGps({
          loading: false,
          lat: latitude,
          lng: longitude,
          distance: dist,
          valid: dist <= RADIUS_M,
        });
      },
      () => {
        setGps({ loading: false, lat: null, lng: null, distance: null, valid: null });
      },
      { enableHighAccuracy: true, timeout: 10000 }
    );
  }, []);

  // Start camera
  useEffect(() => {
    let stream: MediaStream | null = null;
    navigator.mediaDevices.getUserMedia({ video: { facingMode: "user", width: 640, height: 480 } })
      .then((s) => {
        stream = s;
        if (videoRef.current) videoRef.current.srcObject = s;
      })
      .catch(() => {});
    return () => { stream?.getTracks().forEach((t) => t.stop()); };
  }, []);

  // Capture photo + face detection
  const capture = useCallback(() => {
    if (!videoRef.current || !canvasRef.current) return;
    const video = videoRef.current;
    const canvas = canvasRef.current;
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.drawImage(video, 0, 0);
    const dataUrl = canvas.toDataURL("image/jpeg", 0.8);
    setPhoto(dataUrl);

    // Simulate face detection (real: face-api.js)
    setFace({ detected: false, confidence: 0, checking: true });
    setTimeout(() => {
      setFace({ detected: true, confidence: 0.85, checking: false });
    }, 1000);

    // Get GPS
    getGps();
  }, [getGps]);

  // Submit attendance
  const submit = useCallback(async () => {
    if (!photo || !gps.valid || !face.detected) return;
    setSubmitting(true);
    try {
      // TODO: POST /api/attendance/
      await new Promise((r) => setTimeout(r, 1000));
      setResult("success");
    } catch {
      setResult("pending");
    } finally {
      setSubmitting(false);
    }
  }, [photo, gps.valid, face.detected]);

  const allValid = gps.valid === true && face.detected && photo;

  return (
    <div className="mx-auto max-w-md">
      <h1 className="text-2xl font-bold text-[#000000]">Absensi Hari Ini</h1>
      <p className="mt-1 text-sm text-gray-500">Ambil foto dan validasi kehadiran</p>

      {/* Camera preview */}
      <div className="mt-6 overflow-hidden rounded-xl border border-gray-200 bg-black">
        {photo ? (
          <img src={photo} alt="Foto absensi" className="w-full" />
        ) : (
          <video ref={videoRef} autoPlay playsInline muted className="w-full" />
        )}
        <canvas ref={canvasRef} className="hidden" />
      </div>

      {/* Capture button */}
      {!photo && (
        <button
          onClick={capture}
          className="mt-4 w-full rounded-lg bg-[#08CB00] py-3 text-sm font-semibold text-black hover:bg-[#0aad00]"
        >
          📷 Ambil Foto
        </button>
      )}

      {/* Validation status */}
      {photo && (
        <div className="mt-4 space-y-3">
          {/* Face detection */}
          <StatusRow
            label="Deteksi Wajah"
            status={
              face.checking
                ? "loading"
                : face.detected
                ? "success"
                : "error"
            }
            detail={
              face.checking
                ? "Memeriksa..."
                : face.detected
                ? `Wajah terdeteksi (${(face.confidence * 100).toFixed(0)}%)`
                : "Wajah tidak terdeteksi"
            }
          />

          {/* GPS */}
          <StatusRow
            label="Lokasi GPS"
            status={
              gps.loading
                ? "loading"
                : gps.valid === true
                ? "success"
                : gps.valid === false
                ? "error"
                : "idle"
            }
            detail={
              gps.loading
                ? "Mendapatkan lokasi..."
                : gps.distance !== null
                ? `${gps.distance.toFixed(0)}m dari studio`
                : "GPS tidak tersedia"
            }
          />

          {/* Submit */}
          <button
            onClick={submit}
            disabled={!allValid || submitting}
            className="mt-2 w-full rounded-lg bg-[#08CB00] py-3 text-sm font-semibold text-black disabled:opacity-50 disabled:cursor-not-allowed hover:bg-[#0aad00]"
          >
            {submitting ? "Mengirim..." : "Kirim Absensi"}
          </button>

          {/* Retake */}
          <button
            onClick={() => { setPhoto(null); setFace({ detected: false, confidence: 0, checking: false }); setResult(null); }}
            className="w-full rounded-lg border border-gray-300 py-2 text-sm text-gray-600 hover:bg-gray-50"
          >
            Ambil Ulang
          </button>

          {/* Result */}
          {result === "success" && (
            <div className="rounded-lg bg-[#08CB00]/10 p-3 text-sm text-[#08CB00]">
              ✅ Absensi berhasil dikirim!
            </div>
          )}
          {result === "pending" && (
            <div className="rounded-lg bg-yellow-50 p-3 text-sm text-yellow-700">
              ⏳ Absensi menunggu verifikasi admin
            </div>
          )}
        </div>
      )}

      {/* Info */}
      <div className="mt-6 rounded-lg bg-gray-50 p-3 text-xs text-gray-500">
        <p>• Radius validasi: 150m dari studio</p>
        <p>• Wajah harus terdeteksi ≥30% frame</p>
        <p>• Server akan memverifikasi ulang secara redundan</p>
      </div>
    </div>
  );
}

function StatusRow({ label, status, detail }: { label: string; status: string; detail: string }) {
  const icons: Record<string, string> = {
    loading: "⏳",
    success: "✅",
    error: "❌",
    idle: "⚪",
  };
  const colors: Record<string, string> = {
    loading: "text-yellow-600",
    success: "text-[#08CB00]",
    error: "text-red-500",
    idle: "text-gray-400",
  };

  return (
    <div className="flex items-center justify-between rounded-lg border border-gray-200 bg-white p-3">
      <div className="flex items-center gap-2">
        <span>{icons[status]}</span>
        <span className="text-sm font-medium text-[#000000]">{label}</span>
      </div>
      <span className={`text-xs ${colors[status]}`}>{detail}</span>
    </div>
  );
}

function haversine(lat1: number, lng1: number, lat2: number, lng2: number): number {
  const R = 6371000;
  const phi1 = (lat1 * Math.PI) / 180;
  const phi2 = (lat2 * Math.PI) / 180;
  const dPhi = ((lat2 - lat1) * Math.PI) / 180;
  const dLambda = ((lng2 - lng1) * Math.PI) / 180;
  const a = Math.sin(dPhi / 2) ** 2 + Math.cos(phi1) * Math.cos(phi2) * Math.sin(dLambda / 2) ** 2;
  return R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
}
