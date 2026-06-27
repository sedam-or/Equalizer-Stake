"use client";

import { useState } from "react";

interface Notification {
  id: string;
  channel: string;
  category: string;
  subject: string;
  body: string;
  status: string;
  retryCount: number;
  createdAt: string;
  read: boolean;
}

export default function NotificationsPage() {
  const [notifications, setNotifications] = useState<Notification[]>([
    { id: "1", channel: "whatsapp", category: "non_critical", subject: "Pengingat Jadwal", body: "Andi Pratama les Gitar pukul 14:30", status: "sent", retryCount: 0, createdAt: "2026-01-15T13:30:00", read: false },
    { id: "2", channel: "email", category: "critical", subject: "Tagihan Januari 2026", body: "Tagihan Rp 850.000 jatuh tempo 10 Januari", status: "sent", retryCount: 0, createdAt: "2026-01-05T00:00:00", read: true },
    { id: "3", channel: "whatsapp", category: "critical", subject: "Akun Dibekukan", body: "Akun Anda dibekukan karena tagihan terlambat", status: "failed", retryCount: 3, createdAt: "2026-01-11T00:00:00", read: false },
    { id: "4", channel: "email", category: "non_critical", subject: "Pembayaran Diverifikasi", body: "Pembayaran Rp 850.127 telah diverifikasi", status: "sent", retryCount: 0, createdAt: "2026-01-08T10:00:00", read: true },
  ]);

  const [filter, setFilter] = useState("all");

  const filtered = notifications.filter((n) => {
    if (filter === "all") return true;
    if (filter === "unread") return !n.read;
    if (filter === "critical") return n.category === "critical";
    if (filter === "failed") return n.status === "failed";
    return true;
  });

  const markAsRead = (id: string) => {
    setNotifications((prev) =>
      prev.map((n) => (n.id === id ? { ...n, read: true } : n))
    );
  };

  const unreadCount = notifications.filter((n) => !n.read).length;

  const channelIcons: Record<string, string> = {
    whatsapp: "💬",
    email: "📧",
  };

  const statusColors: Record<string, string> = {
    sent: "text-[#08CB00]",
    failed: "text-red-500",
    pending: "text-yellow-600",
    retrying: "text-yellow-600",
  };

  const statusLabels: Record<string, string> = {
    sent: "Terkirim",
    failed: "Gagal",
    pending: "Menunggu",
    retrying: "Retry",
  };

  return (
    <div>
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[#000000]">Notifikasi</h1>
          <p className="mt-1 text-sm text-gray-500">
            {unreadCount > 0 ? `${unreadCount} notifikasi belum dibaca` : "Semua sudah dibaca"}
          </p>
        </div>
      </div>

      {/* Filters */}
      <div className="mt-4 flex gap-2">
        {["all", "unread", "critical", "failed"].map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`rounded-full px-3 py-1 text-xs font-medium transition-colors ${
              filter === f
                ? "bg-[#08CB00] text-black"
                : "bg-gray-100 text-gray-600 hover:bg-gray-200"
            }`}
          >
            {f === "all" ? "Semua" : f === "unread" ? "Belum Dibaca" : f === "critical" ? "Kritis" : "Gagal"}
          </button>
        ))}
      </div>

      {/* Notification list */}
      <div className="mt-4 space-y-2">
        {filtered.length === 0 ? (
          <div className="rounded-lg border border-dashed border-gray-300 p-8 text-center text-gray-500">
            Tidak ada notifikasi
          </div>
        ) : (
          filtered.map((n) => (
            <div
              key={n.id}
              onClick={() => markAsRead(n.id)}
              className={`cursor-pointer rounded-lg border p-4 transition-colors ${
                n.read
                  ? "border-gray-200 bg-white"
                  : "border-[#253900] bg-[#f0fff0]"
              }`}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-3">
                  <span className="text-lg">{channelIcons[n.channel]}</span>
                  <div>
                    <div className="flex items-center gap-2">
                      <p className="font-medium text-[#000000]">{n.subject}</p>
                      {!n.read && (
                        <span className="h-2 w-2 rounded-full bg-[#08CB00]" />
                      )}
                    </div>
                    <p className="mt-0.5 text-sm text-gray-600">{n.body}</p>
                    <div className="mt-2 flex items-center gap-3 text-xs text-gray-400">
                      <span>{new Date(n.createdAt).toLocaleString("id-ID")}</span>
                      <span className={`font-medium ${statusColors[n.status]}`}>
                        {statusLabels[n.status]}
                      </span>
                      {n.retryCount > 0 && (
                        <span>Retry: {n.retryCount}x</span>
                      )}
                      <span className="capitalize">{n.channel}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Info */}
      <div className="mt-6 rounded-lg bg-gray-50 p-3 text-xs text-gray-500">
        <p>• Notifikasi kritis: email utama, WhatsApp cadangan</p>
        <p>• Notifikasi non-kritis: WhatsApp utama, email cadangan</p>
        <p>• Retry otomatis 3x (10s → 20s → 40s) lalu fallback channel</p>
      </div>
    </div>
  );
}
