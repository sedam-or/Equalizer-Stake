"use client";
/**
 * T-065/T-111: My Schedule — connected to real API with instrument tabs
 * GET /api/my/schedule?instrument=xxx
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface Session {
  id_session: string;
  session_date: string;
  start_time: string;
  end_time: string;
  status: string;
  session_type: string;
  student_name?: string;
  instrument?: string;
  student_id?: string;
}

const STATUS_COLORS: Record<string, string> = {
  scheduled: "bg-blue-500/10 text-blue-400",
  completed: "bg-[#08CB00]/10 text-[#08CB00]",
  cancelled: "bg-red-500/10 text-red-400",
  "no-show": "bg-gray-500/10 text-gray-400",
  rescheduled: "bg-yellow-500/10 text-yellow-500",
};

const STATUS_LABELS: Record<string, string> = {
  scheduled: "Terjadwal",
  completed: "Selesai",
  cancelled: "Dibatalkan",
  "no-show": "Tanpa Kabar",
  rescheduled: "Dijadwal Ulang",
};

export default function MySchedulePage() {
  const router = useRouter();
  const [sessions, setSessions] = useState<Session[]>([]);
  const [loading, setLoading] = useState(true);
  const [role, setRole] = useState("");
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const [instruments, setInstruments] = useState<string[]>([]);
  const [activeInstrument, setActiveInstrument] = useState<string>("");
  const limit = 20;

  // Fetch teacher instruments from profile
  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) return;
    fetch("/api/auth/me/profile", { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : null)
      .then(profile => {
        if (profile?.instruments?.length) {
          setInstruments(profile.instruments);
        }
      })
      .catch(() => {});
  }, []);

  // Fetch schedule
  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    const r = localStorage.getItem("role") || "";
    setRole(r);

    let url = `/api/my/schedule?limit=${limit}&offset=${offset}`;
    if (activeInstrument) url += `&instrument=${activeInstrument}`;

    fetch(url, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : Promise.reject())
      .then(res => {
        setSessions(res.data || (Array.isArray(res) ? res : []));
        setTotal(res.total || 0);
      })
      .catch(() => setSessions([]))
      .finally(() => setLoading(false));
  }, [router, offset, activeInstrument]);

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-[#EEEEEE]">Jadwal Saya</h1>
        <p className="text-sm text-gray-400">
          {role === "teacher" ? "Jadwal mengajar Anda" : "Jadwal kursus anak-anak Anda"}
        </p>
      </div>

      {/* Instrument Tabs (teacher only) */}
      {role === "teacher" && instruments.length > 0 && (
        <div className="flex gap-2 flex-wrap">
          <button
            onClick={() => { setActiveInstrument(""); setOffset(0); }}
            className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
              activeInstrument === ""
                ? "bg-[#08CB00] text-black"
                : "bg-[#1a1a1a] text-gray-400 hover:text-[#EEEEEE]"
            }`}
          >
            Semua
          </button>
          {instruments.map(inst => (
            <button
              key={inst}
              onClick={() => { setActiveInstrument(inst); setOffset(0); }}
              className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                activeInstrument === inst
                  ? "bg-[#08CB00] text-black"
                  : "bg-[#1a1a1a] text-gray-400 hover:text-[#EEEEEE]"
              }`}
            >
              {inst.charAt(0).toUpperCase() + inst.slice(1)}
            </button>
          ))}
        </div>
      )}

      <div className="bg-[#111] rounded-xl border border-[#253900] overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-400">Memuat jadwal...</div>
        ) : sessions.length === 0 ? (
          <div className="p-8 text-center text-gray-400">Belum ada jadwal</div>
        ) : (
          <>
          <div className="divide-y divide-[#1a1a1a]">
            {sessions.map(s => (
              <div key={s.id_session} className="flex items-center gap-4 px-4 py-3 hover:bg-[#1a1a1a]">
                <div className="w-16 text-center">
                  <p className="text-lg font-bold text-[#EEEEEE]">{new Date(s.session_date).getDate()}</p>
                  <p className="text-xs text-gray-400">{new Date(s.session_date).toLocaleDateString("id-ID", { month: "short" })}</p>
                </div>
                <div className="flex-1">
                  <p className="text-sm font-medium text-[#EEEEEE]">{s.start_time} — {s.end_time}</p>
                  <p className="text-xs text-gray-400">
                    {s.student_name ? `${s.student_name} · ` : ""}{s.instrument ? `${s.instrument} · ` : ""}{s.session_type}
                  </p>
                </div>
                <span className={`px-2 py-1 rounded-full text-xs font-medium ${STATUS_COLORS[s.status] || "bg-gray-500/10 text-gray-400"}`}>
                  {STATUS_LABELS[s.status] || s.status}
                </span>
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
