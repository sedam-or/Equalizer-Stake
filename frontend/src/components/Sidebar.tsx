"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const navItems = [
  { href: "/dashboard", label: "Dashboard", icon: "📊" },
  { href: "/dashboard/schedule", label: "Jadwal", icon: "📅" },
  { href: "/dashboard/attendance", label: "Absensi", icon: "📷" },
  { href: "/dashboard/billing", label: "Tagihan", icon: "💰" },
  { href: "/dashboard/students", label: "Siswa", icon: "👨‍🎓" },
  { href: "/dashboard/notifications", label: "Notifikasi", icon: "🔔" },
  { href: "/dashboard/reports", label: "Laporan", icon: "📈" },
  { href: "/dashboard/audit-log", label: "Audit Log", icon: "📋" },
];

export default function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="hidden w-64 flex-shrink-0 border-r border-gray-200 bg-white md:flex md:flex-col">
      <div className="flex h-16 items-center border-b border-gray-200 px-6">
        <span className="text-lg font-bold text-[#1E40AF]">EQS</span>
        <span className="ml-2 text-xs text-gray-400">Admin</span>
      </div>
      <nav className="mt-4 flex-1 space-y-1 px-3">
        {navItems.map((item) => {
          const isActive = pathname === item.href;
          return (
            <Link
              key={item.href}
              href={item.href}
              className={`flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-[#1E40AF] text-white"
                  : "text-gray-600 hover:bg-gray-100"
              }`}
            >
              <span className="mr-3 text-lg">{item.icon}</span>
              {item.label}
            </Link>
          );
        })}
      </nav>
      <div className="border-t border-gray-200 p-3">
        <Link
          href="/dashboard/profile"
          className={`flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-colors ${
            pathname === "/dashboard/profile"
              ? "bg-[#08CB00] text-black"
              : "text-gray-600 hover:bg-gray-100"
          }`}
        >
          <span className="mr-3 text-lg">👤</span>
          Profil Saya
        </Link>
      </div>
    </aside>
  );
}
