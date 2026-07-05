"use client";
/**
 * T-065/T-111/T-112: My Schedule — connected to real API
 * GET /api/my/schedule?instrument=xxx&student_id=xxx
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

interface Child {
  id_student: string;
  full_name: string;
  instrument: string;
}

const STATUS_COLORS: Record<string, string> = {
  scheduled: "bg-blue-500/10 text-blue-400",
  completed: "bg-[#2563EB]/10 text-[#2563EB]",
  cancelled: "bg-red-500/10 text-red-400",
  "no-show": "bg-gray-500/10 text-gray-500",
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
  const [children, setChildren] = useState<Child[]>([]);
  const [activeStudentId, setActiveStudentId] = useState<string>("");
  const limit = 20;

  // Fetch teacher instruments or guardian children
  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) return;
    const r = localStorage.getItem("role") || "";

    if (r === "teacher") {
      fetch("/api/v1/auth/me/profile", { headers: { Authorization: `Bearer ${token}` } })
        .then(r => r.ok ? r.json() : null)
        .then(profile => {
          if (profile?.instruments?.length) setInstruments(profile.instruments);
        })
        .catch(() => {});
    } else if (r === "guardian") {
      fetch("/api/v1/my/children?limit=50", { headers: { Authorization: `Bearer ${token}` } })
        .then(r => r.ok ? r.json() : null)
        .then(res => {
          if (res?.data) setChildren(res.data);
        })
        .catch(() => {});
    }
  }, []);

  // Fetch schedule
  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    const r = localStorage.getItem("role") || "";
    setRole(r);

    let url = `/api/my/schedule?limit=${limit}&offset=${offset}`;
    if (activeInstrument) url += `&instrument=${activeInstrument}`;
    if (activeStudentId) url += `&student_id=${activeStudentId}`;

    fetch(url, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : Promise.reject())
      .then(res => {
        setSessions(res.data || (Array.isArray(res) ? res : []));
        setTotal(res.total || 0);
      })
      .catch(() => setSessions([]))
      .finally(() => setLoading(false));
  }, [router, offset, activeInstrument, activeStudentId]);

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Jadwal Saya</h1>
        <p className="text-sm text-gray-500">
          {role === "teacher" ? "Jadwal mengajar Anda" : "Jadwal kursus anak-anak Anda"}
        </p>
      </div>

      {/* Student Dropdown (guardian only) */}
      {role === "guardian" && children.length > 1 && (
        <select
          value={activeStudentId}
          onChange={e => { setActiveStudentId(e.target.value); setOffset(0); }}
          className="bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-sm text-gray-900 focus:outline-none focus:border-[#2563EB]"
        >
          <option value="">Semua Anak</option>
          {children.map(c => (
            <option key={c.id_student} value={c.id_student}>
              {c.full_name} ({c.instrument})
            </option>
          ))}
        </select>
      )}

      {/* Instrument Tabs (teacher only) */}
      {role === "teacher" && instruments.length > 0 && (
        <div className="flex gap-2 flex-wrap">
          <button
            onClick={() => { setActiveInstrument(""); setOffset(0); }}
            className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
              activeInstrument === ""
                ? "bg-[#2563EB] text-black"
                : "bg-gray-50 text-gray-500 hover:text-gray-900"
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
                  ? "bg-[#2563EB] text-black"
                  : "bg-gray-50 text-gray-500 hover:text-gray-900"
              }`}
            >
              {inst.charAt(0).toUpperCase() + inst.slice(1)}
            </button>
          ))}
        </div>
      )}

      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-500">Memuat jadwal...</div>
        ) : sessions.length === 0 ? (
          <div className="p-8 text-center text-gray-500">Belum ada jadwal</div>
        ) : (
          <>
          <div className="divide-y divide-[#1a1a1a]">
            {sessions.map(s => (
              <div key={s.id_session} className="flex items-center gap-4 px-4 py-3 hover:bg-gray-50">
                <div className="w-16 text-center">
                  <p className="text-lg font-bold text-gray-900">{new Date(s.session_date).getDate()}</p>
                  <p className="text-xs text-gray-500">{new Date(s.session_date).toLocaleDateString("id-ID", { month: "short" })}</p>
                </div>
                <div className="flex-1">
                  <p className="text-sm font-medium text-gray-900">{s.start_time} — {s.end_time}</p>
                  <p className="text-xs text-gray-500">
                    {s.student_name ? `${s.student_name} · ` : ""}{s.instrument ? `${s.instrument} · ` : ""}{s.session_type}
                  </p>
                </div>
                <span className={`px-2 py-1 rounded-full text-xs font-medium ${STATUS_COLORS[s.status] || "bg-gray-500/10 text-gray-500"}`}>
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
