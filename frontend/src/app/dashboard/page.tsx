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

  // Role-based dashboard content
  if (user.role === "owner") {
    return <OwnerDashboard />;
  }

  if (user.role === "teacher") {
    return <TeacherDashboard />;
  }

  if (user.role === "guardian") {
    return <GuardianDashboard />;
  }

  return <AdminDashboard />;
}

function AdminDashboard() {
  return (
    <div>
      <h1 className="text-2xl font-bold text-[#000000]">Dashboard Admin</h1>
      <p className="mt-1 text-sm text-gray-500">Selamat datang di Equalizer-Stake</p>

      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <KpiCard label="Sesi Hari Ini" value="12" color="text-[#08CB00]" />
        <KpiCard label="Pending Verifikasi" value="3" color="text-yellow-500" />
        <KpiCard label="Tagihan Belum Bayar" value="5" color="text-red-500" />
        <KpiCard label="Notifikasi Gagal" value="0" color="text-gray-400" />
      </div>

      <div className="mt-8">
        <h2 className="text-lg font-semibold text-[#000000]">Aksi Cepat</h2>
        <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-3">
          <ActionButton href="/dashboard/schedule" label="Buat Jadwal Baru" />
          <ActionButton href="/dashboard/attendance" label="Review Absensi" />
          <ActionButton href="/dashboard/billing" label="Verifikasi Pembayaran" />
        </div>
      </div>
    </div>
  );
}

