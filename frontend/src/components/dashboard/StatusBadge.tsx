"use client";

interface StatusBadgeProps {
  status: string;
  size?: "sm" | "md";
}

const STATUS_STYLES: Record<string, string> = {
  active: "bg-emerald-100 text-emerald-700",
  pending: "bg-amber-100 text-amber-700",
  overdue: "bg-red-100 text-red-700",
  paid: "bg-emerald-100 text-emerald-700",
  issued: "bg-blue-100 text-blue-700",
  frozen: "bg-slate-100 text-slate-700",
  approved: "bg-emerald-100 text-emerald-700",
  rejected: "bg-red-100 text-red-700",
  hadir: "bg-emerald-100 text-emerald-700",
  izin: "bg-amber-100 text-amber-700",
  alpha: "bg-red-100 text-red-700",
};

const STATUS_LABELS: Record<string, string> = {
  active: "Aktif",
  pending: "Menunggu",
  overdue: "Terlambat",
  paid: "Lunas",
  issued: "Diterbitkan",
  frozen: "Dibekukan",
  approved: "Disetujui",
  rejected: "Ditolak",
  hadir: "Hadir",
  izin: "Izin",
  alpha: "Alpha",
};

export default function StatusBadge({ status, size = "sm" }: StatusBadgeProps) {
  const style = STATUS_STYLES[status] || "bg-gray-100 text-gray-700";
  const label = STATUS_LABELS[status] || status;

  return (
    <span
      className={`inline-flex items-center rounded-full font-medium ${style} ${
        size === "sm" ? "px-2.5 py-0.5 text-xs" : "px-3 py-1 text-sm"
      }`}
    >
      <span className={`mr-1.5 h-1.5 w-1.5 rounded-full ${style.replace("text-", "bg-")}`} />
      {label}
    </span>
  );
}
