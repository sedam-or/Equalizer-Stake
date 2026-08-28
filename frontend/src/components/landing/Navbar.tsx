"use client";

import { useEffect, useState } from "react";

export default function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  const [onDark, setOnDark] = useState(true);
  const [mobileOpen, setMobileOpen] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setScrolled(window.scrollY > 50);
    };

    const sections = document.querySelectorAll("[data-theme]");
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setOnDark(entry.target.getAttribute("data-theme") === "dark");
          }
        });
      },
      { threshold: 0.3 }
    );

    sections.forEach((s) => observer.observe(s));
    window.addEventListener("scroll", handleScroll, { passive: true });

    return () => {
      observer.disconnect();
      window.removeEventListener("scroll", handleScroll);
    };
  }, []);

  // Close mobile menu on scroll
  useEffect(() => {
    if (mobileOpen) {
      const close = () => setMobileOpen(false);
      window.addEventListener("scroll", close, { once: true });
      return () => window.removeEventListener("scroll", close);
    }
  }, [mobileOpen]);

  return (
    <>
      <header
        className={`fixed left-0 right-0 z-50 transition-all duration-500 ${
          scrolled
            ? onDark
              ? "border-b border-white/10 bg-[#0F172A]/95 backdrop-blur-md"
              : "border-b border-gray-200 bg-white/95 backdrop-blur-md"
            : "bg-transparent"
        }`}
      >
        <div className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4 md:px-8">
          {/* Logo */}
          <a
            href="/"
            className={`font-heading text-xl font-bold tracking-tight transition-colors duration-300 ${
              onDark ? "text-white" : "text-[#0F172A]"
            }`}
          >
            Equalizer-Stake <span className="text-[#D4AF37]">Music</span>
          </a>

          {/* Desktop nav links */}
          <nav className="hidden items-center gap-8 md:flex">
            <a
              href="#instrumen"
              className={`text-sm transition-colors duration-300 hover:text-[#D4AF37] ${
                onDark ? "text-gray-400" : "text-gray-500"
              }`}
            >
              Program
            </a>
            <a
              href="#tentang"
              className={`text-sm transition-colors duration-300 hover:text-[#D4AF37] ${
                onDark ? "text-gray-400" : "text-gray-500"
              }`}
            >
              Tentang
            </a>
            <a
              href="/login"
              className={`text-sm transition-colors duration-300 hover:text-[#D4AF37] ${
                onDark ? "text-gray-400" : "text-gray-500"
              }`}
            >
              Login
            </a>
            <a
              href="/register-course"
              className={`rounded-lg border px-5 py-2 text-sm font-semibold transition-all duration-300 ${
                onDark
                  ? "border-[#D4AF37]/30 text-[#D4AF37] hover:bg-[#D4AF37]/10"
                  : "border-[#2563EB] text-[#2563EB] hover:bg-[#2563EB]/5"
              }`}
            >
              Daftar
            </a>
          </nav>

          {/* Mobile hamburger */}
          <button
            onClick={() => setMobileOpen(!mobileOpen)}
            className={`flex flex-col gap-1.5 md:hidden ${onDark ? "text-white" : "text-[#0F172A]"}`}
            aria-label="Menu"
          >
            <span
              className={`h-px w-6 transition-all duration-300 ${
                mobileOpen ? "translate-y-1 rotate-45" : ""
              } ${onDark ? "bg-white" : "bg-[#0F172A]"}`}
            />
            <span
              className={`h-px w-4 transition-all duration-300 ${
                mobileOpen ? "opacity-0" : ""
              } ${onDark ? "bg-white" : "bg-[#0F172A]"}`}
            />
            <span
              className={`h-px w-6 transition-all duration-300 ${
                mobileOpen ? "-translate-y-1 -rotate-45" : ""
              } ${onDark ? "bg-white" : "bg-[#0F172A]"}`}
            />
          </button>
        </div>
      </header>

      {/* Mobile menu overlay */}
      {mobileOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/50 md:hidden"
          onClick={() => setMobileOpen(false)}
        />
      )}

      {/* Mobile menu drawer */}
      <nav
        className={`fixed inset-y-0 right-0 z-50 flex w-64 flex-col bg-[#0F172A] transition-transform duration-300 md:hidden ${
          mobileOpen ? "translate-x-0" : "translate-x-full"
        }`}
      >
        <div className="flex h-16 items-center justify-between border-b border-white/10 px-6">
          <span className="font-heading text-lg font-bold text-white">
            Equalizer-Stake <span className="text-[#D4AF37]">Music</span>
          </span>
          <button
            onClick={() => setMobileOpen(false)}
            className="text-gray-400 hover:text-white"
            aria-label="Tutup menu"
          >
            ✕
          </button>
        </div>

        <div className="flex-1 space-y-1 px-4 py-6">
          <a
            href="#instrumen"
            onClick={() => setMobileOpen(false)}
            className="block rounded-lg px-4 py-3 text-sm font-medium text-gray-300 transition-colors hover:bg-white/5 hover:text-white"
          >
            Program Kursus
          </a>
          <a
            href="#tentang"
            onClick={() => setMobileOpen(false)}
            className="block rounded-lg px-4 py-3 text-sm font-medium text-gray-300 transition-colors hover:bg-white/5 hover:text-white"
          >
            Tentang Equalizer-Stake
          </a>
          <a
            href="/login"
            onClick={() => setMobileOpen(false)}
            className="block rounded-lg px-4 py-3 text-sm font-medium text-gray-300 transition-colors hover:bg-white/5 hover:text-white"
          >
            Login
          </a>
        </div>

        <div className="border-t border-white/10 p-4">
          <a
            href="/register-course"
            onClick={() => setMobileOpen(false)}
            className="block w-full rounded-lg border border-[#D4AF37]/30 bg-[#D4AF37]/10 px-4 py-3 text-center text-sm font-semibold text-[#D4AF37] transition-all hover:bg-[#D4AF37]/20"
          >
            Daftar Kursus
          </a>
        </div>
      </nav>
    </>
  );
}
