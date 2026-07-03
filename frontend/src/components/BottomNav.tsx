"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { NotificationBadge } from "@/components/NotificationBadge";
import { useEffect, useState } from "react";

const guardianItems = [
  { href: "/dashboard", label: "Beranda", icon: "🏠" },
  { href: "/dashboard/my-schedule", label: "Jadwal", icon: "📅" },
  { href: "/dashboard/my-children", label: "Anak", icon: "👶" },
  { href: "/dashboard/my-payments", label: "Bayar", icon: "💳" },
  { href: "/dashboard/profile", label: "Profil", icon: "👤" },
];

const teacherItems = [
  { href: "/dashboard", label: "Beranda", icon: "🏠" },
  { href: "/dashboard/my-schedule", label: "Jadwal", icon: "📅" },
  { href: "/dashboard/attendance/submit", label: "Absen", icon: "📷" },
  { href: "/dashboard/notifications", label: "Notif", icon: "🔔" },
  { href: "/dashboard/profile", label: "Profil", icon: "👤" },
];

type NavItem = { href: string; label: string; icon: string };

export default function BottomNav() {
  const pathname = usePathname();
  const [role, setRole] = useState<string>("");
  const [items, setItems] = useState<NavItem[]>([]);

  useEffect(() => {
    const r = localStorage.getItem("role") || "";
    setRole(r);
    if (r === "guardian") setItems(guardianItems);
    else if (r === "teacher") setItems(teacherItems);
    else setItems([]); // admin/owner don't use bottom nav
  }, []);

  // Only show for guardian and teacher
  if (role !== "guardian" && role !== "teacher") return null;
  if (items.length === 0) return null;

  return (
    <nav className="fixed bottom-0 left-0 right-0 z-50 border-t border-[#CBD5E1] bg-[#0a0a0a] md:hidden">
      <div className="flex items-center justify-around px-2 py-2">
        {items.map((item) => {
          const isActive = pathname === item.href;
          return (
            <Link
              key={item.href}
              href={item.href}
              className={`flex flex-col items-center gap-0.5 rounded-lg px-2 py-1 text-xs ${
                isActive
                  ? "text-[#2563EB]"
                  : "text-gray-500 hover:text-gray-300"
              }`}
            >
              <span className="relative text-lg">{item.icon}{item.label === "Notif" && <NotificationBadge />}</span>
              <span className="text-[10px]">{item.label}</span>
            </Link>
          );
        })}
      </div>
    </nav>
  );
}
