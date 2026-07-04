"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import { NotificationBadge } from "@/components/NotificationBadge";
import { ThemeToggle } from "@/components/ThemeToggle";

type Role = "owner" | "admin" | "teacher" | "guardian";

interface NavItem {
  href: string;
  label: string;
  icon: string;
}

interface NavSection {
  label: string;
  roles: Role[];
  items: NavItem[];
}

const navSections: NavSection[] = [
  {
    label: "Operasional",
    roles: ["admin"],
    items: [
      { href: "/dashboard/schedule", label: "Jadwal", icon: "📅" },
      { href: "/dashboard/attendance", label: "Absensi", icon: "📷" },
      { href: "/dashboard/billing", label: "Tagihan", icon: "💰" },
      { href: "/dashboard/students", label: "Siswa", icon: "👨‍🎓" },
      { href: "/dashboard/payments", label: "Pembayaran", icon: "💳" },
      { href: "/dashboard/holidays", label: "Libur", icon: "🏖️" },
      { href: "/dashboard/studio", label: "Studio", icon: "🎵" },
      { href: "/dashboard/registrations", label: "Pendaftaran", icon: "📝" },
    ],
  },
  {
    label: "Manajemen",
    roles: ["owner"],
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
    roles: ["teacher", "guardian"],
    items: [
      { href: "/dashboard/my-schedule", label: "Jadwal Saya", icon: "📅" },
      { href: "/dashboard/attendance/submit", label: "Absen", icon: "📷" },
      { href: "/dashboard/my-children", label: "Anak Saya", icon: "👶" },
      { href: "/dashboard/my-payments", label: "Pembayaran Saya", icon: "💳" },
    ],
  },
];

const guardianOnlyItems = ["/dashboard/my-children", "/dashboard/my-payments"];
const profileItem = { href: "/dashboard/profile", label: "Profil Saya", icon: "👤" };

export default function Sidebar() {
  const pathname = usePathname();
  const [role, setRole] = useState<Role | null>(null);

  useEffect(() => {
    setRole(localStorage.getItem("role") as Role | null);
  }, []);

  const currentRole: Role = role ?? "admin";

  return (
    <aside className="hidden w-64 flex-shrink-0 border-r border-gray-200 bg-white md:flex md:flex-col">
      <div className="flex h-16 items-center border-b border-gray-200 px-6">
        <span className="text-lg font-bold text-[#2563EB]">BMD</span>
        <span className="ml-2 text-xs capitalize text-gray-500">{currentRole}</span>
      </div>
      <nav className="mt-4 flex-1 space-y-1 overflow-y-auto px-3">
        <Link
          href="/dashboard"
          className={`flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-colors ${
            pathname === "/dashboard"
              ? "bg-[#2563EB] text-black"
              : "text-gray-500 hover:bg-gray-100 hover:text-gray-900"
          }`}
        >
          <span className="mr-3 text-lg">📊</span>
          Dashboard
        </Link>

        {navSections.map((section) => {
          if (!section.roles.includes(currentRole)) return null;
          const visibleItems = section.items.filter((item) => {
            if (currentRole === "teacher" && guardianOnlyItems.includes(item.href)) return false;
            return true;
          });
          if (visibleItems.length === 0) return null;

          return (
            <div key={section.label} className="mt-3">
              <p className="mb-1 px-3 text-xs font-semibold uppercase tracking-wider text-gray-600">
                {section.label}
              </p>
              {visibleItems.map((item) => {
                const isActive = pathname === item.href;
                return (
                  <Link
                    key={item.href}
                    href={item.href}
                    className={`flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-colors ${
                      isActive
                        ? "bg-[#2563EB] text-black"
                        : "text-gray-500 hover:bg-gray-100 hover:text-gray-900"
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

        <div className="mt-3">
          <p className="mb-1 px-3 text-xs font-semibold uppercase tracking-wider text-gray-600">
            Lainnya
          </p>
          <Link
            href="/dashboard/notifications"
            className={`flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-colors ${
              pathname === "/dashboard/notifications"
                ? "bg-[#2563EB] text-black"
                : "text-gray-500 hover:bg-gray-100 hover:text-gray-900"
            }`}
          >
            <span className="relative mr-3 text-lg">🔔<NotificationBadge /></span>
            Notifikasi
          </Link>
        </div>
      </nav>

      <div className="border-t border-gray-200 p-3">
        <Link
          href={profileItem.href}
          className={`flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-colors ${
            pathname === profileItem.href
              ? "bg-[#2563EB] text-black"
              : "text-gray-500 hover:bg-gray-100 hover:text-gray-900"
          }`}
        >
          <span className="mr-3 text-lg">{profileItem.icon}</span>
          {profileItem.label}
        </Link>
      </div>
      <div className="px-3 pb-3">
        <ThemeToggle />
      </div>
    </aside>
  );
}
