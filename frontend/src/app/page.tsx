"use client";

import { useEffect, useState } from "react";
import AnimatedLines from "@/components/ui/AnimatedLines";

export default function LandingPage() {
  return (
    <div className="relative min-h-screen bg-[#000000] text-white">
      <AnimatedLines color="#08CB00" opacity={0.12} lineCount={14} speed={0.4} />
      <div className="relative z-10">
        <Navbar />
        <Hero />
        <Layanan />
        <CaraKerja />
        <StudioSection />
        <Testimonial />
        <CtaSection />
        <Footer />
      </div>
    </div>
  );
}

/* ── Navbar: fixed top, transparent, blur on scroll ── */
function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 50);
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled ? "bg-black/70 backdrop-blur-md shadow-lg" : "bg-transparent"
      }`}
    >
      <nav className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4 md:px-8">
        {/* Logo */}
        <a href="#" className="text-xl font-bold tracking-tight text-[#08CB00]">
          BMD MUSIC
        </a>

        {/* Desktop nav */}
        <div className="hidden items-center gap-6 md:flex">
          <a href="#layanan" className="text-sm font-medium text-gray-300 transition-colors hover:text-[#08CB00]">
            Layanan
          </a>
          <a href="#studio" className="text-sm font-medium text-gray-300 transition-colors hover:text-[#08CB00]">
            Studio
          </a>
          <a href="#hubungi" className="text-sm font-medium text-gray-300 transition-colors hover:text-[#08CB00]">
            Hubungi
          </a>
          <a
            href="/login"
            className="rounded-full border border-[#08CB00] px-5 py-2 text-sm font-semibold text-[#08CB00] transition-all hover:bg-[#08CB00] hover:text-black"
          >
            Masuk
          </a>
        </div>

        {/* Mobile hamburger */}
        <button className="flex h-10 w-10 items-center justify-center text-white md:hidden" aria-label="Menu">
          <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
          </svg>
        </button>
      </nav>
    </header>
  );
}

/* ── Hero: full-screen, centered content, animated lines bg ── */
function Hero() {
  return (
    <section className="flex min-h-screen flex-col items-center justify-center px-6 pt-20 md:px-8">
      <div className="mx-auto max-w-4xl text-center">
        {/* Brand */}
        <p className="text-sm font-semibold uppercase tracking-widest text-[#08CB00] md:text-base">
          BMD Music School
        </p>

        {/* Headline */}
        <h1 className="mt-4 text-4xl font-bold leading-tight tracking-tight text-white md:text-6xl lg:text-7xl">
          Sistem Penjadwalan
          <br />
          Kursus Musik
        </h1>

        {/* Sub-headline */}
        <p className="mt-4 text-lg text-gray-400 md:text-xl">
          & Reservasi Studio
        </p>

        {/* Rotating text */}
        <p className="mt-3 text-lg text-gray-500 md:text-xl">
          Belajar{" "}
          <span className="font-semibold text-[#08CB00]">
            <RotatingText texts={["Gitar", "Piano", "Vokal", "Drum", "Bass", "Keyboard"]} />
          </span>{" "}
          jadi lebih mudah
        </p>

        {/* CTA buttons */}
        <div className="mt-10 flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
          <a
            href="#layanan"
            className="inline-block rounded-full border-2 border-[#08CB00] px-8 py-3.5 text-base font-semibold text-[#08CB00] transition-all hover:bg-[#08CB00] hover:text-black"
          >
            Mulai Sekarang
          </a>
          <a
            href="/login"
            className="inline-block rounded-full bg-[#08CB00] px-8 py-3.5 text-base font-semibold text-black transition-all hover:bg-[#0aad00]"
          >
            Masuk
          </a>
        </div>
      </div>

      {/* Scroll indicator */}
      <div className="mt-auto pb-8 animate-bounce text-[#08CB00]">
        <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
        </svg>
      </div>
    </section>
  );
}

/* ── Layanan: bento grid, transparent cards ── */
function Layanan() {
  const services = [
    { title: "Jadwal Kursus", desc: "Penjadwalan otomatis tanpa konflik", icon: "📅" },
    { title: "Tagihan Otomatis", desc: "Invoice & pembayaran manual", icon: "💰" },
    { title: "Absensi Digital", desc: "GPS + face detection", icon: "📷" },
    { title: "Notifikasi", desc: "WhatsApp & email otomatis", icon: "🔔" },
    { title: "Studio Rekaman", desc: "Reservasi studio & ruang", icon: "🎙️" },
    { title: "Laporan & Audit", desc: "Dashboard owner & audit log", icon: "📊" },
  ];

  return (
    <section id="layanan" className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl">
        {/* Section header */}
        <div className="text-center">
          <h2 className="text-3xl font-bold tracking-tight text-white md:text-4xl">Layanan Kami</h2>
          <p className="mt-3 text-base text-gray-400 md:text-lg">Solusi lengkap untuk kursus musik Anda</p>
        </div>

        {/* Grid */}
        <div className="mt-12 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {services.map((s, i) => (
            <div
              key={i}
              className="group rounded-2xl border border-[#253900]/50 bg-transparent p-6 transition-all duration-300 hover:border-[#08CB00] hover:bg-[#08CB00]/5 hover:shadow-[0_0_30px_rgba(8,203,0,0.1)]"
            >
              <div className="text-3xl">{s.icon}</div>
              <h3 className="mt-4 text-lg font-semibold text-white">{s.title}</h3>
              <p className="mt-2 text-sm leading-relaxed text-gray-400">{s.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ── Cara Kerja: 3 steps ── */
function CaraKerja() {
  const steps = [
    { num: "01", title: "Daftar & Pilih Instrumen", desc: "Buat akun dan pilih kursus musik yang diinginkan" },
    { num: "02", title: "Pilih Jadwal Otomatis", desc: "Sistem memilihkan jadwal terbaik tanpa konflik" },
    { num: "03", title: "Pantau Progress Anak", desc: "Monitoring kehadiran dan pembayaran real-time" },
  ];

  return (
    <section className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="text-center">
          <h2 className="text-3xl font-bold tracking-tight text-white md:text-4xl">Mulai dalam 3 Langkah</h2>
        </div>

        <div className="mt-12 grid grid-cols-1 gap-6 md:grid-cols-3">
          {steps.map((s, i) => (
            <div key={i} className="rounded-2xl border border-[#08CB00]/30 bg-transparent p-6">
              <div className="text-4xl font-bold text-[#08CB00]">{s.num}</div>
              <h3 className="mt-4 text-lg font-semibold text-white">{s.title}</h3>
              <p className="mt-2 text-sm leading-relaxed text-gray-300">{s.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ── Studio: horizontal scroll gallery ── */
function StudioSection() {
  const studios = ["Studio Gitar", "Studio Piano", "Studio Vokal", "Studio Drum", "Ruang Rekaman", "Ruang Latihan"];

  return (
    <section id="studio" className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="text-center">
          <h2 className="text-3xl font-bold tracking-tight text-white md:text-4xl">Studio Kami</h2>
          <p className="mt-3 text-base text-gray-400 md:text-lg">Fasilitas terbaik untuk belajar musik</p>
        </div>

        <div className="mt-12 flex gap-4 overflow-x-auto pb-4 scrollbar-hide">
          {studios.map((name, i) => (
            <div
              key={i}
              className="flex h-48 w-64 flex-shrink-0 items-center justify-center rounded-2xl border border-[#253900]/50 bg-transparent text-base font-semibold text-gray-400 transition-all duration-300 hover:border-[#08CB00] hover:text-[#08CB00]"
            >
              {name}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ── Testimonial ── */
function Testimonial() {
  const items = [
    { name: "Ananda Putra", role: "Orang Tua", quote: "Sangat membantu mengatur jadwal les anak saya" },
    { name: "Rizky Pratama", role: "Guru Musik", quote: "Sistem absensi memudahkan monitoring kelas" },
    { name: "Putri Wulandari", role: "Wali Murid", quote: "Tagihan otomatis dan notifikasi sangat praktis" },
  ];

  return (
    <section className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="text-center">
          <h2 className="text-3xl font-bold tracking-tight text-white md:text-4xl">Apa Kata Mereka</h2>
        </div>

        <div className="mt-12 grid grid-cols-1 gap-6 md:grid-cols-3">
          {items.map((t, i) => (
            <div key={i} className="rounded-2xl border border-[#253900]/30 bg-transparent p-6">
              <div className="flex h-10 w-10 items-center justify-center rounded-full border border-[#08CB00]/30 text-sm font-bold text-[#08CB00]">
                {t.name[0]}
              </div>
              <blockquote className="mt-4 text-sm leading-relaxed text-gray-300">
                &ldquo;{t.quote}&rdquo;
              </blockquote>
              <div className="mt-4">
                <p className="text-sm font-semibold text-white">{t.name}</p>
                <p className="text-xs text-gray-500">{t.role}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ── CTA ── */
function CtaSection() {
  return (
    <section className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-3xl text-center">
        <h2 className="text-3xl font-bold tracking-tight text-[#08CB00] md:text-5xl">Siap Mulai?</h2>
        <p className="mt-4 text-base text-gray-400 md:text-lg">
          Daftar sekarang dan nikmati kemudahan penjadwalan kursus musik
        </p>
        <div className="mt-8">
          <a
            href="#hubungi"
            className="inline-block rounded-full border-2 border-[#08CB00] px-8 py-3.5 text-base font-semibold text-[#08CB00] transition-all hover:bg-[#08CB00] hover:text-black"
          >
            Hubungi Kami
          </a>
        </div>
      </div>
    </section>
  );
}

/* ── Footer ── */
function Footer() {
  return (
    <footer id="hubungi" className="border-t border-[#253900]/50 px-6 py-12 md:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="grid grid-cols-1 gap-8 md:grid-cols-3">
          <div>
            <h3 className="text-lg font-bold text-[#08CB00]">BMD MUSIC SCHOOL</h3>
            <p className="mt-2 text-sm text-gray-400">Sistem Penjadwalan Kursus Musik</p>
          </div>
          <div>
            <h4 className="text-sm font-semibold text-white">Kontak</h4>
            <p className="mt-2 text-sm text-gray-400">Seturan, Yogyakarta</p>
            <p className="mt-1 text-sm text-[#08CB00]">+62 xxx-xxxx-xxxx</p>
            <p className="mt-1 text-sm text-[#08CB00]">info@bmdmusic.com</p>
          </div>
          <div>
            <h4 className="text-sm font-semibold text-white">Navigasi</h4>
            <div className="mt-2 flex flex-col gap-2">
              <a href="#layanan" className="text-sm text-gray-400 hover:text-[#08CB00]">Layanan</a>
              <a href="#studio" className="text-sm text-gray-400 hover:text-[#08CB00]">Studio</a>
              <a href="/login" className="text-sm text-gray-400 hover:text-[#08CB00]">Masuk</a>
            </div>
          </div>
        </div>
        <div className="mt-8 border-t border-[#253900]/30 pt-4 text-center text-xs text-gray-500">
          © 2026 BMD Music School. All rights reserved.
        </div>
      </div>
    </footer>
  );
}

/* ── RotatingText ── */
function RotatingText({ texts }: { texts: string[] }) {
  const [index, setIndex] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setIndex((p) => (p + 1) % texts.length), 2000);
    return () => clearInterval(id);
  }, [texts.length]);
  return <span>{texts[index]}</span>;
}
