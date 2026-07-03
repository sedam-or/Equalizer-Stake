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
  const dummySessions = [
    { id: 1, jam: "09:00", siswa: "Andi Pratama", guru: "Pak Dimas", instrumen: "Gitar", ruang: "A1", status: "Selesai" },
    { id: 2, jam: "10:00", siswa: "Rina Wulandari", guru: "Mbak Sari", instrumen: "Piano", ruang: "B2", status: "Selesai" },
    { id: 3, jam: "13:00", siswa: "Budi Santoso", guru: "Pak Dimas", instrumen: "Drum", ruang: "C1", status: "Berlangsung" },
    { id: 4, jam: "14:30", siswa: "Dewi Lestari", guru: "Mbak Sari", instrumen: "Vokal", ruang: "A2", status: "Menunggu" },
    { id: 5, jam: "16:00", siswa: "Fajar Nugroho", guru: "Pak Rizal", instrumen: "Biola", ruang: "B1", status: "Menunggu" },
  ];

  return (
    <div>
      <h1 className="text-2xl font-bold text-[#EEEEEE]">Dashboard Admin</h1>
      <p className="mt-1 text-sm text-gray-400">Selamat datang di BMD Music School</p>

      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <KpiCard label="Sesi Hari Ini" value="12" />
        <KpiCard label="Pending Verifikasi" value="3" />
        <KpiCard label="Tagihan Belum Bayar" value="5" />
        <KpiCard label="Notifikasi Gagal" value="0" />
      </div>

      {/* Recent sessions table */}
      <div className="mt-8">
        <h2 className="text-lg font-semibold text-[#EEEEEE]">Sesi Terbaru</h2>
        <div className="mt-4 overflow-x-auto rounded-xl border border-[#CBD5E1] bg-[#111]">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[#CBD5E1] text-left text-gray-400">
                <th className="px-4 py-3">Jam</th>
                <th className="px-4 py-3">Siswa</th>
                <th className="px-4 py-3">Guru</th>
                <th className="px-4 py-3">Instrumen</th>
                <th className="px-4 py-3">Ruang</th>
                <th className="px-4 py-3">Status</th>
              </tr>
            </thead>
            <tbody>
              {dummySessions.map((s) => (
                <tr key={s.id} className="border-b border-[#1a1a1a] hover:bg-[#1a1a1a]">
                  <td className="px-4 py-3 text-[#2563EB] font-medium">{s.jam}</td>
                  <td className="px-4 py-3 text-[#EEEEEE]">{s.siswa}</td>
                  <td className="px-4 py-3 text-gray-400">{s.guru}</td>
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

      {/* Action buttons */}
      <div className="mt-8">
        <h2 className="text-lg font-semibold text-[#EEEEEE]">Aksi Cepat</h2>
        <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-3">
          <ActionButton href="/dashboard/schedule" label="Buat Jadwal Baru" />
          <ActionButton href="/dashboard/attendance" label="Review Absensi" />
          <ActionButton href="/dashboard/billing" label="Verifikasi Pembayaran" />
        </div>
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
  const children = [
    { id: 1, nama: "Andi Pratama", instrumen: "Gitar", level: "Pemula", jadwal: "Senin & Rabu 14:00", status: "Aktif" },
    { id: 2, nama: "Siti Rahma", instrumen: "Piano", level: "Menengah", jadwal: "Selasa & Kamis 15:00", status: "Aktif" },
    { id: 3, nama: "Rizky Aditya", instrumen: "Drum", level: "Pemula", jadwal: "Jumat 16:00", status: "Beku" },
  ];

  const schedule = [
    { id: 1, jam: "14:00", anak: "Andi Pratama", instrumen: "Gitar", guru: "Pak Dimas", status: "Berlangsung" },
    { id: 2, jam: "15:00", anak: "Siti Rahma", instrumen: "Piano", guru: "Mbak Sari", status: "Menunggu" },
    { id: 3, jam: "16:00", anak: "Rizky Aditya", instrumen: "Drum", guru: "Pak Dimas", status: "Dibatalkan" },
  ];

  return (
    <div>
      <h1 className="text-2xl font-bold text-[#EEEEEE]">Dashboard Wali Murid</h1>
      <p className="mt-1 text-sm text-gray-400">Monitoring anak-anak Anda</p>

      {/* Children cards */}
      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {children.map((c) => (
          <StudentCard key={c.id} name={c.nama} instrument={c.instrumen} level={c.level} jadwal={c.jadwal} status={c.status} />
        ))}
      </div>

      {/* Schedule table */}
      <div className="mt-8">
        <h2 className="text-lg font-semibold text-[#EEEEEE]">Jadwal Hari Ini</h2>
        <div className="mt-4 overflow-x-auto rounded-xl border border-[#CBD5E1] bg-[#111]">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[#CBD5E1] text-left text-gray-400">
                <th className="px-4 py-3">Jam</th>
                <th className="px-4 py-3">Anak</th>
                <th className="px-4 py-3">Instrumen</th>
                <th className="px-4 py-3">Guru</th>
                <th className="px-4 py-3">Status</th>
              </tr>
            </thead>
            <tbody>
              {schedule.map((s) => (
                <tr key={s.id} className="border-b border-[#1a1a1a] hover:bg-[#1a1a1a]">
                  <td className="px-4 py-3 text-[#2563EB] font-medium">{s.jam}</td>
                  <td className="px-4 py-3 text-[#EEEEEE]">{s.anak}</td>
                  <td className="px-4 py-3 text-gray-400">{s.instrumen}</td>
                  <td className="px-4 py-3 text-gray-400">{s.guru}</td>
                  <td className="px-4 py-3">
                    <span className={`rounded-full px-2 py-1 text-xs font-medium ${
                      s.status === "Berlangsung"
                        ? "bg-blue-500/10 text-blue-400"
                        : s.status === "Menunggu"
                        ? "bg-yellow-500/10 text-yellow-400"
                        : "bg-red-500/10 text-red-400"
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

      {/* Quick links */}
      <div className="mt-8">
        <h2 className="text-lg font-semibold text-[#EEEEEE]">Menu Cepat</h2>
        <div className="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
          <ActionButton href="/dashboard/my-children" label="👨‍🎓 Anak Saya" />
          <ActionButton href="/dashboard/my-schedule" label="📅 Jadwal" />
          <ActionButton href="/dashboard/my-payments" label="💰 Tagihan" />
          <ActionButton href="/dashboard/profile" label="👤 Profil" />
        </div>
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
