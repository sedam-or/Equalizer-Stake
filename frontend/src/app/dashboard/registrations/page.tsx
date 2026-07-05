"use client";
import { SkeletonTable } from "@/components/Skeleton";
import { useDashboard } from "@/components/DashboardContext";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface Registration {
  id: string;
  full_name: string;
  phone_whatsapp: string;
  email: string;
  address: string;
  instrument: string;
  level: string;
  class_type: string;
  status: "menunggu_konfirmasi" | "disetujui" | "ditolak";
  created_at: string;
}

const INSTRUMENT_LABELS: Record<string, string> = {
  piano_klasik: "Piano Klasik",
  piano_pop: "Piano Pop",
  vokal: "Vokal",
  biola: "Biola",
  gitar: "Gitar",
  keyboard: "Keyboard",
  drum: "Drum",
  flute: "Flute",
  saxophone: "Saxophone",
};

const LEVEL_LABELS: Record<string, string> = {
  pra_elementary: "Pra-Elementary",
  elementary: "Elementary",
  grade_1: "Grade 1",
  grade_2: "Grade 2",
  grade_3: "Grade 3",
  grade_4: "Grade 4",
  grade_5: "Grade 5",
  grade_6: "Grade 6",
  beginner: "Beginner",
  intermediate: "Intermediate",
  intermediate_2: "Intermediate 2",
  basic: "Basic",
  advance: "Advance",
};

const CLASS_TYPE_LABELS: Record<string, string> = {
  privat_30: "Privat 30 Menit",
  privat_60: "Privat 60 Menit",
  grup_2_30: "Grup 2 Orang 30 Menit",
  grup_2_60: "Grup 2 Orang 60 Menit",
  grup_3_30: "Grup 3 Orang 30 Menit",
  grup_3_60: "Grup 3 Orang 60 Menit",
  grup_4_30: "Grup 4 Orang 30 Menit",
  grup_4_60: "Grup 4 Orang 60 Menit",
  grup_5_plus_60: "Grup 5+ Orang 60 Menit",
};

const STATUS_STYLES: Record<string, string> = {
  menunggu_konfirmasi: "bg-yellow-500/10 text-yellow-400",
  disetujui: "bg-[#2563EB]/10 text-[#2563EB]",
  ditolak: "bg-red-500/10 text-red-400",
};

const STATUS_LABELS: Record<string, string> = {
  menunggu_konfirmasi: "Menunggu",
  disetujui: "Disetujui",
  ditolak: "Ditolak",
};

const DAYS = ["Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu", "Minggu"];

