"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface Student {
  id_student: string;
  id_guardian: string;
  full_name: string;
  instrument: string;
  level: string;
  enrollment_status: string;
}

const badge: Record<string, string> = {
  active: "bg-[#2563EB]/10 text-[#2563EB]",
  inactive: "bg-red-500/10 text-red-500",
  cuti: "bg-yellow-500/10 text-yellow-500",
  trial: "bg-blue-500/10 text-blue-500",
};

export default function StudentsPage() {
  const router = useRouter();
  const [students, setStudents] = useState<Student[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    fetch(`/api/students/?limit=${limit}&offset=${offset}`, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : { data: [], total: 0 })
      .then(res => {
        setStudents(res.data || []);
        setTotal(res.total || 0);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [router, offset]);

  const filtered = students.filter(
    (s) =>
      s.full_name.toLowerCase().includes(search.toLowerCase()) ||
      s.instrument.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="min-h-screen bg-black p-6">
      <h1 className="text-2xl font-bold text-[#EEEEEE]">Siswa</h1>
      <p className="mt-1 text-sm text-gray-400">Daftar siswa terdaftar</p>

      <div className="mt-4">
        <input
          type="text"
          placeholder="Cari nama atau instrumen..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full max-w-sm rounded-lg border border-[#CBD5E1] bg-[#111] px-4 py-2 text-sm text-[#EEEEEE] placeholder-gray-500 outline-none focus:border-[#2563EB]"
        />
      </div>

      <div className="mt-4 overflow-x-auto rounded-xl border border-[#CBD5E1] bg-[#111]">
        {loading ? (
          <div className="p-8 text-center text-gray-400">Memuat data siswa...</div>
        ) : (
          <>
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-[#CBD5E1] text-left text-gray-400">
                  <th className="px-4 py-3">Nama</th>
                  <th className="px-4 py-3">Instrumen</th>
                  <th className="px-4 py-3">Level</th>
                  <th className="px-4 py-3">Status</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((s) => (
                  <tr key={s.id_student} className="border-b border-[#1a1a1a] hover:bg-[#1a1a1a]">
                    <td className="px-4 py-3 text-[#EEEEEE]">{s.full_name}</td>
                    <td className="px-4 py-3 text-[#EEEEEE]">{s.instrument}</td>
                    <td className="px-4 py-3 text-[#EEEEEE]">{s.level}</td>
                    <td className="px-4 py-3">
                      <span className={`rounded-full px-2 py-1 text-xs font-medium ${badge[s.enrollment_status]}`}>
                        {s.enrollment_status}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
          </>
        )}
      </div>
    </div>
  );
}
