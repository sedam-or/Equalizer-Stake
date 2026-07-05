"use client";

interface StatCardProps {
  label: string;
  value: string | number;
  trend?: {
    value: number;
    isPositive: boolean;
  };
  icon?: string;
  gradient?: string;
}

export default function StatCard({
  label,
  value,
  trend,
  icon,
  gradient = "from-[#2563EB] to-[#1E40AF]",
}: StatCardProps) {
  return (
    <div className={`relative overflow-hidden rounded-xl bg-gradient-to-r ${gradient} p-6 text-white shadow-lg`}>
      {/* Icon */}
      {icon && (
        <div className="absolute right-4 top-4 text-3xl opacity-20">{icon}</div>
      )}

      {/* Value */}
      <div className="font-heading text-3xl font-bold">{value}</div>

      {/* Label */}
      <div className="mt-1 text-sm text-white/70">{label}</div>

      {/* Trend */}
      {trend && (
        <div className="mt-3 flex items-center gap-1">
          <span className={`text-sm font-semibold ${trend.isPositive ? "text-emerald-300" : "text-red-300"}`}>
            {trend.isPositive ? "↑" : "↓"} {Math.abs(trend.value)}%
          </span>
          <span className="text-xs text-white/50">vs bulan lalu</span>
        </div>
      )}
    </div>
  );
}
