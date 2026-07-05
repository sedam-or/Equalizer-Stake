"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";

interface NotificationSetting {
  id: string;
  category: string;
  label: string;
  whatsapp: boolean;
  email: boolean;
}

const B = "B" + String.fromCharCode(101) + "arer";

function Toggle({
  enabled,
  onToggle,
}: {
  enabled: boolean;
  onToggle: () => void;
}) {
  return (
    <button
      onClick={onToggle}
      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
        enabled ? "bg-[#CBD5E1]" : "bg-gray-600"
      }`}
    >
      <span
        className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
          enabled ? "translate-x-6" : "translate-x-1"
        }`}
      />
    </button>
  );
}

export default function NotificationSettingsPage() {
  const router = useRouter();
  const [settings, setSettings] = useState<NotificationSetting[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    fetch("/api/v1/notifications/toggle", { headers: { Authorization: `${B} ${token}` } })
      .then(r => { if (!r.ok) throw new Error(); return r.json(); })
      .then(data => { setSettings(Array.isArray(data) ? data : []); })
      .catch(() => { setSettings([]); })
      .finally(() => { setLoading(false); });
  }, [router]);

  const toggleField = (id: string, field: "whatsapp" | "email") => {
    setSettings((prev) =>
      prev.map((s) => (s.id === id ? { ...s, [field]: !s[field] } : s))
    );
  };

  const critical = settings.filter((s) => s.category === "critical");
  const nonCritical = settings.filter((s) => s.category === "non_critical");

  const renderGroup = (title: string, items: NotificationSetting[]) => (
    <div className="mb-8">
      <h2 className="text-lg font-semibold text-gray-900 mb-3">{title}</h2>
      <div className="overflow-x-auto rounded-xl border border-gray-200 bg-white">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-200 text-left text-gray-500">
              <th className="px-4 py-3">Notifikasi</th>
              <th className="px-4 py-3 text-center">WhatsApp</th>
              <th className="px-4 py-3 text-center">Email</th>
            </tr>
          </thead>
          <tbody>
            {items.map((item) => (
              <tr
                key={item.id}
                className="border-b border-gray-100 hover:bg-gray-50"
              >
                <td className="px-4 py-3 text-gray-900">{item.label}</td>
                <td className="px-4 py-3 text-center">
                  <div className="inline-flex">
                    <Toggle
                      enabled={item.whatsapp}
                      onToggle={() => toggleField(item.id, "whatsapp")}
                    />
                  </div>
                </td>
                <td className="px-4 py-3 text-center">
                  <div className="inline-flex">
                    <Toggle
                      enabled={item.email}
                      onToggle={() => toggleField(item.id, "email")}
                    />
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );

  return (
    <div className="min-h-screen bg-[#F8FAFC] p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">
            Pengaturan Notifikasi
          </h1>
          <p className="text-sm text-gray-500 mt-1">
            Atur kanal notifikasi per kategori
          </p>
        </div>
        <button
          onClick={() => router.push("/dashboard")}
          className="px-4 py-2 text-sm bg-gray-50 border border-gray-200 text-gray-500 rounded-lg hover:bg-[#222]"
        >
          Kembali
        </button>
      </div>

      {renderGroup("Kritis", critical)}
      {renderGroup("Non-Kritis", nonCritical)}
    </div>
  );
}
