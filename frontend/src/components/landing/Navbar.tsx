"use client";

import { useEffect, useState } from "react";

export default function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  const [onDark, setOnDark] = useState(true);

  useEffect(() => {
    const handleScroll = () => {
      setScrolled(window.scrollY > 50);
    };

    // Detect dark/light sections via data-theme attribute
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

  return (
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
          BMD <span className="text-[#D4AF37]">Music</span>
        </a>

        {/* Nav links */}
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
            href="/register-course"
            className={`rounded-lg border px-5 py-2 text-sm font-semibold transition-all duration-300 ${
              onDark
                ? "border-white/20 text-white hover:border-[#D4AF37]/50 hover:text-[#D4AF37]"
                : "border-gray-300 text-[#0F172A] hover:border-[#2563EB] hover:text-[#2563EB]"
            }`}
          >
            Daftar
          </a>
        </nav>

        {/* Mobile menu button */}
        <button
          className={`flex flex-col gap-1.5 md:hidden ${onDark ? "text-white" : "text-[#0F172A]"}`}
          aria-label="Menu"
        >
          <span className={`h-px w-6 ${onDark ? "bg-white" : "bg-[#0F172A]"}`} />
          <span className={`h-px w-4 ${onDark ? "bg-white" : "bg-[#0F172A]"}`} />
        </button>
      </div>
    </header>
  );
}
