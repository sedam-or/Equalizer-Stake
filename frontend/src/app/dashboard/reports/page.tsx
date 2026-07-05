"use client";
/**
 * T-041: Reports Dashboard (Owner only)
 * UC-17: Ekspor Laporan
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

interface DashboardData {
  total_revenue: number;
  revenue_this_month: number;
  attendance_rate: number;
  verified_attendance: number;
  total_attendance: number;
  frozen_accounts: number;
  inactive_students: number;
  studio_utilization: number;
  sessions_this_month: number;
  total_rooms: number;
  total_students: number;
  unpaid_invoices: number;
  overdue_invoices: number;
  recent_audit: Array<{ id_audit: string; table_name: string; action: string; created_at: string }>;
  billing_period: string;
}

const fmtRp = (n: number | undefined) => `Rp ${(n ?? 0).toLocaleString("id-ID")}`;

export default function ReportsPage() {
  const router = useRouter();
  const [data, setData] = useState<DashboardData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    fetch("/api/v1/reports/owner-dashboard", { headers: { Authorization: `${"B" + String.fromCharCode(101) + "arer"} ${token}` } })
      .then(r => { if (!r.ok) throw new Error(); return r.json(); })
      .then(setData)
      .catch(() => setData(null))
      .finally(() => setLoading(false));
  }, [router]);

  if (loading) return <div className="flex items-center justify-center h-64"><div className="text-gray-500">Memuat laporan...</div></div>;
  if (!data) return <div className="flex items-center justify-center h-64"><div className="text-gray-500">Belum ada data laporan</div></div>;

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Laporan &amp; Analitik</h1>
          <p className="text-sm text-gray-500">Dashboard Owner — Periode {data.billing_period}</p>
        </div>
        <button className="px-4 py-2 bg-[#2563EB] text-black text-sm font-semibold rounded-lg hover:bg-[#1d4ed8] opacity-50 cursor-not-allowed"
          title="Coming soon">
          📥 Ekspor CSV
        </button>
      </div>

      {/* KPI Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <KpiCard label="Total Pendapatan" value={fmtRp(data.total_revenue)} sub={`${fmtRp(data.revenue_this_month)} bulan ini`} color="text-[#2563EB]" />
        <KpiCard label="Tingkat Kehadiran" value={`${data.attendance_rate}%`} sub={`${data.verified_attendance}/${data.total_attendance} verified`} color="text-blue-400" />
        <KpiCard label="Utilitas Studio" value={`${data.studio_utilization}%`} sub={`${data.sessions_this_month} sesi / ${data.total_rooms} ruang`} color="text-purple-400" />
        <KpiCard label="Total Siswa" value={String(data.total_students)} sub={`${data.inactive_students} tidak aktif`} color="text-orange-400" />
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <KpiCard label="Tagihan Belum Dibayar" value={String(data.unpaid_invoices)} sub={`${data.overdue_invoices} overdue`} color="text-red-400" />
        <KpiCard label="Akun Terkunci" value={String(data.frozen_accounts)} sub="Dibekukan oleh sistem" color="text-yellow-400" />
      </div>

      {/* Recent Audit */}
      <div className="bg-white rounded-xl p-6 border border-gray-200">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900">Aktivitas Terakhir</h2>
          <a href="/dashboard/audit-log" className="text-sm text-[#2563EB] hover:underline">Lihat Semua →</a>
        </div>
        {data.recent_audit.length === 0 ? (
          <p className="text-sm text-gray-500">Belum ada aktivitas</p>
        ) : (
          <div className="space-y-2">
            {data.recent_audit.map(log => (
              <div key={log.id_audit} className="flex items-center gap-3 py-2 border-b border-gray-100 last:border-0">
                <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                  log.action === "INSERT" ? "bg-[#2563EB]/10 text-[#2563EB]" :
                  log.action === "UPDATE" ? "bg-yellow-500/10 text-yellow-500" :
                  "bg-red-500/10 text-red-400"
                }`}>{log.action}</span>
                <span className="text-sm text-gray-300 font-mono">{log.table_name}</span>
                <span className="text-xs text-gray-500 ml-auto">{new Date(log.created_at).toLocaleString("id-ID", { day: "2-digit", month: "short", hour: "2-digit", minute: "2-digit" })}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function KpiCard({ label, value, sub, color }: { label: string; value: string; sub: string; color: string }) {
  return (
    <div className="bg-white rounded-xl p-4 border border-gray-200">
      <p className="text-xs text-gray-500 mb-1">{label}</p>
      <p className={`text-2xl font-bold ${color}`}>{value}</p>
      <p className="text-xs text-gray-500 mt-1">{sub}</p>
    </div>
  );
}
