"use client";

import type { SyncStatus } from "@/hooks/useAutoSave";

interface DraftIndicatorProps {
  status: SyncStatus;
  lastSaved: string | null;
  onRestore?: () => void;
  showRestorePrompt?: boolean;
}

export default function DraftIndicator({
  status,
  lastSaved,
  onRestore,
  showRestorePrompt = false,
}: DraftIndicatorProps) {
  if (showRestorePrompt && onRestore) {
    return (
      <div className="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded-lg flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-yellow-800">Draft ditemukan</p>
          <p className="text-xs text-yellow-600">Pulihkan draft dari sesi sebelumnya?</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={onRestore}
            className="px-3 py-1.5 text-sm bg-yellow-600 text-white rounded hover:bg-yellow-700"
          >
            Pulihkan
          </button>
          <button
            onClick={() => {}}
            className="px-3 py-1.5 text-sm bg-gray-100 text-gray-600 rounded hover:bg-gray-200"
          >
            Abaikan
          </button>
        </div>
      </div>
    );
  }

  const statusConfig: Record<SyncStatus, { label: string; color: string; icon: string }> = {
    idle: { label: "", color: "", icon: "" },
    saving: { label: "Menyimpan...", color: "text-blue-500", icon: "⏳" },
    synced: { label: "Tersinkronisasi", color: "text-green-600", icon: "✓" },
    unsynced: { label: "Belum Tersinkronisasi", color: "text-orange-500", icon: "⚠" },
    error: { label: "Gagal menyimpan", color: "text-red-500", icon: "✕" },
  };

  const config = statusConfig[status];
  if (!config.label) return null;

  return (
    <div className="flex items-center gap-2 text-xs">
      <span className={config.color}>
        {config.icon} {config.label}
      </span>
      {lastSaved && status === "synced" && (
        <span className="text-gray-400">
          • {new Date(lastSaved).toLocaleTimeString("id-ID")}
        </span>
      )}
    </div>
  );
}
