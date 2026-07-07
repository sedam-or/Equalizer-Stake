"use client";
import { api } from "@/lib/apiClient";
import type { AdminStats, OwnerDashboard, Session, SessionSummary, Student } from "@/types/api";
import { toast } from "@/components/Toast";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { StatCard, DashboardChart, StatusBadge, EmptyState, PageHeader } from "@/components/dashboard";

interface UserInfo {
  role: string;
  username: string;
}

export default function DashboardPage() {
  const router = useRouter();
  const [user, setUser] = useState<UserInfo | null>(null);

  useEffect(() => {
    const token = localStorage.getItem("access_token");
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
    const token = localStorage.getItem("access_token");
    fetch("/api/v1/reports/admin-stats", {
      headers: { Authorization: `Bearer ${token}` },
    })
      .then((r) => r.json())
      .then(setData)
      .catch(() => toast("error", "Gagal memuat data dashboard"))
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
      <PageHeader
        title={data?.greeting || "Selamat datang, Admin"}
        description={data?.date || ""}
      />

      {/* KPI Cards */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <StatCard
          label="Sesi Hari Ini"
          value={kpi.sessions_today ?? 0}
          icon="📅"
          gradient="from-[#2563EB] to-[#1E40AF]"
        />
        <StatCard
          label="Absen Pending"
          value={kpi.pending_attendance ?? 0}
          icon="⏳"
          gradient="from-[#F59E0B] to-[#D97706]"
        />
        <StatCard
          label="Tagihan Overdue"
          value={kpi.overdue_invoices ?? 0}
          icon="⚠️"
          gradient="from-[#EF4444] to-[#DC2626]"
        />
        <StatCard
          label="Notif Gagal"
          value={kpi.failed_notifications ?? 0}
          icon="🔔"
          gradient="from-[#8B5CF6] to-[#7C3AED]"
        />
      </div>

      {/* Charts Row */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2" style={{ height: "280px" }}>
        <DashboardChart
          type="bar"
          title="Sesi Minggu Ini"
          height={220}
          data={data?.weekly_sessions || [
            { name: "Sen", value: 12 },
            { name: "Sel", value: 15 },
            { name: "Rab", value: 18 },
            { name: "Kam", value: 14 },
            { name: "Jum", value: 20 },
            { name: "Sab", value: 16 },
          ]}
        />
        <DashboardChart
          type="donut"
          title="Status Pembayaran"
          height={220}
          data={((data?.unpaid_invoices || 0) + (data?.overdue_invoices || 0)) > 0 ? [{name:'Lunas',value:(data?.total_students||0) - (data?.unpaid_invoices||0) - (data?.overdue_invoices||0)},{name:'Belum Bayar',value:data?.unpaid_invoices||0},{name:'Terlambat',value:data?.overdue_invoices||0}] : [
            { name: "Lunas", value: 45 },
            { name: "Pending", value: 12 },
            { name: "Overdue", value: 8 },
          ]}
          colors={["#10B981", "#F59E0B", "#EF4444"]}
        />
      </div>

      {/* Recent Sessions */}
      {data?.recent_sessions?.length > 0 ? (
        <div className="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
          <h2 className="mb-4 font-heading text-lg font-bold text-[#0F172A]">Sesi Hari Ini</h2>
          <div className="space-y-3">
            {data.recent_sessions.map((s: any) => (
              <div key={s.id_session} className="flex items-center justify-between rounded-lg border border-gray-100 p-3 transition-colors hover:bg-gray-50">
                <div className="flex items-center gap-3">
                  <span className="font-heading text-sm font-bold text-[#0F172A]">{s.start_time}</span>
                  <span className="text-sm text-gray-500">{s.session_type}</span>
                </div>
                <StatusBadge status={s.status} />
              </div>
            ))}
          </div>
        </div>
      ) : (
        <EmptyState
          icon="📅"
          title="Belum Ada Sesi Hari Ini"
          description="Buat jadwal baru untuk memulai."
          action={{ label: "Buat Jadwal", href: "/dashboard/schedule" }}
        />
      )}

      {/* Quick Actions */}
      <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
        <a href="/dashboard/schedule" className="rounded-xl border border-gray-200 bg-white p-4 text-center shadow-sm transition-all hover:border-[#2563EB]/30 hover:shadow-md">
          <span className="text-2xl">📅</span>
          <div className="mt-2 text-sm font-semibold text-[#0F172A]">Buat Jadwal</div>
        </a>
        <a href="/dashboard/attendance" className="rounded-xl border border-gray-200 bg-white p-4 text-center shadow-sm transition-all hover:border-[#2563EB]/30 hover:shadow-md">
          <span className="text-2xl">✅</span>
          <div className="mt-2 text-sm font-semibold text-[#0F172A]">Review Absensi</div>
        </a>
        <a href="/dashboard/billing" className="rounded-xl border border-gray-200 bg-white p-4 text-center shadow-sm transition-all hover:border-[#2563EB]/30 hover:shadow-md">
          <span className="text-2xl">💰</span>
          <div className="mt-2 text-sm font-semibold text-[#0F172A]">Verifikasi Bayar</div>
        </a>
        <a href="/dashboard/students" className="rounded-xl border border-gray-200 bg-white p-4 text-center shadow-sm transition-all hover:border-[#2563EB]/30 hover:shadow-md">
          <span className="text-2xl">👥</span>
          <div className="mt-2 text-sm font-semibold text-[#0F172A]">Kelola Siswa</div>
        </a>
      </div>
    </div>
  );
}

/* ─── OwnerDashboard ─── */
function OwnerDashboard() {
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = localStorage.getItem("access_token");
    fetch("/api/v1/reports/owner-dashboard", {
      headers: { Authorization: `Bearer ${token}` },
    })
      .then((r) => r.json())
      .then(setData)
      .catch(() => toast("error", "Gagal memuat data owner"))
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

  return (
    <div className="space-y-6">
      <PageHeader
        title="Owner Dashboard"
        description="Overview bisnis BMD Music School"
      />

      {/* KPI Cards */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <StatCard
          label="Total Revenue"
          value={`Rp ${((data?.total_revenue || 0) / 1000000).toFixed(1)}jt`}
          icon="💰"
          gradient="from-[#10B981] to-[#059669]"
          trend={{ value: 12, isPositive: true }}
        />
        <StatCard
          label="Siswa Aktif"
          value={data?.total_students || 0}
          icon="👥"
          gradient="from-[#2563EB] to-[#1E40AF]"
          trend={{ value: 5, isPositive: true }}
        />
        <StatCard
          label="Attendance Rate"
          value={`${data?.attendance_rate || 0}%`}
          icon="📊"
          gradient="from-[#F59E0B] to-[#D97706]"
        />
        <StatCard
          label="Frozen Accounts"
          value={data?.frozen_accounts || 0}
          icon="❄️"
          gradient="from-[#64748B] to-[#475569]"
        />
      </div>

      {/* Charts — 2x2 grid */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <div style={{ height: "280px" }}>
          <DashboardChart
            type="bar"
            title="Revenue Bulanan"
            height={220}
            data={(data?.revenue_trend || []).map((r: any) => ({ name: r.period?.slice(5) || "", value: r.total_amount || 0 }))}
            colors={["#2563EB"]}
          />
        </div>
        <div style={{ height: "280px" }}>
          <DashboardChart
            type="donut"
            title="Status Pembayaran"
            height={220}
            data={[
              { name: "Lunas", value: (data?.total_students || 0) - (data?.unpaid_invoices || 0) - (data?.overdue_invoices || 0) },
              { name: "Belum Bayar", value: data?.unpaid_invoices || 0 },
              { name: "Terlambat", value: data?.overdue_invoices || 0 },
            ]}
            colors={["#10B981", "#F59E0B", "#EF4444"]}
          />
        </div>
        <div style={{ height: "280px" }}>
          <DashboardChart
            type="line"
            title="Pertumbuhan Siswa"
            height={220}
            data={data?.revenue_trend?.map((r: any) => ({ name: r.period?.slice(5) || "", value: r.invoice_count || 0 })) || []}
            colors={["#10B981"]}
          />
        </div>
        <div style={{ height: "280px" }}>
          <DashboardChart
            type="pie"
            title="Utilisasi Studio"
            height={220}
            data={(data?.studio_breakdown || []).map((s: any) => ({ name: s.room_name, value: s.sessions || 0 }))}
          />
        </div>
      </div>
    </div>
  );
}

/* ─── TeacherDashboard ─── */
function TeacherDashboard() {
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = localStorage.getItem("access_token");
    fetch("/api/v1/my/schedule", {
      headers: { Authorization: `Bearer ${token}` },
    })
      .then((r) => r.json())
      .then(setData)
      .catch(() => toast("error", "Gagal memuat jadwal"))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="h-8 w-64 animate-pulse rounded-lg bg-gray-200" />
        <div className="grid grid-cols-2 gap-4">
          {[1, 2].map((i) => (
            <div key={i} className="h-24 animate-pulse rounded-xl bg-gray-200" />
          ))}
        </div>
      </div>
    );
  }

  const sessions = Array.isArray(data) ? data : data?.data || [];

  return (
    <div className="space-y-6">
      <PageHeader
        title="Jadwal Mengajar"
        description="Sesi hari ini dan mendatang"
      />

      {/* KPI */}
      <div className="grid grid-cols-2 gap-4">
        <StatCard
          label="Sesi Hari Ini"
          value={sessions.length}
          icon="📅"
          gradient="from-[#2563EB] to-[#1E40AF]"
        />
        <StatCard
          label="Total Siswa"
          value={sessions.reduce((acc: number, s: any) => acc + (s.student_count || 1), 0)}
          icon="👥"
          gradient="from-[#10B981] to-[#059669]"
        />
      </div>

      {/* Sessions List */}
      {sessions.length > 0 ? (
        <div className="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
          <h2 className="mb-4 font-heading text-lg font-bold text-[#0F172A]">Jadwal Hari Ini</h2>
          <div className="space-y-3">
            {sessions.map((s: any) => (
              <div key={s.id_session} className="flex items-center justify-between rounded-lg border border-gray-100 p-3 transition-colors hover:bg-gray-50">
                <div className="flex items-center gap-3">
                  <span className="font-heading text-sm font-bold text-[#0F172A]">{s.start_time}</span>
                  <span className="text-sm text-gray-500">{s.student_name}</span>
                </div>
                <StatusBadge status={s.status || "scheduled"} />
              </div>
            ))}
          </div>
        </div>
      ) : (
        <EmptyState
          icon="📅"
          title="Tidak Ada Jadwal Hari Ini"
          description="Nikmati hari libur Anda."
        />
      )}
    </div>
  );
}

/* ─── GuardianDashboard ─── */
function GuardianDashboard() {
  const [children, setChildren] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = localStorage.getItem("access_token");
    fetch("/api/v1/my/children", {
      headers: { Authorization: `Bearer ${token}` },
    })
      .then((r) => r.json())
      .then((d) => setChildren(d?.data || []))
      .catch(() => toast("error", "Gagal memuat data anak"))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="h-8 w-64 animate-pulse rounded-lg bg-gray-200" />
        <div className="grid grid-cols-2 gap-4">
          {[1, 2].map((i) => (
            <div key={i} className="h-24 animate-pulse rounded-xl bg-gray-200" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Anak Saya"
        description="Lihat jadwal dan perkembangan anak Anda"
      />

      {/* Children Cards */}
      {children.length > 0 ? (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {children.map((child: any) => (
            <a
              key={child.id_student}
              href={`/dashboard/my-schedule?student_id=${child.id_student}`}
              className="group rounded-xl border border-gray-200 bg-white p-5 shadow-sm transition-all hover:border-[#2563EB]/30 hover:shadow-md"
            >
              <div className="flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-[#2563EB]/10 text-lg">
                  🎵
                </div>
                <div>
                  <div className="font-heading text-sm font-bold text-[#0F172A]">{child.full_name}</div>
                  <div className="text-xs text-gray-500">{child.instrument} — {child.level}</div>
                </div>
              </div>
              <div className="mt-3 flex items-center justify-between">
                <StatusBadge status={child.enrollment_status || "active"} />
                <span className="text-xs text-gray-400 group-hover:text-[#2563EB]">Lihat →</span>
              </div>
            </a>
          ))}
        </div>
      ) : (
        <EmptyState
          icon="👶"
          title="Belum Ada Anak Terdaftar"
          description="Daftarkan anak Anda untuk memulai kursus musik."
          action={{ label: "Daftar Kursus", href: "/register-course" }}
        />
      )}

      {/* Quick Actions */}
      <div className="grid grid-cols-2 gap-3">
        <a href="/dashboard/my-schedule" className="rounded-xl border border-gray-200 bg-white p-4 text-center shadow-sm transition-all hover:border-[#2563EB]/30 hover:shadow-md">
          <span className="text-2xl">📅</span>
          <div className="mt-2 text-sm font-semibold text-[#0F172A]">Jadwal</div>
        </a>
        <a href="/dashboard/my-payments" className="rounded-xl border border-gray-200 bg-white p-4 text-center shadow-sm transition-all hover:border-[#2563EB]/30 hover:shadow-md">
          <span className="text-2xl">💰</span>
          <div className="mt-2 text-sm font-semibold text-[#0F172A]">Pembayaran</div>
        </a>
      </div>
    </div>
  );
}
