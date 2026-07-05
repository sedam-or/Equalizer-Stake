"use client";

import { useEffect, useState } from "react";

export default function Navbar() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => setScrolled(window.scrollY > 50);
    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  return (
    <header
      className={`fixed left-0 right-0 z-50 transition-all duration-500 ${
        scrolled
          ? "border-b border-white/10 bg-[#0F172A]/90 backdrop-blur-md"
          : "bg-transparent"
      }`}
    >
      <div className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4 md:px-8">
        {/* Logo */}
        <a href="/" className="font-heading text-xl font-bold tracking-tight text-white">
          BMD <span className="text-[#D4AF37]">Music</span>
        </a>

        {/* Nav links */}
        <nav className="hidden items-center gap-8 md:flex">
          <a href="#instrumen" className="text-sm text-gray-400 transition-colors duration-300 hover:text-white">
            Program
          </a>
          <a href="#tentang" className="text-sm text-gray-400 transition-colors duration-300 hover:text-white">
            Tentang
          </a>
          <a
            href="/register-course"
            className="rounded-lg border border-white/20 px-5 py-2 text-sm font-semibold text-white transition-all duration-300 hover:border-[#D4AF37]/50 hover:text-[#D4AF37]"
          >
            Daftar
          </a>
        </nav>

        {/* Mobile menu button */}
        <button className="flex flex-col gap-1.5 md:hidden" aria-label="Menu">
          <span className="h-px w-6 bg-white" />
          <span className="h-px w-4 bg-white" />
        </button>
      </div>
    </header>
  );
}
