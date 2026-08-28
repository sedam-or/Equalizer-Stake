"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";

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
      { href: "/dashboard/students", label: "Student", icon: "👨‍🎓" },
      { href: "/dashboard/payments", label: "Pembayaran", icon: "💳" },
      { href: "/dashboard/holidays", label: "Libur", icon: "🏖️" },
      { href: "/dashboard/studio", label: "Studio", icon: "🎵" },
      { href: "/dashboard/teachers", label: "Instructor", icon: "👨‍🏫" },
      { href: "/dashboard/registrations", label: "Pendaftaran", icon: "📝" },
      { href: "/dashboard/assign-session", label: "Assign Sesi", icon: "🔗" },
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
    label: "Instructor",
    roles: ["teacher"],
    items: [
      { href: "/dashboard/my-schedule", label: "Jadwal Mengajar", icon: "📅" },
      { href: "/dashboard/attendance/submit", label: "Absensi", icon: "📷" },
      { href: "/dashboard/feedback", label: "Feedback", icon: "💬" },
    ],
  },
  {
    label: "Guardian",
    roles: ["guardian"],
    items: [
      { href: "/dashboard/my-schedule", label: "Jadwal Anak", icon: "📅" },
      { href: "/dashboard/my-students", label: "Anak Saya", icon: "👶" },
      { href: "/dashboard/my-payments", label: "Pembayaran", icon: "💳" },
      { href: "/dashboard/ratings", label: "Beri Nilai", icon: "⭐" },
    ],
  },
];

const roleLabels: Record<Role, string> = {
  owner: "Owner",
  admin: "Admin Operasional",
  teacher: "Instructor",
  guardian: "Guardian",
};

function SidebarNav({ role, pathname, collapsed, onLinkClick }: { role: Role; pathname: string; collapsed: boolean; onLinkClick?: () => void }) {
  return (
    <>
      {/* Logo */}
      <div className="flex h-16 items-center justify-between border-b border-white/10 px-4">
        <Link href="/dashboard" onClick={onLinkClick} className="flex items-center gap-2">
          <span className="font-heading text-xl font-bold text-white">
            Equalizer-Stake <span className="text-[#D4AF37]">Music</span>
          </span>
          {!collapsed && (
            <span className="rounded-full bg-[#D4AF37]/10 px-2 py-0.5 text-[10px] font-semibold text-[#D4AF37]">
              {roleLabels[role]}
            </span>
          )}
        </Link>
      </div>

      {/* Navigation */}
      <nav className="mt-4 flex-1 space-y-1 overflow-y-auto px-3">
        {/* Dashboard */}
        <Link
          href="/dashboard"
          onClick={onLinkClick}
          className={`group flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200 ${
            pathname === "/dashboard"
              ? "bg-[#D4AF37]/10 text-[#D4AF37]"
              : "text-gray-400 hover:bg-white/5 hover:text-white"
          }`}
        >
          <span className="mr-3 text-lg">📊</span>
          {!collapsed && <span>Dashboard</span>}
        </Link>

        {/* Nav sections */}
        {navSections.map((section) => {
          if (!section.roles.includes(role)) return null;
          if (section.items.length === 0) return null;

          return (
            <div key={section.label} className="mt-4">
              {!collapsed && (
                <p className="mb-2 px-3 text-[10px] font-semibold uppercase tracking-[0.2em] text-gray-600">
                  {section.label}
                </p>
              )}
              {section.items.map((item) => {
                const isActive = pathname === item.href;
                return (
                  <Link
                    key={item.href}
                    href={item.href}
                    onClick={onLinkClick}
                    className={`group flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200 ${
                      isActive
                        ? "bg-[#D4AF37]/10 text-[#D4AF37]"
                        : "text-gray-400 hover:bg-white/5 hover:text-white"
                    }`}
                    title={collapsed ? item.label : undefined}
                  >
                    <span className="mr-3 text-lg">{item.icon}</span>
                    {!collapsed && <span>{item.label}</span>}
                    {isActive && !collapsed && (
                      <span className="ml-auto h-1.5 w-1.5 rounded-full bg-[#D4AF37]" />
                    )}
                  </Link>
                );
              })}
            </div>
          );
        })}

        {/* Notifications */}
        <div className="mt-4">
          {!collapsed && (
            <p className="mb-2 px-3 text-[10px] font-semibold uppercase tracking-[0.2em] text-gray-600">
              Lainnya
            </p>
          )}
          <Link
            href="/dashboard/notifications"
            onClick={onLinkClick}
            className={`group flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200 ${
              pathname === "/dashboard/notifications"
                ? "bg-[#D4AF37]/10 text-[#D4AF37]"
                : "text-gray-400 hover:bg-white/5 hover:text-white"
            }`}
            title={collapsed ? "Notifikasi" : undefined}
          >
            <span className="mr-3 text-lg">🔔</span>
            {!collapsed && <span>Notifikasi</span>}
          </Link>
        </div>
      </nav>

      {/* Footer */}
      <div className="border-t border-white/10 p-3">
        <Link
          href="/dashboard/profile"
          onClick={onLinkClick}
          className={`group flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200 ${
            pathname === "/dashboard/profile"
              ? "bg-[#D4AF37]/10 text-[#D4AF37]"
              : "text-gray-400 hover:bg-white/5 hover:text-white"
          }`}
          title={collapsed ? "Profil Saya" : undefined}
        >
          <span className="mr-3 text-lg">👤</span>
          {!collapsed && <span>Profil Saya</span>}
        </Link>

        <button
          onClick={() => {
            localStorage.removeItem("access_token");
            localStorage.removeItem("refresh_token");
            localStorage.removeItem("role");
            window.location.href = "/login";
          }}
          className="mt-1 flex w-full items-center rounded-lg px-3 py-2.5 text-sm font-medium text-gray-500 transition-all duration-200 hover:bg-red-500/10 hover:text-red-400"
          title={collapsed ? "Keluar" : undefined}
        >
          <span className="mr-3 text-lg">🚪</span>
          {!collapsed && <span>Keluar</span>}
        </button>
      </div>
    </>
  );
}

