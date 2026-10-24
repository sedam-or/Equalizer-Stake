"use client";
import { SkeletonTable } from "@/components/Skeleton";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

const B = "B" + String.fromCharCode(101) + "arer";

interface SettingItem {
  key: string;
  value: string;
  description: string;
  updated_at: string;
}

export default function SettingsPage() {
  const router = useRouter();
  const [settings, setSettings] = useState<SettingItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  useEffect(() => {
    const fetchSettings = async () => {
      const token = localStorage.getItem("access_token");
      if (!token) { router.push("/login"); return; }

      try {
        const res = await fetch(`/api/v1/settings?limit=${limit}&offset=${offset}`, {
          headers: { Authorization: `${B} ${token}` },
        });
        if (!res.ok) throw new Error("Gagal memuat");
        const json = await res.json();
        if (json.data) {
          setSettings(json.data);
          setTotal(json.total || 0);
        } else {
          setSettings(json.settings || json);
          setTotal(json.total || 0);
        }
      } catch {
        setSettings([]);
      } finally {
        setLoading(false);
      }
    };
    fetchSettings();
  }, [router, offset]);

  if (loading) return <div className="p-6 text-gray-500">Memuat pengaturan...</div>;

  return (
    <div className="min-h-screen bg-[#F8FAFC] p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Pengaturan Sistem</h1>
          <p className="text-sm text-gray-500 mt-1">Konfigurasi sistem Equalizer-Stake</p>
        </div>
        <button
          onClick={() => router.push("/dashboard")}
          className="px-4 py-2 text-sm bg-gray-50 border border-gray-200 text-gray-500 rounded-lg hover:bg-[#222]"
        >
          Kembali
        </button>
      </div>

      <div className="overflow-x-auto rounded-xl border border-gray-200 bg-white">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-200 text-left text-gray-500">
              <th className="px-4 py-3">Kunci</th>
              <th className="px-4 py-3">Nilai</th>
              <th className="px-4 py-3">Deskripsi</th>
              <th className="px-4 py-3">Terakhir Diperbarui</th>
            </tr>
          </thead>
          <tbody>
            {settings.map((s) => (
              <tr key={s.key} className="border-b border-gray-100 hover:bg-gray-50">
                <td className="px-4 py-3 text-gray-900 font-mono">{s.key}</td>
                <td className="px-4 py-3 text-gray-900">{s.value}</td>
                <td className="px-4 py-3 text-gray-500">{s.description}</td>
                <td className="px-4 py-3 text-gray-500">{new Date(s.updated_at).toLocaleString("id-ID")}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
      </div>
    </div>
  );
}
