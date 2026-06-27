"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const bottomNavItems = [
  { href: "/dashboard", label: "Beranda", icon: "🏠" },
  { href: "/dashboard/schedule", label: "Jadwal", icon: "📅" },
  { href: "/dashboard/attendance", label: "Absensi", icon: "📷" },
  { href: "/dashboard/billing", label: "Tagihan", icon: "💰" },
  { href: "/dashboard/notifications", label: "Notif", icon: "🔔" },
];

export default function BottomNav() {
  const pathname = usePathname();

  return (
    <nav className="fixed bottom-0 left-0 right-0 z-50 border-t border-gray-200 bg-white md:hidden">
      <div className="flex items-center justify-around py-2">
        {bottomNavItems.map((item) => {
          const isActive = pathname === item.href;
          return (
            <Link
              key={item.href}
              href={item.href}
              className={`flex flex-col items-center px-2 py-1 text-xs ${
                isActive ? "text-[#1E40AF]" : "text-gray-500"
              }`}
            >
              <span className="text-xl">{item.icon}</span>
              <span className="mt-0.5">{item.label}</span>
            </Link>
          );
        })}
      </div>
    </nav>
  );
}
