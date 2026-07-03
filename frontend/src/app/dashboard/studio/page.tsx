"use client";
/**
 * Studio Room Management
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface Room {
  id_room: string;
  name: string;
  capacity: number;
  equipment: string;
  status: string;
}

const STATUS_LABELS: Record<string, string> = {
  available: "Tersedia",
  in_use: "Sedang Dipakai",
  maintenance: "Perawatan",
};

export default function StudioPage() {
  const router = useRouter();
  const [rooms, setRooms] = useState<Room[]>([]);
  const [loading, setLoading] = useState(true);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    fetch(`/api/sessions/rooms?limit=${limit}&offset=${offset}`, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.ok ? r.json() : { data: [], total: 0 })
      .then(res => {
        setRooms(res.data || (Array.isArray(res) ? res : res.rooms || []));
        setTotal(res.total || 0);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [router, offset]);

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-[#EEEEEE]">Studio Musik</h1>
        <p className="text-sm text-gray-400">Kelola ruang studio dan peralatan</p>
      </div>

      {loading ? (
        <div className="p-8 text-center text-gray-400">Memuat data studio...</div>
      ) : rooms.length === 0 ? (
        <div className="p-8 text-center text-gray-400">Belum ada ruang studio</div>
      ) : (
        <>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {rooms.map(room => (
            <div key={room.id_room} className="bg-[#111] rounded-xl border border-[#253900] p-5 space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold text-[#EEEEEE]">{room.name}</h3>
                <span className={`rounded-full px-2 py-1 text-xs font-medium ${
                  room.status === "available" ? "bg-[#08CB00]/10 text-[#08CB00]" :
                  room.status === "in_use" ? "bg-yellow-500/10 text-yellow-500" :
                  "bg-red-500/10 text-red-400"
                }`}>
                  {STATUS_LABELS[room.status] || room.status}
                </span>
              </div>
              <div className="space-y-1">
                <p className="text-sm text-gray-400">Kapasitas: <span className="text-[#EEEEEE]">{room.capacity} orang</span></p>
                <p className="text-sm text-gray-400">Peralatan: <span className="text-[#EEEEEE]">{room.equipment}</span></p>
              </div>
            </div>
          ))}
        </div>
        <div className="mt-4">
          <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
        </div>
        </>
      )}
    </div>
  );
}
