"use client";
/**
 * ConfirmDialog component — 4 severity levels
 * T-133: Confirmation Dialog Component
 */

export type ConfirmSeverity = "simple" | "destructive" | "financial" | "complex";

interface ConfirmDialogProps {
  open: boolean;
  severity: ConfirmSeverity;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
  loading?: boolean;
}

const severityConfig: Record<ConfirmSeverity, { confirmClass: string; icon: string }> = {
  simple: {
    confirmClass: "bg-[#2563EB] text-white hover:bg-[#1d4ed8]",
    icon: "❓",
  },
  destructive: {
    confirmClass: "bg-red-600 text-white hover:bg-red-700",
    icon: "⚠️",
  },
  financial: {
    confirmClass: "bg-[#F59E0B] text-black hover:bg-[#D97706]",
    icon: "💰",
  },
  complex: {
    confirmClass: "bg-purple-600 text-white hover:bg-purple-700",
    icon: "🔐",
  },
};

export function ConfirmDialog({
  open,
  severity,
  title,
  message,
  confirmLabel = "Konfirmasi",
  cancelLabel = "Batal",
  onConfirm,
  onCancel,
  loading = false,
}: ConfirmDialogProps) {
  if (!open) return null;

  const config = severityConfig[severity];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-[#F8FAFC]/50 animate-fade-in"
        onClick={onCancel}
        aria-label="Tutup"
      />

      {/* Dialog */}
      <div className="relative mx-4 w-full max-w-md rounded-xl bg-white p-6 shadow-2xl animate-slide-up">
        <div className="mb-4 flex items-center gap-3">
          <span className="text-2xl">{config.icon}</span>
          <h3 className="font-heading text-lg font-bold text-gray-900">{title}</h3>
        </div>

        <p className="mb-6 text-sm text-gray-600">{message}</p>

        <div className="flex gap-3">
          <button
            onClick={onCancel}
            disabled={loading}
            className="flex-1 rounded-lg border border-gray-300 px-4 py-2.5 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50"
          >
            {cancelLabel}
          </button>
          <button
            onClick={onConfirm}
            disabled={loading}
            className={`flex-1 rounded-lg px-4 py-2.5 text-sm font-semibold disabled:opacity-50 ${config.confirmClass}`}
          >
            {loading ? "Memproses..." : confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