export default function RegistrationsPage() {
  const router = useRouter();
  const [registrations, setRegistrations] = useState<Registration[]>([]);
  const [loading, setLoading] = useState(true);
  const { refreshKey, triggerRefresh } = useDashboard();
  const [selected, setSelected] = useState<Registration | null>(null);
  const [teacher, setTeacher] = useState("");
  const [scheduleDay, setScheduleDay] = useState("");
  const [scheduleTime, setScheduleTime] = useState("");
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  const getToken = () => localStorage.getItem("access_token");

  useEffect(() => {
    const fetchRegistrations = async () => {
      const token = getToken();
      if (!token) { router.push("/login"); return; }
      try {
        const res = await fetch(`/api/registrations/?limit=${limit}&offset=${offset}`, {
          headers: { Authorization: `Bearer ${token}` },
        });
        if (!res.ok) throw new Error("Gagal memuat pendaftaran");
        const data = await res.json();
        setRegistrations(data.data || data.registrations || []);
        setTotal(data.total || 0);
      } catch {
        setRegistrations([]);
      } finally {
        setLoading(false);
      }
    };
    fetchRegistrations();
  }, [router, offset]);

  const handleApprove = async () => {
    if (!selected) return;
    const token = getToken();
    try {
      const res = await fetch(`/api/registrations/${selected.id}/approve`, {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify({ teacher_id: teacher, schedule_day: scheduleDay, schedule_time: scheduleTime }),
      });
      if (!res.ok) throw new Error("Gagal menyetujui");
      setRegistrations(prev => prev.map(r => r.id === selected.id ? { ...r, status: "disetujui" } : r));
      setSelected(null);
      setTeacher("");
      setScheduleDay("");
      setScheduleTime("");
    } catch {
      // ponytail: no toast, add when toast system exists
    }
  };

  const handleReject = async () => {
    if (!selected) return;
    const token = getToken();
    try {
      const res = await fetch(`/api/registrations/${selected.id}/reject`, {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
      });
      if (!res.ok) throw new Error("Gagal menolak");
      setRegistrations(prev => prev.map(r => r.id === selected.id ? { ...r, status: "ditolak" } : r));
      setSelected(null);
    } catch {
      // ponytail: no toast
    }
  };

  const formatDate = (iso: string) =>
    new Date(iso).toLocaleDateString("id-ID", {
      day: "numeric",
      month: "short",
      year: "numeric",
    });

  const inputClass =
    "w-full rounded-lg border border-gray-200 bg-[#F8FAFC] px-3 py-2 text-sm text-gray-900 outline-none focus:border-[#2563EB]";

  if (loading) return <SkeletonTable rows={5} />;

  return (
    <div>
      <h1 className="mb-1 text-2xl font-bold text-gray-900">
        Pendaftaran Kursus
      </h1>
      <p className="mb-6 text-sm text-gray-500">
        Review dan kelola pendaftaran kursus baru
      </p>

      {/* Table */}
      <div className="overflow-x-auto rounded-xl border border-gray-200 bg-white">
        {registrations.length === 0 ? (
          <div className="p-8 text-center text-gray-500">Tidak ada pendaftaran</div>
        ) : (
          <>
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-gray-200 text-xs uppercase tracking-wider text-gray-500">
                <th className="px-4 py-3">Nama</th>
                <th className="px-4 py-3">WhatsApp</th>
                <th className="hidden px-4 py-3 md:table-cell">Instrumen</th>
                <th className="hidden px-4 py-3 md:table-cell">Level</th>
                <th className="hidden px-4 py-3 lg:table-cell">Tipe Kelas</th>
                <th className="px-4 py-3">Status</th>
                <th className="hidden px-4 py-3 sm:table-cell">Tanggal</th>
                <th className="px-4 py-3">Aksi</th>
              </tr>
            </thead>
            <tbody>
              {registrations.map((r) => (
                <tr
                  key={r.id}
                  className="border-b border-gray-200/50 transition-colors hover:bg-white/[0.02]"
                >
                  <td className="px-4 py-3 font-medium text-gray-900">
                    {r.full_name}
                  </td>
                  <td className="px-4 py-3 text-gray-500">{r.phone_whatsapp}</td>
                  <td className="hidden px-4 py-3 text-gray-500 md:table-cell">
                    {INSTRUMENT_LABELS[r.instrument] ?? r.instrument}
                  </td>
                  <td className="hidden px-4 py-3 text-gray-500 md:table-cell">
                    {LEVEL_LABELS[r.level] ?? r.level}
                  </td>
                  <td className="hidden px-4 py-3 text-gray-500 lg:table-cell">
                    {CLASS_TYPE_LABELS[r.class_type] ?? r.class_type}
                  </td>
                  <td className="px-4 py-3">
                    <span
                      className={`inline-block rounded-full px-2.5 py-0.5 text-xs font-medium ${STATUS_STYLES[r.status]}`}
                    >
                      {STATUS_LABELS[r.status]}
                    </span>
                  </td>
                  <td className="hidden px-4 py-3 text-gray-500 sm:table-cell">
                    {formatDate(r.created_at)}
                  </td>
                  <td className="px-4 py-3">
                    <button
                      onClick={() => {
                        setSelected(r);
                        setTeacher("");
                        setScheduleDay("");
                        setScheduleTime("");
                      }}
                      className="rounded-lg border border-gray-200 px-3 py-1.5 text-xs font-medium text-gray-900 transition-colors hover:border-[#2563EB] hover:text-[#2563EB]"
                    >
                      Detail
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
          </>
        )}
      </div>

      {/* Modal */}
      {selected && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-[#F8FAFC]/70 p-4 backdrop-blur-sm"
          onClick={() => setSelected(null)}
        >
          <div
            className="w-full max-w-lg rounded-2xl border border-gray-200 bg-white p-6"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="mb-4 flex items-center justify-between">
              <h2 className="text-lg font-bold text-gray-900">
                Detail Pendaftaran
              </h2>
              <button
                onClick={() => setSelected(null)}
                className="text-gray-500 hover:text-gray-900"
              >
                ✕
              </button>
            </div>

            <div className="space-y-3 text-sm">
              <DetailRow label="Nama" value={selected.full_name} />
              <DetailRow label="WhatsApp" value={selected.phone_whatsapp} />
              <DetailRow label="Email" value={selected.email || "-"} />
              <DetailRow label="Alamat" value={selected.address || "-"} />
              <DetailRow
                label="Instrumen"
                value={INSTRUMENT_LABELS[selected.instrument] ?? selected.instrument}
              />
              <DetailRow
                label="Level"
                value={LEVEL_LABELS[selected.level] ?? selected.level}
              />
              <DetailRow
                label="Tipe Kelas"
                value={CLASS_TYPE_LABELS[selected.class_type] ?? selected.class_type}
              />
              <DetailRow
                label="Status"
                value={STATUS_LABELS[selected.status]}
              />
              <DetailRow
                label="Tanggal Daftar"
                value={formatDate(selected.created_at)}
              />
            </div>

            {selected.status === "menunggu_konfirmasi" && (
              <div className="mt-5 border-t border-gray-200 pt-5">
                <h3 className="mb-3 text-sm font-semibold text-gray-900">
                  Setujui Pendaftaran
                </h3>
                <div className="space-y-3">
                  <div>
                    <label className="mb-1 block text-xs text-gray-500">
                      Pengajar
                    </label>
                    <input
                      type="text"
                      value={teacher}
                      onChange={(e) => setTeacher(e.target.value)}
                      placeholder="ID Pengajar"
                      className={inputClass}
                    />
                  </div>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <div>
                      <label className="mb-1 block text-xs text-gray-500">
                        Hari
                      </label>
                      <select
                        value={scheduleDay}
                        onChange={(e) => setScheduleDay(e.target.value)}
                        className={inputClass}
                      >
                        <option value="">Pilih hari</option>
                        {DAYS.map((d) => (
                          <option key={d} value={d}>
                            {d}
                          </option>
                        ))}
                      </select>
                    </div>
                    <div>
                      <label className="mb-1 block text-xs text-gray-500">
                        Jam
                      </label>
                      <input
                        type="time"
                        value={scheduleTime}
                        onChange={(e) => setScheduleTime(e.target.value)}
                        className={inputClass}
                      />
                    </div>
                  </div>
                </div>

                <div className="mt-4 flex gap-3">
                  <button
                    onClick={handleApprove}
                    disabled={!teacher || !scheduleDay || !scheduleTime}
                    className="flex-1 rounded-full bg-[#2563EB] py-2.5 text-sm font-semibold text-black transition-all hover:bg-[#0aad00] disabled:opacity-40"
                  >
                    Setujui
                  </button>
                  <button
                    onClick={handleReject}
                    className="flex-1 rounded-full border border-red-500 py-2.5 text-sm font-semibold text-red-400 transition-all hover:bg-red-500/10"
                  >
                    Tolak
                  </button>
                </div>
              </div>
            )}

            {selected.status !== "menunggu_konfirmasi" && (
              <div className="mt-4 flex justify-end">
                <button
                  onClick={() => setSelected(null)}
                  className="rounded-full border border-gray-200 px-6 py-2 text-sm text-gray-500 hover:border-[#2563EB] hover:text-gray-900"
                >
                  Tutup
                </button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between gap-4">
      <span className="shrink-0 text-gray-500">{label}</span>
      <span className="text-right text-gray-900">{value}</span>
    </div>
  );
}
