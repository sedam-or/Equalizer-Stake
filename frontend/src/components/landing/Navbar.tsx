"use client";

import { useEffect, useState } from "react";

export default function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 50);
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${scrolled ? "bg-black/70 backdrop-blur-md shadow-lg" : "bg-transparent"}`}>
      <nav className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4 md:px-8">
        <img src="/logo.jpg" alt="BMD Music" className="h-8 w-auto" />
        <div className="hidden items-center gap-6 md:flex">
          <a href="#layanan" className="text-sm font-medium text-gray-300 transition-colors hover:text-[#2563EB]">Layanan</a>
          <a href="#studio" className="text-sm font-medium text-gray-300 transition-colors hover:text-[#2563EB]">Studio</a>
          <a href="#harga" className="text-sm font-medium text-gray-300 transition-colors hover:text-[#2563EB]">Harga</a>
          <a href="#hubungi" className="text-sm font-medium text-gray-300 transition-colors hover:text-[#2563EB]">Hubungi</a>
          <a href="/register-course" className="rounded-full bg-[#2563EB] px-5 py-2 text-sm font-semibold text-black transition-all hover:bg-[#0aad00]">Daftar Kursus</a>
          <a href="/login" className="rounded-full border border-[#2563EB] px-5 py-2 text-sm font-semibold text-[#2563EB] transition-all hover:bg-[#2563EB] hover:text-black">Masuk</a>
        </div>
        <button className="flex h-10 w-10 items-center justify-center text-white md:hidden" aria-label="Menu">
          <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" /></svg>
        </button>
      </nav>
    </header>
  );
}
