"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";

const navSections = [
  {
    label: "Admin",
    items: [
      { href: "/dashboard/schedule", label: "Jadwal", icon: "📅" },
      { href: "/dashboard/attendance", label: "Absensi", icon: "📷" },
      { href: "/dashboard/billing", label: "Tagihan", icon: "💰" },
      { href: "/dashboard/students", label: "Siswa", icon: "👨‍🎓" },
      { href: "/dashboard/payments", label: "Pembayaran", icon: "💳" },
      { href: "/dashboard/holidays", label: "Libur", icon: "🏖️" },
      { href: "/dashboard/studio", label: "Studio", icon: "🎵" },
    ],
  },
  {
    label: "Owner",
    ownerOnly: true,
    items: [
      { href: "/dashboard/admins", label: "Admins", icon: "👥" },
      { href: "/dashboard/override", label: "Override", icon: "⚡" },
      { href: "/dashboard/settings", label: "Settings", icon: "⚙️" },
      { href: "/dashboard/pricing", label: "Pricing", icon: "💲" },
      { href: "/dashboard/reports", label: "Laporan", icon: "📈" },
      { href: "/dashboard/audit-log", label: "Audit Log", icon: "📋" },
    ],
  },
  {
    label: "Portal",
    items: [
      { href: "/dashboard/my-schedule", label: "Jadwal Saya", icon: "📅" },
      { href: "/dashboard/my-children", label: "Anak Saya", icon: "👶" },
      { href: "/dashboard/my-payments", label: "Pembayaran Saya", icon: "💳" },
    ],
  },
];

const profileItem = { href: "/dashboard/profile", label: "Profil Saya", icon: "👤" };

export default function Sidebar() {
  const pathname = usePathname();
  const [role, setRole] = useState<string | null>(null);

  useEffect(() => {
    setRole(localStorage.getItem("role"));
  }, []);

  const isOwner = role === "owner";

  return (
    <aside className="hidden w-64 flex-shrink-0 border-r border-gray-200 bg-white md:flex md:flex-col">
      <div className="flex h-16 items-center border-b border-gray-200 px-6">
        <span className="text-lg font-bold text-[#1E40AF]">EQS</span>
        <span className="ml-2 text-xs text-gray-400">Admin</span>
      </div>
      <nav className="mt-4 flex-1 space-y-1 overflow-y-auto px-3">
        {/* Dashboard home */}
        <Link
          href="/dashboard"
          className={`flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-colors ${
            pathname === "/dashboard"
              ? "bg-[#08CB00] text-black"
              : "text-gray-600 hover:bg-gray-100"
          }`}
        >
          <span className="mr-3 text-lg">📊</span>
          Dashboard
        </Link>

        {navSections.map((section) => {
          if (section.ownerOnly && !isOwner) return null;
          return (
            <div key={section.label} className="mt-3">
              <p className="mb-1 px-3 text-xs font-semibold uppercase tracking-wider text-gray-400">
                {section.label}
              </p>
              {section.items.map((item) => {
                const isActive = pathname === item.href;
                return (
                  <Link
                    key={item.href}
                    href={item.href}
                    className={`flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-colors ${
                      isActive
                        ? "bg-[#08CB00] text-black"
                        : "text-gray-600 hover:bg-gray-100"
                    }`}
                  >
                    <span className="mr-3 text-lg">{item.icon}</span>
                    {item.label}
                  </Link>
                );
              })}
            </div>
          );
        })}

        {/* Notifications for all */}
        <div className="mt-3">
          <p className="mb-1 px-3 text-xs font-semibold uppercase tracking-wider text-gray-400">
            Lainnya
          </p>
          <Link
            href="/dashboard/notifications"
            className={`flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-colors ${
              pathname === "/dashboard/notifications"
                ? "bg-[#08CB00] text-black"
                : "text-gray-600 hover:bg-gray-100"
            }`}
          >
            <span className="mr-3 text-lg">🔔</span>
            Notifikasi
          </Link>
        </div>
      </nav>

      {/* Profile pinned at bottom */}
      <div className="border-t border-gray-200 p-3">
        <Link
          href={profileItem.href}
          className={`flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-colors ${
            pathname === profileItem.href
              ? "bg-[#08CB00] text-black"
              : "text-gray-600 hover:bg-gray-100"
          }`}
        >
          <span className="mr-3 text-lg">{profileItem.icon}</span>
          {profileItem.label}
        </Link>
      </div>
    </aside>
  );
}
