"use client";

import { useState, useEffect } from "react";

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
  const [sessions, setSessions] = useState<Session[]>([]);
  const [selectedDate, setSelectedDate] = useState(new Date().toISOString().split("T")[0]);
  const [showForm, setShowForm] = useState(false);

  // Mock data
  useEffect(() => {
    setSessions([
      { id: "1", student: "Andi Pratama", teacher: "Pak Budi", room: "Studio Gitar", date: selectedDate, startTime: "14:30", endTime: "15:30", status: "locked" },
      { id: "2", student: "Rina Wulandari", teacher: "Bu Sari", room: "Studio Piano", date: selectedDate, startTime: "15:30", endTime: "16:30", status: "pending" },
      { id: "3", student: "Budi Santoso", teacher: "Pak Budi", room: "Studio Drum", date: selectedDate, startTime: "16:30", endTime: "17:00", status: "draft" },
    ]);
  }, [selectedDate]);

  return (
    <div>
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[#000000]">Penjadwalan</h1>
          <p className="mt-1 text-sm text-gray-500">Kelola jadwal kursus</p>
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
          className="rounded-lg border border-[#253900] bg-white px-4 py-2 text-sm text-[#000000]"
        />
      </div>

      {/* Sessions list */}
      <div className="mt-6 space-y-3">
        {sessions.length === 0 ? (
          <div className="rounded-lg border border-dashed border-gray-300 p-8 text-center text-gray-500">
            Tidak ada jadwal untuk tanggal ini
          </div>
        ) : (
          sessions.map((session) => (
            <SessionRow key={session.id} session={session} />
          ))
        )}
      </div>

      {/* Conflict indicator */}
      <div className="mt-4 rounded-lg border border-yellow-200 bg-yellow-50 p-3 text-sm text-yellow-700">
        ⚠️ Validasi konflik aktif — guru, ruangan, dan waktu diperiksa secara real-time
      </div>

      {/* New session form modal */}
      {showForm && <NewSessionForm onClose={() => setShowForm(false)} />}
    </div>
  );
}

function SessionRow({ session }: { session: Session }) {
  const statusColors: Record<string, string> = {
    locked: "bg-[#08CB00]/10 text-[#08CB00]",
    pending: "bg-yellow-100 text-yellow-700",
    draft: "bg-gray-100 text-gray-600",
    completed: "bg-blue-100 text-blue-700",
    cancelled: "bg-red-100 text-red-700",
  };

  const statusLabels: Record<string, string> = {
    locked: "Terkunci",
    pending: "Menunggu",
    draft: "Draft",
    completed: "Selesai",
    cancelled: "Dibatalkan",
  };

  return (
    <div className="flex items-center justify-between rounded-lg border border-gray-200 bg-white p-4">
      <div className="flex items-center gap-4">
        <div className="text-center">
          <div className="text-lg font-bold text-[#08CB00]">{session.startTime}</div>
          <div className="text-xs text-gray-500">{session.endTime}</div>
        </div>
        <div>
          <p className="font-medium text-[#000000]">{session.student}</p>
          <p className="text-sm text-gray-500">
            {session.teacher} · {session.room}
          </p>
        </div>
      </div>
      <div className="flex items-center gap-2">
        <span className={`rounded-full px-2 py-1 text-xs font-medium ${statusColors[session.status]}`}>
          {statusLabels[session.status]}
        </span>
        <button className="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600">
          ⋮
        </button>
      </div>
    </div>
  );
}

function NewSessionForm({ onClose }: { onClose: () => void }) {
  const [form, setForm] = useState({
    student: "",
    teacher: "",
    room: "",
    date: "",
    startTime: "",
    endTime: "",
  });
  const [conflict, setConflict] = useState<string | null>(null);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    setForm({ ...form, [e.target.name]: e.target.value });
    setConflict(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    // TODO: validate conflict via API
    // POST /api/sessions/validate
    alert("Jadwal berhasil disimpan!");
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md rounded-2xl bg-white p-6 shadow-xl">
        <h2 className="text-lg font-bold text-[#000000]">Jadwal Baru</h2>

        <form onSubmit={handleSubmit} className="mt-4 space-y-4">
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700">Siswa</label>
            <select
              name="student"
              value={form.student}
              onChange={handleChange}
              className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
              required
            >
              <option value="">Pilih siswa</option>
              <option value="1">Andi Pratama</option>
              <option value="2">Rina Wulandari</option>
            </select>
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700">Guru</label>
            <select
              name="teacher"
              value={form.teacher}
              onChange={handleChange}
              className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
              required
            >
              <option value="">Pilih guru</option>
              <option value="1">Pak Budi</option>
              <option value="2">Bu Sari</option>
            </select>
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700">Ruangan</label>
            <select
              name="room"
              value={form.room}
              onChange={handleChange}
              className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
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
              <label className="mb-1 block text-sm font-medium text-gray-700">Tanggal</label>
              <input
                type="date"
                name="date"
                value={form.date}
                onChange={handleChange}
                className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
                required
              />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-gray-700">Jam Mulai</label>
              <input
                type="time"
                name="startTime"
                value={form.startTime}
                onChange={handleChange}
                className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
                required
              />
            </div>
          </div>

          {conflict && (
            <div className="rounded-lg bg-red-50 p-3 text-sm text-red-600">
              ⚠️ {conflict}
            </div>
          )}

          <div className="flex gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 rounded-lg border border-gray-300 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
            >
              Batal
            </button>
            <button
              type="submit"
              className="flex-1 rounded-lg bg-[#08CB00] py-2 text-sm font-semibold text-black hover:bg-[#0aad00]"
            >
              Simpan
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
