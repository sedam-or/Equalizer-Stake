"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

interface UserInfo {
  role: string;
  username: string;
}

export default function DashboardPage() {
  const router = useRouter();
  const [user, setUser] = useState<UserInfo | null>(null);

  useEffect(() => {
    const token = localStorage.getItem("token");
    const role = localStorage.getItem("role");
    if (!token) {
      router.push("/login");
      return;
    }
    setUser({ role: role || "admin", username: "Admin" });
  }, [router]);

  if (!user) return null;

  if (user.role === "owner") return <OwnerDashboard />;
  if (user.role === "teacher") return <TeacherDashboard />;
  if (user.role === "guardian") return <GuardianDashboard />;
  return <AdminDashboard />;
}

/* ─── AdminDashboard ─── */
function AdminDashboard() {
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = localStorage.getItem("token");
    fetch("/api/v1/reports/admin-stats", {
      headers: { Authorization: `Bearer ${token}` },
    })
      .then((r) => r.json())
      .then(setData)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="h-8 w-64 animate-pulse rounded-lg bg-gray-200" />
        <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="h-24 animate-pulse rounded-xl bg-gray-200" />
          ))}
        </div>
      </div>
    );
  }

  const kpi = data?.kpi || {};

  return (
    <div className="space-y-6">
      {/* Greeting */}
      <h1 className="font-heading text-2xl font-bold text-gray-900">
        {data?.greeting || "Selamat datang, Admin"}
      </h1>
      <p className="text-sm text-gray-500">{data?.date || ""}</p>

      {/* KPI Cards */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <KpiCard label="Sesi Hari Ini" value={String(kpi.sessions_today ?? 0)} />
        <KpiCard label="Absen Pending" value={String(kpi.pending_attendance ?? 0)} />
        <KpiCard label="Tagihan Overdue" value={String(kpi.overdue_invoices ?? 0)} />
        <KpiCard label="Notif Gagal" value={String(kpi.failed_notifications ?? 0)} />
      </div>

      {/* Recent Sessions */}
      {data?.recent_sessions?.length > 0 && (
        <div className="rounded-xl border border-gray-200 bg-white p-5">
          <h2 className="mb-3 font-heading text-lg font-bold text-gray-900">Sesi Hari Ini</h2>
          <div className="space-y-2">
            {data.recent_sessions.map((s: any) => (
              <div key={s.id_session} className="flex items-center justify-between rounded-lg border border-gray-100 p-3">
                <div>
                  <span className="font-medium text-gray-900">{s.start_time}</span>
                  <span className="ml-2 text-sm text-gray-500">{s.session_type}</span>
                </div>
                <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                  s.status === "completed" ? "bg-green-100 text-green-700" : "bg-gray-100 text-gray-600"
                }`}>
                  {s.status}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Action Buttons */}
      <div className="grid grid-cols-2 gap-3">
        <ActionButton href="/dashboard/scheduling" label="Buat Jadwal Baru" />
        <ActionButton href="/dashboard/attendance" label="Review Absensi" />
        <ActionButton href="/dashboard/billing" label="Verifikasi Pembayaran" />
      </div>
    </div>
  );
}

/* ─── OwnerDashboard ─── */
function OwnerDashboard() {
  const fallbackData: Record<string, unknown> = {
    billing_period: "Juni 2026",
    total_revenue: 85000000,
    revenue_this_month: 12500000,
    attendance_rate: 92,
    frozen_accounts: 2,
    studio_utilization: 78,
    total_students: 45,
    unpaid_invoices: 5,
    overdue_invoices: 1,
    verified_attendance: 380,
    total_attendance: 410,
    sessions_this_month: 95,
    total_rooms: 6,
    recent_audit: [
      { id_audit: "1", table_name: "payments", action: "INSERT", created_at: "2026-06-30T10:00:00Z" },
      { id_audit: "2", table_name: "attendance", action: "UPDATE", created_at: "2026-06-30T09:30:00Z" },
      { id_audit: "3", table_name: "students", action: "INSERT", created_at: "2026-06-29T14:00:00Z" },
    ],
  };

  const [data, setData] = useState<Record<string, unknown> | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) return;
    fetch("/api/reports/owner-dashboard", {
      headers: { Authorization: `Bearer ${token}` },
    })
      .then((res) => {
        if (!res.ok) throw new Error("Gagal memuat data");
        return res.json();
      })
      .then(setData)
      .catch((e) => {
        setError(e.message);
        setData(fallbackData);
      })
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div className="p-8 text-center text-gray-400">Memuat data...</div>;
  if (!data) return null;

  const fmtRp = (n: number) =>
    "Rp " + n.toLocaleString("id-ID", { maximumFractionDigits: 0 });

  return (
    <div>
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[#EEEEEE]">Dashboard Owner</h1>
          <p className="mt-1 text-sm text-gray-400">
            Monitoring keseluruhan sistem — Periode {String(data.billing_period)}
            {error && <span className="ml-2 text-yellow-500 text-xs">(data dummy)</span>}
          </p>
        </div>
        <button
          onClick={() => window.print()}
          className="rounded-lg border border-[#CBD5E1] bg-[#111] px-4 py-2 text-sm font-medium text-[#EEEEEE] hover:bg-[#1a1a1a] print:hidden"
        >
          🖨️ Cetak Laporan
        </button>
      </div>

      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <KpiCard label="Total Pendapatan" value={fmtRp(Number(data.total_revenue))} />
        <KpiCard label="Pendapatan Bulan Ini" value={fmtRp(Number(data.revenue_this_month))} />
        <KpiCard label="Tingkat Kehadiran" value={`${data.attendance_rate}%`} />
        <KpiCard label="Akun Beku" value={String(data.frozen_accounts)} />
      </div>

      <div className="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <KpiCard label="Studio Utilization" value={`${data.studio_utilization}%`} />
        <KpiCard label="Total Siswa Aktif" value={String(data.total_students)} />
        <KpiCard label="Tagihan Belum Bayar" value={String(data.unpaid_invoices)} />
        <KpiCard label="Tagihan Overdue" value={String(data.overdue_invoices)} />
      </div>

      {/* Detail section */}
      <div className="mt-8 grid grid-cols-1 gap-6 lg:grid-cols-2">
        <div className="rounded-xl border border-[#CBD5E1] bg-[#111] p-6">
          <h2 className="text-lg font-semibold text-[#EEEEEE]">Kehadiran</h2>
          <div className="mt-4 space-y-2 text-sm">
            <div className="flex justify-between"><span className="text-gray-400">Terverifikasi</span><span className="font-medium text-[#EEEEEE]">{String(data.verified_attendance)}</span></div>
            <div className="flex justify-between"><span className="text-gray-400">Total Absensi</span><span className="font-medium text-[#EEEEEE]">{String(data.total_attendance)}</span></div>
            <div className="flex justify-between"><span className="text-gray-400">Sesi Bulan Ini</span><span className="font-medium text-[#EEEEEE]">{String(data.sessions_this_month)}</span></div>
            <div className="flex justify-between"><span className="text-gray-400">Total Ruangan</span><span className="font-medium text-[#EEEEEE]">{String(data.total_rooms)}</span></div>
          </div>
        </div>

        <div className="rounded-xl border border-[#CBD5E1] bg-[#111] p-6">
          <h2 className="text-lg font-semibold text-[#EEEEEE]">Audit Log Terbaru</h2>
          <div className="mt-4 space-y-2">
            {Array.isArray(data.recent_audit) && data.recent_audit.length > 0 ? (
              (data.recent_audit as Array<Record<string, string>>).map((log) => (
                <div key={log.id_audit} className="flex items-center justify-between text-sm">
                  <span className="text-gray-400">{log.table_name} — {log.action}</span>
                  <span className="text-xs text-gray-500">{new Date(log.created_at).toLocaleDateString("id-ID")}</span>
                </div>
              ))
            ) : (
              <p className="text-sm text-gray-500">Belum ada aktivitas</p>
            )}
          </div>
        </div>
      </div>

      {/* Management links */}
      <div className="mt-8">
        <h2 className="text-lg font-semibold text-[#EEEEEE]">Manajemen</h2>
        <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-3">
          <ActionButton href="/dashboard/admins" label="Kelola Admin" />
          <ActionButton href="/dashboard/override" label="Override Operasional" />
          <ActionButton href="/dashboard/settings" label="Pengaturan Sistem" />
          <ActionButton href="/dashboard/pricing" label="Kelola Tarif" />
          <ActionButton href="/dashboard/audit" label="Audit Log" />
          <ActionButton href="/dashboard/reports" label="Laporan Detail" />
        </div>
      </div>

      {/* Print-only summary */}
      <div className="hidden print:block mt-8 border-t border-[#CBD5E1] pt-4 text-xs text-gray-500">
        <p>Dicetak: {new Date().toLocaleString("id-ID")} — BMD Music School Equalizer-Stake</p>
      </div>
    </div>
  );
}

/* ─── TeacherDashboard ─── */
function TeacherDashboard() {
  const dummySchedule = [
    { id: 1, jam: "09:00", siswa: "Andi Pratama", instrumen: "Gitar", ruang: "A1", status: "Selesai" },
    { id: 2, jam: "11:00", siswa: "Rina Wulandari", instrumen: "Piano", ruang: "B2", status: "Selesai" },
    { id: 3, jam: "14:30", siswa: "Budi Santoso", instrumen: "Drum", ruang: "C1", status: "Berlangsung" },
    { id: 4, jam: "15:30", siswa: "Dewi Lestari", instrumen: "Vokal", ruang: "A2", status: "Menunggu" },
    { id: 5, jam: "16:30", siswa: "Fajar Nugroho", instrumen: "Biola", ruang: "B1", status: "Menunggu" },
  ];

  return (
    <div>
      <h1 className="text-2xl font-bold text-[#EEEEEE]">Dashboard Guru</h1>
      <p className="mt-1 text-sm text-gray-400">Jadwal dan absensi hari ini</p>

      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2">
        <KpiCard label="Sesi Hari Ini" value="5" />
        <KpiCard label="Menunggu Verifikasi" value="2" />
      </div>

      <div className="mt-8">
        <h2 className="text-lg font-semibold text-[#EEEEEE]">Jadwal Hari Ini</h2>
        <div className="mt-4 overflow-x-auto rounded-xl border border-[#CBD5E1] bg-[#111]">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[#CBD5E1] text-left text-gray-400">
                <th className="px-4 py-3">Jam</th>
                <th className="px-4 py-3">Siswa</th>
                <th className="px-4 py-3">Instrumen</th>
                <th className="px-4 py-3">Ruang</th>
                <th className="px-4 py-3">Status</th>
              </tr>
            </thead>
            <tbody>
              {dummySchedule.map((s) => (
                <tr key={s.id} className="border-b border-[#1a1a1a] hover:bg-[#1a1a1a]">
                  <td className="px-4 py-3 text-[#2563EB] font-medium">{s.jam}</td>
                  <td className="px-4 py-3 text-[#EEEEEE]">{s.siswa}</td>
                  <td className="px-4 py-3 text-gray-400">{s.instrumen}</td>
                  <td className="px-4 py-3 text-gray-400">{s.ruang}</td>
                  <td className="px-4 py-3">
                    <span className={`rounded-full px-2 py-1 text-xs font-medium ${
                      s.status === "Selesai"
                        ? "bg-[#2563EB]/10 text-[#2563EB]"
                        : s.status === "Berlangsung"
                        ? "bg-blue-500/10 text-blue-400"
                        : "bg-yellow-500/10 text-yellow-400"
                    }`}>
                      {s.status}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

/* ─── GuardianDashboard ─── */
function GuardianDashboard() {
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = localStorage.getItem("token");
    fetch("/api/v1/reports/guardian-stats", {
      headers: { Authorization: `Bearer ${token}` },
    })
      .then((r) => r.json())
      .then(setData)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="h-8 w-64 animate-pulse rounded-lg bg-gray-200" />
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="h-32 animate-pulse rounded-xl bg-gray-200" />
          ))}
        </div>
      </div>
    );
  }

  const children = data?.children || [];

  return (
    <div className="space-y-6">
      <h1 className="font-heading text-2xl font-bold text-gray-900">
        {data?.greeting || "Selamat datang, Wali Murid"}
      </h1>
      <p className="text-sm text-gray-500">{data?.date || ""}</p>

      {/* Children cards */}
      {children.length > 0 ? (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {children.map((child: any) => (
            <div key={child.id_student} className="rounded-xl border border-gray-200 bg-white p-5">
              <div className="flex items-center gap-3 mb-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-[#2563EB]/10 text-[#2563EB]">
                  {child.instrument === "guitar" ? "🎸" : child.instrument === "piano" ? "🎹" : "🎵"}
                </div>
                <div>
                  <h3 className="font-heading font-bold text-gray-900">{child.full_name}</h3>
                  <p className="text-xs text-gray-500">{child.instrument} • {child.level}</p>
                </div>
              </div>

              {/* Upcoming sessions */}
              {child.upcoming_sessions?.length > 0 && (
                <div className="mb-3 space-y-1">
                  <p className="text-xs font-medium text-gray-400">Jadwal Mendatang</p>
                  {child.upcoming_sessions.slice(0, 2).map((s: any, i: number) => (
                    <div key={i} className="flex justify-between text-sm">
                      <span className="text-gray-600">{s.session_date} {s.start_time}</span>
                      <span className="text-[#2563EB]">{s.session_type}</span>
                    </div>
                  ))}
                </div>
              )}

              {/* Pending invoice */}
              {child.pending_invoice && (
                <div className="rounded-lg bg-amber-50 p-2 text-xs">
                  <span className="font-medium text-amber-700">Tagihan: Rp {child.pending_invoice.total_amount?.toLocaleString("id-ID")}</span>
                  <span className="ml-2 text-amber-600">Jatuh tempo: {child.pending_invoice.due_date}</span>
                </div>
              )}
            </div>
          ))}
        </div>
      ) : (
        <div className="rounded-xl border border-gray-200 bg-white p-8 text-center">
          <p className="text-gray-500">Belum ada data anak</p>
        </div>
      )}

      {/* Quick links */}
      <div className="grid grid-cols-2 gap-3">
        <ActionButton href="/dashboard/my-children" label="Anak Saya" />
        <ActionButton href="/dashboard/my-schedule" label="Jadwal" />
        <ActionButton href="/dashboard/my-payments" label="Tagihan" />
        <ActionButton href="/dashboard/profile" label="Profil" />
      </div>
    </div>
  );
}
/* ─── Shared Components ─── */

function KpiCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-xl border border-[#CBD5E1] bg-[#111] p-6">
      <div className="text-sm text-gray-400">{label}</div>
      <div className="mt-1 text-3xl font-bold text-[#2563EB]">{value}</div>
    </div>
  );
}

function ActionButton({ href, label }: { href: string; label: string }) {
  return (
    <a
      href={href}
      className="rounded-xl border border-[#CBD5E1] bg-[#111] p-4 text-center text-sm font-medium text-[#EEEEEE] transition-colors hover:border-[#2563EB] hover:bg-[#1a1a1a]"
    >
      {label}
    </a>
  );
}

function SessionCard({ time, student, instrument, status }: {
  time: string; student: string; instrument: string; status: string;
}) {
  return (
    <div className="flex items-center justify-between rounded-xl border border-[#CBD5E1] bg-[#111] p-4">
      <div className="flex items-center gap-3">
        <div className="text-lg font-bold text-[#2563EB]">{time}</div>
        <div>
          <p className="font-medium text-[#EEEEEE]">{student}</p>
          <p className="text-sm text-gray-400">{instrument}</p>
        </div>
      </div>
      <span
        className={`rounded-full px-2 py-1 text-xs font-medium ${
          status === "aktif"
            ? "bg-[#2563EB]/10 text-[#2563EB]"
            : "bg-yellow-500/10 text-yellow-400"
        }`}
      >
        {status === "aktif" ? "Aktif" : "Menunggu"}
      </span>
    </div>
  );
}

function StudentCard({ name, instrument, level, jadwal, status }: {
  name: string; instrument: string; level: string; jadwal: string; status: string;
}) {
  return (
    <div className="rounded-xl border border-[#CBD5E1] bg-[#111] p-6">
      <div className="flex items-center justify-between">
        <p className="font-semibold text-[#EEEEEE]">{name}</p>
        <span className={`rounded-full px-2 py-1 text-xs font-medium ${
          status === "Aktif"
            ? "bg-[#2563EB]/10 text-[#2563EB]"
            : "bg-red-500/10 text-red-400"
        }`}>
          {status}
        </span>
      </div>
      <div className="mt-3 space-y-1 text-sm">
        <p className="text-gray-400">🎹 {instrument}</p>
        <p className="text-gray-400">📊 Level: {level}</p>
        <p className="text-gray-400">📅 {jadwal}</p>
      </div>
    </div>
  );
}
