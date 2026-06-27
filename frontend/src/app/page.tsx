"use client";

import { useEffect, useState } from "react";
import AnimatedLines from "@/components/ui/AnimatedLines";

export default function LandingPage() {
  return (
    <div className="relative min-h-screen bg-[#000000] text-white">
      {/* Global animated background — spans entire page */}
      <div className="pointer-events-none fixed inset-0 z-0">
        <AnimatedLines color="#08CB00" opacity={0.12} lineCount={14} speed={0.4} />
      </div>

      {/* All content overlays sit on top, fully transparent */}
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

/* ── Navbar ── */
function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 50);
    window.addEventListener("scroll", onScroll);
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <nav
      className={`fixed top-0 left-0 right-0 z-50 flex items-center justify-between px-6 py-4 transition-all duration-300 ${
        scrolled ? "bg-black/60 backdrop-blur-md" : "bg-transparent"
      }`}
    >
      <a href="#" className="text-lg font-bold text-[#08CB00]">BMD MUSIC</a>
      <div className="hidden items-center gap-8 text-sm text-gray-300 md:flex">
        <a href="#layanan" className="hover:text-[#08CB00]">Layanan</a>
        <a href="#studio" className="hover:text-[#08CB00]">Studio</a>
        <a href="#hubungi" className="hover:text-[#08CB00]">Hubungi</a>
        <a href="/login" className="rounded-full border border-[#08CB00] px-4 py-1.5 text-[#08CB00] hover:bg-[#08CB00] hover:text-black">Masuk</a>
      </div>
      <button className="text-2xl text-white md:hidden">☰</button>
    </nav>
  );
}

/* ── Hero ── */
function Hero() {
  return (
    <section className="relative flex min-h-screen flex-col items-center justify-center px-4">
      <div className="relative z-10 text-center">
        <h1 className="text-4xl font-bold tracking-tight text-[#08CB00] md:text-6xl lg:text-7xl">BMD MUSIC SCHOOL</h1>
        <p className="mt-4 text-xl text-white md:text-3xl">Sistem Penjadwalan Kursus Musik</p>
        <p className="mt-1 text-xl text-white md:text-3xl">&amp; Reservasi Studio</p>
        <p className="mt-3 text-lg text-gray-400">
          Belajar{" "}
          <span className="font-semibold text-[#08CB00]">
            <RotatingText texts={["Gitar", "Piano", "Vokal", "Drum", "Bass", "Keyboard"]} />
          </span>{" "}
          jadi lebih mudah
        </p>
        <div className="mt-10">
          <a href="#layanan" className="inline-block rounded-full border-2 border-[#08CB00] px-10 py-3.5 text-lg font-semibold text-[#08CB00] transition-all hover:bg-[#08CB00] hover:text-black">Mulai Sekarang</a>
        </div>
      </div>
      <div className="absolute bottom-8 animate-bounce text-2xl text-[#08CB00]">↓</div>
    </section>
  );
}

