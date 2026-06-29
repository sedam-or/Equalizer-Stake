"use client";
/**
 * T-043: My Children (Guardian)
 * UC-06: Multi-student portal
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

interface Child {
  id_student: string;
  full_name: string;
  instrument: string;
  level: string;
  enrollment_status: string;
  eligible_for_exam: boolean;
}

const STATUS_COLORS: Record<string, string> = {
  active: "bg-green-100 text-green-700",
  trial: "bg-blue-100 text-blue-700",
  cuti: "bg-yellow-100 text-yellow-700",
  inactive: "bg-gray-100 text-gray-500",
};

export default function MyChildrenPage() {
  const router = useRouter();
  const [children, setChildren] = useState<Child[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    fetch("/api/my/children", { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : [])
      .then(setChildren)
      .finally(() => setLoading(false));
  }, [router]);

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-[#000000]">Anak-anak Saya</h1>
        <p className="text-sm text-gray-500">{children.length} siswa terdaftar</p>
      </div>

      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-500">Memuat...</div>
        ) : children.length === 0 ? (
          <div className="p-8 text-center text-gray-400">Belum ada siswa terdaftar</div>
        ) : (
          <div className="divide-y divide-gray-100">
            {children.map(c => (
              <div key={c.id_student} className="flex items-center gap-4 px-4 py-4 hover:bg-gray-50">
                <div className="w-10 h-10 bg-[#08CB00]/10 rounded-full flex items-center justify-center text-[#08CB00] font-bold">
                  {c.full_name.charAt(0)}
                </div>
                <div className="flex-1">
                  <p className="font-medium text-gray-900">{c.full_name}</p>
                  <p className="text-xs text-gray-500 capitalize">{c.instrument} — {c.level}</p>
                </div>
                <div className="text-right">
                  <span className={`px-2 py-0.5 rounded text-xs font-medium ${STATUS_COLORS[c.enrollment_status] || "bg-gray-100"}`}>
                    {c.enrollment_status}
                  </span>
                  <p className="text-xs text-gray-400 mt-1">
                    {c.eligible_for_exam ? "✅ Lulus ujian" : "Belum eligible"}
                  </p>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
