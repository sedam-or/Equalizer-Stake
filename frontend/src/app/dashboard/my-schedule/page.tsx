"use client";
/**
 * T-043: My Schedule (Teacher + Guardian)
 * - Teacher: jadwal mengajar
 * - Guardian: jadwal anak-anak
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

interface Session {
  id_session: string;
  session_date: string;
  start_time: string;
  end_time: string;
  status: string;
  session_type: string;
  student_id?: string;
}

const STATUS_COLORS: Record<string, string> = {
  scheduled: "bg-blue-100 text-blue-700",
  completed: "bg-green-100 text-green-700",
  cancelled: "bg-red-100 text-red-700",
  "no-show": "bg-gray-100 text-gray-500",
  rescheduled: "bg-yellow-100 text-yellow-700",
};

export default function MySchedulePage() {
  const router = useRouter();
  const [sessions, setSessions] = useState<Session[]>([]);
  const [loading, setLoading] = useState(true);
  const [role, setRole] = useState("");

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    const r = localStorage.getItem("role") || "";
    setRole(r);
    fetch("/api/my/schedule", { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : [])
      .then(setSessions)
      .finally(() => setLoading(false));
  }, [router]);

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-[#000000]">Jadwal Saya</h1>
        <p className="text-sm text-gray-500">
          {role === "teacher" ? "Jadwal mengajar Anda" : "Jadwal kursus anak-anak Anda"}
        </p>
      </div>

      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-500">Memuat jadwal...</div>
        ) : sessions.length === 0 ? (
          <div className="p-8 text-center text-gray-400">Belum ada jadwal</div>
        ) : (
          <div className="divide-y divide-gray-100">
            {sessions.map(s => (
              <div key={s.id_session} className="flex items-center gap-4 px-4 py-3 hover:bg-gray-50">
                <div className="w-16 text-center">
                  <p className="text-lg font-bold text-gray-900">{new Date(s.session_date).getDate()}</p>
                  <p className="text-xs text-gray-500">{new Date(s.session_date).toLocaleDateString("id-ID", { month: "short" })}</p>
                </div>
                <div className="flex-1">
                  <p className="text-sm font-medium text-gray-900">{s.start_time} — {s.end_time}</p>
                  <p className="text-xs text-gray-500 capitalize">{s.session_type}</p>
                </div>
                <span className={`px-2 py-0.5 rounded text-xs font-medium ${STATUS_COLORS[s.status] || "bg-gray-100"}`}>
                  {s.status}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
