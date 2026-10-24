"use client";
import { parseApiError } from "@/lib/api-error";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

interface Notification {
  id_notification: string;
  guardian_name?: string;
  guardian?: string;
  students?: string[];
  message: string;
  time?: string;
  created_at?: string;
  unread_count?: number;
  unreadCount?: number;
}

export default function NotificationsPage() {
  const router = useRouter();
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  useEffect(() => {
    const token = localStorage.getItem("access_token");
    if (!token) { router.push("/login"); return; }
    fetch(`/api/notifications/?limit=${limit}&offset=${offset}`, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => { if (!r.ok) throw new Error(); return r.json(); })
      .then(res => {
        setNotifications(res.data || []);
        setTotal(res.total || 0);
      })
      .catch(() => setNotifications([]))
      .finally(() => setLoading(false));
  }, [router, offset]);

  const getName = (n: Notification) => n.guardian_name || n.guardian || "-";
  const getStudents = (n: Notification) => n.students || [];
  const getTime = (n: Notification) => n.time || (n.created_at ? new Date(n.created_at).toLocaleDateString("id-ID") : "");
  const getUnread = (n: Notification) => n.unread_count ?? n.unreadCount ?? 0;

  return (
    <div className="min-h-screen bg-[#F8FAFC] p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">
            Notifikasi
          </h1>
          <p className="text-sm text-gray-500 mt-1">
            Notifikasi dikelompokkan per guardian
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => router.push("/dashboard/notification-settings")}
            className="px-4 py-2 text-sm bg-[#CBD5E1] border border-gray-200 text-gray-900 rounded-lg hover:bg-[#355000]"
          >
            Pengaturan Notifikasi
          </button>
          <button
            onClick={() => router.push("/dashboard")}
            className="px-4 py-2 text-sm bg-gray-50 border border-gray-200 text-gray-500 rounded-lg hover:bg-[#222]"
          >
            Kembali
          </button>
        </div>
      </div>

      {loading ? (
        <div className="p-8 text-center text-gray-500">Memuat notifikasi...</div>
      ) : notifications.length === 0 ? (
        <div className="p-8 text-center text-gray-500">Belum ada notifikasi</div>
      ) : (
        <>
        <div className="space-y-3">
          {notifications.map((item, idx) => (
            <div
              key={item.id_notification || idx}
              className={`rounded-xl border p-4 ${
                getUnread(item) > 0
                  ? "border-gray-200 bg-white"
                  : "border-gray-100 bg-gray-50"
              }`}
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="font-semibold text-gray-900">
                      {getName(item)}
                    </span>
                    {getUnread(item) > 0 && (
                      <span className="inline-flex items-center justify-center w-5 h-5 text-xs font-bold bg-red-600 text-white rounded-full">
                        {getUnread(item)}
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-gray-500 mb-2">
                    {getStudents(item).join(", ")}
                  </p>
                  <p className="text-sm text-gray-900">{item.message}</p>
                </div>
                <span className="text-xs text-gray-500 whitespace-nowrap ml-4">
                  {getTime(item)}
                </span>
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
