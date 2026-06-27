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
  return (
    <div>
      <h1 className="text-2xl font-bold text-[#000000]">Dashboard Owner</h1>
      <p className="mt-1 text-sm text-gray-500">Monitoring keseluruhan sistem</p>

      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <KpiCard label="Total Pendapatan" value="Rp 12.5Jt" color="text-[#08CB00]" />
        <KpiCard label="Tingkat Kehadiran" value="94%" color="text-blue-500" />
        <KpiCard label="Akun Beku" value="2" color="text-red-500" />
        <KpiCard label="Studio Utilization" value="78%" color="text-purple-500" />
      </div>

      <div className="mt-8">
        <h2 className="text-lg font-semibold text-[#000000]">Manajemen</h2>
        <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-3">
          <ActionButton href="/dashboard/admins" label="Kelola Admin" />
          <ActionButton href="/dashboard/audit" label="Audit Log" />
          <ActionButton href="/dashboard/reports" label="Laporan" />
        </div>
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