function OwnerDashboard() {
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
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div className="p-8 text-center text-gray-500">Memuat data...</div>;
  if (error) return <div className="p-8 text-center text-red-500">{error}</div>;
  if (!data) return null;

  const fmtRp = (n: number) =>
    "Rp " + n.toLocaleString("id-ID", { maximumFractionDigits: 0 });

  return (
    <div>
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[#000000]">Dashboard Owner</h1>
          <p className="mt-1 text-sm text-gray-500">
            Monitoring keseluruhan sistem — Periode {String(data.billing_period)}
          </p>
        </div>
        <button
          onClick={() => window.print()}
          className="rounded-lg border border-[#253900] bg-white px-4 py-2 text-sm font-medium text-[#000000] hover:bg-gray-50 print:hidden"
        >
          🖨️ Cetak Laporan
        </button>
      </div>

      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <KpiCard label="Total Pendapatan" value={fmtRp(Number(data.total_revenue))} color="text-[#08CB00]" />
        <KpiCard label="Pendapatan Bulan Ini" value={fmtRp(Number(data.revenue_this_month))} color="text-blue-500" />
        <KpiCard label="Tingkat Kehadiran" value={`${data.attendance_rate}%`} color="text-purple-500" />
        <KpiCard label="Akun Beku" value={String(data.frozen_accounts)} color="text-red-500" />
      </div>

      <div className="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <KpiCard label="Studio Utilization" value={`${data.studio_utilization}%`} color="text-indigo-500" />
        <KpiCard label="Total Siswa Aktif" value={String(data.total_students)} color="text-[#08CB00]" />
        <KpiCard label="Tagihan Belum Bayar" value={String(data.unpaid_invoices)} color="text-yellow-500" />
        <KpiCard label="Tagihan Overdue" value={String(data.overdue_invoices)} color="text-red-500" />
      </div>

      {/* Detail section */}
      <div className="mt-8 grid grid-cols-1 gap-6 lg:grid-cols-2">
        {/* Attendance detail */}
        <div className="rounded-xl border border-gray-200 bg-white p-6">
          <h2 className="text-lg font-semibold text-[#000000]">Kehadiran</h2>
          <div className="mt-4 space-y-2 text-sm">
            <div className="flex justify-between"><span className="text-gray-500">Terverifikasi</span><span className="font-medium">{String(data.verified_attendance)}</span></div>
            <div className="flex justify-between"><span className="text-gray-500">Total Absensi</span><span className="font-medium">{String(data.total_attendance)}</span></div>
            <div className="flex justify-between"><span className="text-gray-500">Sesi Bulan Ini</span><span className="font-medium">{String(data.sessions_this_month)}</span></div>
            <div className="flex justify-between"><span className="text-gray-500">Total Ruangan</span><span className="font-medium">{String(data.total_rooms)}</span></div>
          </div>
        </div>

        {/* Recent audit */}
        <div className="rounded-xl border border-gray-200 bg-white p-6">
          <h2 className="text-lg font-semibold text-[#000000]">Audit Log Terbaru</h2>
          <div className="mt-4 space-y-2">
            {Array.isArray(data.recent_audit) && data.recent_audit.length > 0 ? (
              (data.recent_audit as Array<Record<string, string>>).map((log) => (
                <div key={log.id_audit} className="flex items-center justify-between text-sm">
                  <span className="text-gray-700">{log.table_name} — {log.action}</span>
                  <span className="text-xs text-gray-400">{new Date(log.created_at).toLocaleDateString("id-ID")}</span>
                </div>
              ))
            ) : (
              <p className="text-sm text-gray-400">Belum ada aktivitas</p>
            )}
          </div>
        </div>
      </div>

      {/* Management links */}
      <div className="mt-8">
        <h2 className="text-lg font-semibold text-[#000000]">Manajemen</h2>
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
      <div className="hidden print:block mt-8 border-t pt-4 text-xs text-gray-500">
        <p>Dicetak: {new Date().toLocaleString("id-ID")} — BMD Music School Equalizer-Stake</p>
      </div>
    </div>
  );
}

function TeacherDashboard() {
  return (
    <div>
      <h1 className="text-2xl font-bold text-[#000000]">Dashboard Guru</h1>
      <p className="mt-1 text-sm text-gray-500">Jadwal dan absensi hari ini</p>

      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2">
        <KpiCard label="Sesi Hari Ini" value="4" color="text-[#08CB00]" />
        <KpiCard label="Menunggu Verifikasi" value="1" color="text-yellow-500" />
      </div>

      <div className="mt-8">
        <h2 className="text-lg font-semibold text-[#000000]">Jadwal Hari Ini</h2>
        <div className="mt-4 space-y-3">
          <SessionCard time="14:30" student="Andi Pratama" instrument="Gitar" status="aktif" />
          <SessionCard time="15:30" student="Rina Wulandari" instrument="Piano" status="menunggu" />
          <SessionCard time="16:30" student="Budi Santoso" instrument="Drum" status="menunggu" />
        </div>
      </div>
    </div>
  );
}

function GuardianDashboard() {
  return (
    <div>
      <h1 className="text-2xl font-bold text-[#000000]">Dashboard Wali Murid</h1>
      <p className="mt-1 text-sm text-gray-500">Monitoring anak-anak Anda</p>

      <div className="mt-6">
        <h2 className="text-lg font-semibold text-[#000000]">Anak Anda</h2>
        <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2">
          <StudentCard name="Andi Pratama" instrument="Gitar" status="aktif" />
          <StudentCard name="Rina Wulandari" instrument="Piano" status="aktif" />
        </div>
      </div>

      <div className="mt-8">
        <h2 className="text-lg font-semibold text-[#000000]">Tagihan Terbaru</h2>
        <div className="mt-4 rounded-lg border border-[#253900] bg-[#fafafa] p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium text-[#000000]">Januari 2026</p>
              <p className="text-sm text-gray-500">Jatuh tempo: 10 Januari</p>
            </div>
            <div className="text-right">
              <p className="font-bold text-[#08CB00]">Rp 850.000</p>
              <p className="text-xs text-yellow-600">Kode: 127</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Shared components
function KpiCard({ label, value, color }: { label: string; value: string; color: string }) {
  return (
    <div className="rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
      <div className="text-sm text-gray-500">{label}</div>
      <div className={`mt-1 text-3xl font-bold ${color}`}>{value}</div>
    </div>
  );
}

function ActionButton({ href, label }: { href: string; label: string }) {
  return (
    <a
      href={href}
      className="rounded-lg border border-[#253900] bg-[#fafafa] p-4 text-center text-sm font-medium text-[#000000] transition-colors hover:border-[#08CB00] hover:bg-[#f0fff0]"
    >
      {label}
    </a>
  );
}

function SessionCard({ time, student, instrument, status }: {
  time: string; student: string; instrument: string; status: string;
}) {
  return (
    <div className="flex items-center justify-between rounded-lg border border-gray-200 bg-white p-4">
      <div className="flex items-center gap-3">
        <div className="text-lg font-bold text-[#08CB00]">{time}</div>
        <div>
          <p className="font-medium text-[#000000]">{student}</p>
          <p className="text-sm text-gray-500">{instrument}</p>
        </div>
      </div>
      <span
        className={`rounded-full px-2 py-1 text-xs font-medium ${
          status === "aktif"
            ? "bg-[#08CB00]/10 text-[#08CB00]"
            : "bg-yellow-100 text-yellow-700"
        }`}
      >
        {status === "aktif" ? "Aktif" : "Menunggu"}
      </span>
    </div>
  );
}

function StudentCard({ name, instrument, status }: {
  name: string; instrument: string; status: string;
}) {
  return (
    <div className="rounded-lg border border-gray-200 bg-white p-4">
      <div className="flex items-center justify-between">
        <div>
          <p className="font-medium text-[#000000]">{name}</p>
          <p className="text-sm text-gray-500">{instrument}</p>
        </div>
        <span className="rounded-full bg-[#08CB00]/10 px-2 py-1 text-xs font-medium text-[#08CB00]">
          {status === "aktif" ? "Aktif" : "Beku"}
        </span>
      </div>
    </div>
  );
}
