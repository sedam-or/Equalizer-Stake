"use client";

import { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";

interface Session {
  id: string;
  student: string;
  teacher: string;
  room: string;
  date: string;
  startTime: string;
  endTime: string;
  status: string;
}

export default function SchedulePage() {
  const router = useRouter();
  const [sessions, setSessions] = useState<Session[]>([]);
  const [selectedDate, setSelectedDate] = useState(
    new Date().toISOString().split("T")[0]
  );
  const [showForm, setShowForm] = useState(false);
  const [loading, setLoading] = useState(true);

  const fetchSessions = useCallback(
    async (date: string) => {
      const token = localStorage.getItem("token");
      if (!token) {
        router.push("/login");
        return;
      }

      setLoading(true);
      try {
        const res = await fetch(
          `/api/scheduling/sessions?date=${date}`,
          { headers: { Authorization: `Bearer ${token}` } }
        );
        if (res.status === 401) {
          router.push("/login");
          return;
        }
        if (!res.ok) throw new Error("Gagal memuat jadwal");
        const data = await res.json();
        setSessions(Array.isArray(data) ? data : data.sessions ?? []);
      } catch {
        setSessions([]);
      } finally {
        setLoading(false);
      }
    },
    [router]
  );

  useEffect(() => {
    fetchSessions(selectedDate);
  }, [selectedDate, fetchSessions]);

  return (
    <div>
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[#08CB00]">Penjadwalan</h1>
          <p className="mt-1 text-sm text-[#888]">Kelola jadwal kursus</p>
        </div>
        <button
          onClick={() => setShowForm(true)}
          className="rounded-lg bg-[#08CB00] px-4 py-2 text-sm font-semibold text-black transition-colors hover:bg-[#0aad00]"
        >
          + Jadwal Baru
        </button>
      </div>

      {/* Date picker */}
      <div className="mt-6">
        <input
          type="date"
          value={selectedDate}
          onChange={(e) => setSelectedDate(e.target.value)}
          className="rounded-lg border border-[#333] bg-[#111] px-4 py-2 text-sm text-[#EEE]"
        />
      </div>

      {/* Sessions list */}
      <div className="mt-6 space-y-3">
        {loading ? (
          <div className="py-8 text-center text-[#888]">Memuat jadwal...</div>
        ) : sessions.length === 0 ? (
          <div className="rounded-lg border border-dashed border-[#333] p-8 text-center text-[#888]">
            Tidak ada jadwal untuk tanggal ini
          </div>
        ) : (
          sessions.map((session) => (
            <SessionRow key={session.id} session={session} />
          ))
        )}
      </div>

      {/* Conflict indicator */}
      <div className="mt-4 rounded-lg border border-yellow-900/40 bg-yellow-900/10 p-3 text-sm text-yellow-400">
        ⚠️ Validasi konflik aktif — guru, ruangan, dan waktu diperiksa secara
        real-time
      </div>

      {/* New session form modal */}
      {showForm && (
        <NewSessionForm
          onClose={() => setShowForm(false)}
          onSaved={() => fetchSessions(selectedDate)}
        />
      )}
    </div>
  );
}

function SessionRow({ session }: { session: Session }) {
  const statusColors: Record<string, string> = {
    locked: "bg-[#08CB00]/10 text-[#08CB00]",
    pending: "bg-yellow-900/30 text-yellow-400",
    draft: "bg-[#333]/50 text-[#888]",
    completed: "bg-blue-900/30 text-blue-400",
    cancelled: "bg-red-900/30 text-red-400",
  };

  const statusLabels: Record<string, string> = {
    locked: "Terkunci",
    pending: "Menunggu",
    draft: "Draft",
    completed: "Selesai",
    cancelled: "Dibatalkan",
  };

  return (
    <div className="flex items-center justify-between rounded-lg border border-[#222] bg-[#111] p-4">
      <div className="flex items-center gap-4">
        <div className="text-center">
          <div className="text-lg font-bold text-[#08CB00]">
            {session.startTime}
          </div>
          <div className="text-xs text-[#888]">{session.endTime}</div>
        </div>
        <div>
          <p className="font-medium text-[#EEE]">{session.student}</p>
          <p className="text-sm text-[#888]">
            {session.teacher} · {session.room}
          </p>
        </div>
      </div>
      <div className="flex items-center gap-2">
        <span
          className={`rounded-full px-2 py-1 text-xs font-medium ${statusColors[session.status] ?? statusColors.draft}`}
        >
          {statusLabels[session.status] ?? session.status}
        </span>
        <button className="rounded p-1 text-[#666] hover:bg-[#222] hover:text-[#EEE]">
          ⋮
        </button>
      </div>
    </div>
  );
}

function NewSessionForm({
  onClose,
  onSaved,
}: {
  onClose: () => void;
  onSaved: () => void;
}) {
  const router = useRouter();
  const [form, setForm] = useState({
    student: "",
    teacher: "",
    room: "",
    date: "",
    startTime: "",
    endTime: "",
  });
  const [conflict, setConflict] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>
  ) => {
    setForm({ ...form, [e.target.name]: e.target.value });
    setConflict(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const token = localStorage.getItem("token");
    if (!token) {
      router.push("/login");
      return;
    }
    setSaving(true);
    try {
      const res = await fetch("/api/scheduling/sessions", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify(form),
      });
      if (res.status === 401) {
        router.push("/login");
        return;
      }
      if (res.status === 409) {
        const data = await res.json().catch(() => null);
        setConflict(data?.detail ?? "Konflik jadwal terdeteksi");
        return;
      }
      if (!res.ok) throw new Error("Gagal menyimpan jadwal");
      onSaved();
      onClose();
    } catch (err) {
      setConflict(err instanceof Error ? err.message : "Terjadi kesalahan");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md rounded-2xl bg-[#111] border border-[#333] p-6 shadow-xl">
        <h2 className="text-lg font-bold text-[#08CB00]">Jadwal Baru</h2>

        <form onSubmit={handleSubmit} className="mt-4 space-y-4">
          <div>
            <label className="mb-1 block text-sm font-medium text-[#aaa]">
              Siswa
            </label>
            <select
              name="student"
              value={form.student}
              onChange={handleChange}
              className="w-full rounded-lg border border-[#333] bg-[#000] px-3 py-2 text-sm text-[#EEE]"
              required
            >
              <option value="">Pilih siswa</option>
              <option value="1">Andi Pratama</option>
              <option value="2">Rina Wulandari</option>
            </select>
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium text-[#aaa]">
              Guru
            </label>
            <select
              name="teacher"
              value={form.teacher}
              onChange={handleChange}
              className="w-full rounded-lg border border-[#333] bg-[#000] px-3 py-2 text-sm text-[#EEE]"
              required
            >
              <option value="">Pilih guru</option>
              <option value="1">Pak Budi</option>
              <option value="2">Bu Sari</option>
            </select>
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium text-[#aaa]">
              Ruangan
            </label>
            <select
              name="room"
              value={form.room}
              onChange={handleChange}
              className="w-full rounded-lg border border-[#333] bg-[#000] px-3 py-2 text-sm text-[#EEE]"
              required
            >
              <option value="">Pilih ruangan</option>
              <option value="1">Studio Gitar</option>
              <option value="2">Studio Piano</option>
              <option value="3">Studio Drum</option>
            </select>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="mb-1 block text-sm font-medium text-[#aaa]">
                Tanggal
              </label>
              <input
                type="date"
                name="date"
                value={form.date}
                onChange={handleChange}
                className="w-full rounded-lg border border-[#333] bg-[#000] px-3 py-2 text-sm text-[#EEE]"
                required
              />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-[#aaa]">
                Jam Mulai
              </label>
              <input
                type="time"
                name="startTime"
                value={form.startTime}
                onChange={handleChange}
                className="w-full rounded-lg border border-[#333] bg-[#000] px-3 py-2 text-sm text-[#EEE]"
                required
              />
            </div>
          </div>

          {conflict && (
            <div className="rounded-lg bg-red-900/20 p-3 text-sm text-red-400">
              ⚠️ {conflict}
            </div>
          )}

          <div className="flex gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 rounded-lg border border-[#444] py-2 text-sm font-medium text-[#aaa] hover:bg-[#222]"
            >
              Batal
            </button>
            <button
              type="submit"
              disabled={saving}
              className="flex-1 rounded-lg bg-[#08CB00] py-2 text-sm font-semibold text-black hover:bg-[#0aad00] disabled:opacity-50"
            >
              {saving ? "Menyimpan..." : "Simpan"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