/* ── Layanan ── */
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
    <section id="layanan" className="relative px-4 py-24">
      <div className="relative z-10 mx-auto max-w-6xl">
        <h2 className="text-center text-3xl font-bold md:text-4xl">Layanan Kami</h2>
        <p className="mt-2 text-center text-gray-400">Solusi lengkap untuk kursus musik Anda</p>
        <div className="mt-12 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {services.map((s, i) => (
            <div key={i} className="group rounded-2xl border border-[#253900]/50 bg-transparent p-6 transition-all hover:border-[#08CB00] hover:bg-[#08CB00]/5">
              <div className="text-4xl">{s.icon}</div>
              <h3 className="mt-4 text-xl font-semibold">{s.title}</h3>
              <p className="mt-2 text-gray-400">{s.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ── Cara Kerja ── */
function CaraKerja() {
  const steps = [
    { num: "01", title: "Daftar & Pilih Instrumen", desc: "Buat akun dan pilih kursus musik yang diinginkan" },
    { num: "02", title: "Pilih Jadwal Otomatis", desc: "Sistem memilihkan jadwal terbaik tanpa konflik" },
    { num: "03", title: "Pantau Progress Anak", desc: "Monitoring kehadiran dan pembayaran real-time" },
  ];
  return (
    <section className="relative px-4 py-24">
      <div className="relative z-10 mx-auto max-w-6xl">
        <h2 className="text-center text-3xl font-bold md:text-4xl">Mulai dalam 3 Langkah</h2>
        <div className="mt-12 grid grid-cols-1 gap-6 md:grid-cols-3">
          {steps.map((s, i) => (
            <div key={i} className="rounded-2xl border border-[#08CB00]/30 bg-transparent p-6">
              <div className="text-4xl font-bold text-[#08CB00]">{s.num}</div>
              <h3 className="mt-4 text-xl font-semibold">{s.title}</h3>
              <p className="mt-2 text-gray-300">{s.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ── Studio ── */
function StudioSection() {
  const studios = ["Studio Gitar", "Studio Piano", "Studio Vokal", "Studio Drum", "Ruang Rekaman", "Ruang Latihan"];
  return (
    <section id="studio" className="relative px-4 py-24">
      <div className="relative z-10 mx-auto max-w-6xl">
        <h2 className="text-center text-3xl font-bold md:text-4xl">Studio Kami</h2>
        <p className="mt-2 text-center text-gray-400">Fasilitas terbaik untuk belajar musik</p>
        <div className="mt-12 flex gap-4 overflow-x-auto pb-4">
          {studios.map((name, i) => (
            <div key={i} className="flex h-48 w-64 flex-shrink-0 items-center justify-center rounded-2xl border border-[#253900]/50 bg-transparent text-lg font-semibold text-gray-400 transition-all hover:border-[#08CB00] hover:text-[#08CB00]">{name}</div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ── Testimonial ── */
function Testimonial() {
  const t = [
    { name: "Ananda Putra", role: "Orang Tua", quote: "Sangat membantu mengatur jadwal les anak saya" },
    { name: "Rizky Pratama", role: "Guru Musik", quote: "Sistem absensi memudahkan monitoring kelas" },
    { name: "Putri Wulandari", role: "Wali Murid", quote: "Tagihan otomatis dan notifikasi sangat praktis" },
  ];
  return (
    <section className="relative px-4 py-24">
      <div className="relative z-10 mx-auto max-w-6xl">
        <h2 className="text-center text-3xl font-bold md:text-4xl">Apa Kata Mereka</h2>
        <div className="mt-12 grid grid-cols-1 gap-6 md:grid-cols-3">
          {t.map((item, i) => (
            <div key={i} className="rounded-2xl border border-[#253900]/30 bg-transparent p-6">
              <div className="flex h-12 w-12 items-center justify-center rounded-full border border-[#08CB00]/30 text-lg font-bold text-[#08CB00]">{item.name[0]}</div>
              <p className="mt-4 text-gray-300">&ldquo;{item.quote}&rdquo;</p>
              <div className="mt-4">
                <p className="font-semibold text-white">{item.name}</p>
                <p className="text-sm text-gray-500">{item.role}</p>
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
    <section className="relative px-4 py-24">
      <div className="relative z-10 mx-auto max-w-2xl text-center">
        <h2 className="text-4xl font-bold text-[#08CB00] md:text-5xl">Siap Mulai?</h2>
        <p className="mt-4 text-lg text-gray-400">Daftar sekarang dan nikmati kemudahan penjadwalan kursus musik</p>
        <div className="mt-8">
          <a href="#hubungi" className="inline-block rounded-full border-2 border-[#08CB00] px-10 py-3.5 text-lg font-semibold text-[#08CB00] transition-all hover:bg-[#08CB00] hover:text-black">Hubungi Kami</a>
        </div>
      </div>
    </section>
  );
}

/* ── Footer ── */
function Footer() {
  return (
    <footer id="hubungi" className="border-t border-[#253900]/50 px-4 py-12">
      <div className="mx-auto max-w-6xl">
        <div className="grid grid-cols-1 gap-8 md:grid-cols-3">
          <div>
            <h3 className="text-xl font-bold text-[#08CB00]">BMD MUSIC SCHOOL</h3>
            <p className="mt-2 text-sm text-gray-400">Sistem Penjadwalan Kursus Musik</p>
          </div>
          <div>
            <h4 className="font-semibold text-white">Kontak</h4>
            <p className="mt-2 text-sm text-gray-400">Seturan, Yogyakarta</p>
            <p className="text-sm text-[#08CB00]">+62 xxx-xxxx-xxxx</p>
            <p className="text-sm text-[#08CB00]">info@bmdmusic.com</p>
          </div>
          <div>
            <h4 className="font-semibold text-white">Navigasi</h4>
            <div className="mt-2 flex flex-col gap-2 text-sm text-gray-400">
              <a href="#layanan" className="hover:text-[#08CB00]">Layanan</a>
              <a href="#studio" className="hover:text-[#08CB00]">Studio</a>
              <a href="/login" className="hover:text-[#08CB00]">Masuk</a>
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
