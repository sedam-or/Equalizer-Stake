"use client";

import { useEffect, useState } from "react";

export default function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 50);
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled
          ? "bg-white/90 backdrop-blur-md shadow-sm border-b border-gray-100"
          : "bg-transparent"
      }`}
    >
      <nav className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4 md:px-8">
        {/* Logo */}
        <a href="/" className="font-heading text-xl font-extrabold tracking-tight text-gray-900">
          BMD
        </a>

        {/* Desktop Nav */}
        <div className="hidden items-center gap-8 md:flex">
          <a href="#instrumen" className="text-sm font-medium text-gray-600 transition-colors hover:text-[#2563EB]">
            Instrumen
          </a>
          <a href="#tentang" className="text-sm font-medium text-gray-600 transition-colors hover:text-[#2563EB]">
            Tentang
          </a>
          <a href="#harga" className="text-sm font-medium text-gray-600 transition-colors hover:text-[#2563EB]">
            Harga
          </a>
          <a href="#kontak" className="text-sm font-medium text-gray-600 transition-colors hover:text-[#2563EB]">
            Kontak
          </a>
        </div>

        {/* Desktop CTA */}
        <div className="hidden items-center gap-4 md:flex">
          <a href="/login" className="text-sm font-medium text-gray-600 transition-colors hover:text-[#2563EB]">
            Masuk
          </a>
          <a
            href="/register-course"
            className="rounded-lg bg-[#2563EB] px-5 py-2.5 text-sm font-semibold text-white transition-all hover:bg-[#1D4ED8]"
          >
            Daftar Kursus
          </a>
        </div>

        {/* Mobile Hamburger */}
        <button
          onClick={() => setMobileOpen(!mobileOpen)}
          className="flex h-10 w-10 items-center justify-center text-gray-700 md:hidden"
          aria-label="Menu"
        >
          {mobileOpen ? (
            <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          ) : (
            <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          )}
        </button>
      </nav>

      {/* Mobile Menu */}
      {mobileOpen && (
        <div className="border-t border-gray-100 bg-white px-6 py-4 md:hidden">
          <div className="flex flex-col gap-4">
            <a href="#instrumen" className="text-sm font-medium text-gray-600" onClick={() => setMobileOpen(false)}>
              Instrumen
            </a>
            <a href="#tentang" className="text-sm font-medium text-gray-600" onClick={() => setMobileOpen(false)}>
              Tentang
            </a>
            <a href="#harga" className="text-sm font-medium text-gray-600" onClick={() => setMobileOpen(false)}>
              Harga
            </a>
            <a href="#kontak" className="text-sm font-medium text-gray-600" onClick={() => setMobileOpen(false)}>
              Kontak
            </a>
            <hr className="border-gray-100" />
            <a href="/login" className="text-sm font-medium text-gray-600" onClick={() => setMobileOpen(false)}>
              Masuk
            </a>
            <a
              href="/register-course"
              className="rounded-lg bg-[#2563EB] px-5 py-2.5 text-center text-sm font-semibold text-white"
              onClick={() => setMobileOpen(false)}
            >
              Daftar Kursus
            </a>
          </div>
        </div>
      )}
    </header>
  );
}
