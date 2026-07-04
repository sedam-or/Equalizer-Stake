"use client";
/**
 * T-065: My Children (Guardian) — connected to real API
 * GET /api/my/children
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface Child {
  id_student: string;
  full_name: string;
  instrument: string;
  level: string;
  enrollment_status: string;
  eligible_for_exam: boolean;
}

const STATUS_COLORS: Record<string, string> = {
  active: "bg-[#2563EB]/10 text-[#2563EB]",
  trial: "bg-blue-500/10 text-blue-400",
  cuti: "bg-yellow-500/10 text-yellow-500",
  inactive: "bg-gray-500/10 text-gray-500",
};

const STATUS_LABELS: Record<string, string> = {
  active: "Aktif",
  trial: "Trial",
  cuti: "Cuti",
  inactive: "Tidak Aktif",
};

export default function MyChildrenPage() {
  const router = useRouter();
  const [children, setChildren] = useState<Child[]>([]);
  const [loading, setLoading] = useState(true);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    fetch(`/api/my/children?limit=${limit}&offset=${offset}`, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : Promise.reject())
      .then(res => {
        setChildren(res.data || (Array.isArray(res) ? res : []));
        setTotal(res.total || 0);
      })
      .catch(() => setChildren([]))
      .finally(() => setLoading(false));
  }, [router, offset]);

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Anak-anak Saya</h1>
        <p className="text-sm text-gray-500">Informasi kursus dan status pendaftaran anak Anda</p>
      </div>

      {loading ? (
        <div className="p-8 text-center text-gray-500">Memuat data...</div>
      ) : children.length === 0 ? (
        <div className="p-8 text-center text-gray-500">Belum ada data anak</div>
      ) : (
        <>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {children.map(child => (
            <div key={child.id_student} className="bg-white rounded-xl border border-gray-200 p-5 space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold text-gray-900">{child.full_name}</h3>
                <span className={`rounded-full px-2 py-1 text-xs font-medium ${STATUS_COLORS[child.enrollment_status] || STATUS_COLORS.inactive}`}>
                  {STATUS_LABELS[child.enrollment_status] || child.enrollment_status}
                </span>
              </div>
              <div className="space-y-1.5">
                <p className="text-sm text-gray-500">Instrumen: <span className="text-gray-900">{child.instrument}</span></p>
                <p className="text-sm text-gray-500">Level: <span className="text-gray-900 capitalize">{child.level}</span></p>
                <p className="text-sm text-gray-500">Ujian: <span className={child.eligible_for_exam ? "text-[#2563EB]" : "text-gray-500"}>
                  {child.eligible_for_exam ? "Memenuhi Syarat" : "Belum Memenuhi Syarat"}
                </span></p>
              </div>
            </div>
          ))}
        </div>
        <div className="mt-4">
          <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
        </div>
        </>
      )}
    </div>
  );
}