export default function Sidebar() {
  const pathname = usePathname();
  const [role, setRole] = useState<Role | null>(null);
  const [collapsed, setCollapsed] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);

  useEffect(() => {
    setRole(localStorage.getItem("role") as Role | null);
  }, []);

  const currentRole: Role = role ?? "admin";

  return (
    <>
      {/* Mobile hamburger button */}
      <button
        onClick={() => setMobileOpen(true)}
        className="fixed left-4 top-4 z-50 rounded-lg bg-[#0F172A] p-2 text-white shadow-lg md:hidden"
        aria-label="Buka menu"
      >
        <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
        </svg>
      </button>

      {/* Mobile overlay */}
      {mobileOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/50 md:hidden"
          onClick={() => setMobileOpen(false)}
        />
      )}

      {/* Mobile drawer */}
      <aside
        className={`fixed inset-y-0 left-0 z-50 flex w-64 flex-col bg-[#0F172A] transition-transform duration-300 md:hidden ${
          mobileOpen ? "translate-x-0" : "-translate-x-full"
        }`}
      >
        <SidebarNav role={currentRole} pathname={pathname} collapsed={false} onLinkClick={() => setMobileOpen(false)} />
      </aside>

      {/* Desktop sidebar */}
      <aside
        className={`hidden flex-shrink-0 flex-col border-r border-white/10 bg-[#0F172A] transition-all duration-300 md:flex ${
          collapsed ? "w-[68px]" : "w-64"
        }`}
      >
        <div className="flex h-16 items-center justify-between border-b border-white/10 px-4">
          <Link href="/dashboard" className="flex items-center gap-2">
            <span className="font-heading text-xl font-bold text-white">
              Equalizer-Stake <span className="text-[#D4AF37]">Music</span>
            </span>
            {!collapsed && (
              <span className="rounded-full bg-[#D4AF37]/10 px-2 py-0.5 text-[10px] font-semibold text-[#D4AF37]">
                {roleLabels[currentRole]}
              </span>
            )}
          </Link>
          <button
            onClick={() => setCollapsed(!collapsed)}
            className="rounded-lg p-1.5 text-gray-500 transition-colors hover:bg-white/10 hover:text-white"
            aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          >
            {collapsed ? "→" : "←"}
          </button>
        </div>

        <nav className="mt-4 flex-1 space-y-1 overflow-y-auto px-3">
          <Link
            href="/dashboard"
            className={`group flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200 ${
              pathname === "/dashboard"
                ? "bg-[#D4AF37]/10 text-[#D4AF37]"
                : "text-gray-400 hover:bg-white/5 hover:text-white"
            }`}
          >
            <span className="mr-3 text-lg">📊</span>
            {!collapsed && <span>Dashboard</span>}
          </Link>

          {navSections.map((section) => {
            if (!section.roles.includes(currentRole)) return null;
            if (section.items.length === 0) return null;

            return (
              <div key={section.label} className="mt-4">
                {!collapsed && (
                  <p className="mb-2 px-3 text-[10px] font-semibold uppercase tracking-[0.2em] text-gray-600">
                    {section.label}
                  </p>
                )}
                {section.items.map((item) => {
                  const isActive = pathname === item.href;
                  return (
                    <Link
                      key={item.href}
                      href={item.href}
                      className={`group flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200 ${
                        isActive
                          ? "bg-[#D4AF37]/10 text-[#D4AF37]"
                          : "text-gray-400 hover:bg-white/5 hover:text-white"
                      }`}
                      title={collapsed ? item.label : undefined}
                    >
                      <span className="mr-3 text-lg">{item.icon}</span>
                      {!collapsed && <span>{item.label}</span>}
                      {isActive && !collapsed && (
                        <span className="ml-auto h-1.5 w-1.5 rounded-full bg-[#D4AF37]" />
                      )}
                    </Link>
                  );
                })}
              </div>
            );
          })}

          <div className="mt-4">
            {!collapsed && (
              <p className="mb-2 px-3 text-[10px] font-semibold uppercase tracking-[0.2em] text-gray-600">
                Lainnya
              </p>
            )}
            <Link
              href="/dashboard/notifications"
              className={`group flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200 ${
                pathname === "/dashboard/notifications"
                  ? "bg-[#D4AF37]/10 text-[#D4AF37]"
                  : "text-gray-400 hover:bg-white/5 hover:text-white"
              }`}
              title={collapsed ? "Notifikasi" : undefined}
            >
              <span className="mr-3 text-lg">🔔</span>
              {!collapsed && <span>Notifikasi</span>}
            </Link>
          </div>
        </nav>

        <div className="border-t border-white/10 p-3">
          <Link
            href="/dashboard/profile"
            className={`group flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200 ${
              pathname === "/dashboard/profile"
                ? "bg-[#D4AF37]/10 text-[#D4AF37]"
                : "text-gray-400 hover:bg-white/5 hover:text-white"
            }`}
            title={collapsed ? "Profil Saya" : undefined}
          >
            <span className="mr-3 text-lg">👤</span>
            {!collapsed && <span>Profil Saya</span>}
          </Link>

          <button
            onClick={() => {
              localStorage.removeItem("access_token");
              localStorage.removeItem("refresh_token");
              localStorage.removeItem("role");
              window.location.href = "/login";
            }}
            className="mt-1 flex w-full items-center rounded-lg px-3 py-2.5 text-sm font-medium text-gray-500 transition-all duration-200 hover:bg-red-500/10 hover:text-red-400"
            title={collapsed ? "Keluar" : undefined}
          >
            <span className="mr-3 text-lg">🚪</span>
            {!collapsed && <span>Keluar</span>}
          </button>
        </div>
      </aside>
    </>
  );
}
