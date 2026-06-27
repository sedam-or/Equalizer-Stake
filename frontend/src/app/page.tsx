"use client";

import { useEffect, useRef, useState } from "react";

export default function LandingPage() {
  return (
    <div className="min-h-screen bg-[#000000] text-white">
      <Hero />
      <Layanan />
      <CaraKerja />
      <StudioSection />
      <Testimonial />
      <CtaSection />
      <Footer />
    </div>
  );
}

// Section 1: Hero
function Hero() {
  return (
    <section className="relative flex min-h-screen flex-col items-center justify-center overflow-hidden px-4">
      {/* Background gradient */}
      <div className="absolute inset-0 bg-gradient-to-b from-[#08CB00]/10 via-[#000000] to-[#000000]" />

      {/* Floating lines placeholder */}
      <div className="absolute inset-0 opacity-20">
        {Array.from({ length: 8 }).map((_, i) => (
          <div
            key={i}
            className="absolute h-px bg-gradient-to-r from-transparent via-[#08CB00] to-transparent"
            style={{
              top: `${10 + i * 12}%`,
              left: "-10%",
              right: "-10%",
              transform: `rotate(${-5 + i * 2}deg)`,
              animation: `float ${3 + i * 0.5}s ease-in-out infinite alternate`,
            }}
          />
        ))}
      </div>

      {/* Content */}
      <div className="relative z-10 text-center">
        <h1 className="text-4xl font-bold tracking-tight text-[#08CB00] md:text-6xl">
          BMD MUSIC SCHOOL
        </h1>
        <p className="mt-4 text-2xl font-semibold text-white md:text-4xl">
          Sistem Penjadwalan Kursus Musik
          <br />
          & Reservasi Studio
        </p>
        <p className="mt-2 text-lg text-gray-400">
          belajar{" "}
          <span className="text-[#08CB00]">
            <RotatingText texts={["Gitar", "Piano", "Vokal", "Drum", "Bass", "Keyboard"]} />
          </span>{" "}
          jadi lebih mudah
        </p>
        <div className="mt-8">
          <a
            href="#layanan"
            className="inline-block rounded-full border-2 border-[#08CB00] bg-transparent px-8 py-3 text-lg font-semibold text-[#08CB00] transition-all hover:bg-[#08CB00] hover:text-black"
          >
            Mulai Sekarang
          </a>
        </div>
      </div>

      {/* Scroll indicator */}
      <div className="absolute bottom-8 animate-bounce text-[#08CB00]">
        ↓
      </div>
    </section>
  );
}

// Section 2: Layanan (Bento Grid)
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
    <section id="layanan" className="px-4 py-24">
      <div className="mx-auto max-w-6xl">
        <h2 className="text-center text-3xl font-bold md:text-4xl">Layanan Kami</h2>
        <p className="mt-2 text-center text-gray-400">Solusi lengkap untuk kursus musik Anda</p>

        <div className="mt-12 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {services.map((s, i) => (
            <div
              key={i}
              className="group rounded-2xl border border-[#253900] bg-[#0a0a0a] p-6 transition-all hover:border-[#08CB00] hover:shadow-[0_0_20px_rgba(8,203,0,0.15)]"
            >
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

// Section 3: Cara Kerja
function CaraKerja() {
  const steps = [
    { num: "01", title: "Daftar & Pilih Instrumen", desc: "Buat akun dan pilih kursus musik yang diinginkan" },
    { num: "02", title: "Pilih Jadwal Otomatis", desc: "Sistem memilihkan jadwal terbaik tanpa konflik" },
    { num: "03", title: "Pantau Progress Anak", desc: "Monitoring kehadiran dan pembayaran real-time" },
  ];

  return (
    <section className="bg-[#253900] px-4 py-24">
      <div className="mx-auto max-w-6xl">
        <h2 className="text-center text-3xl font-bold md:text-4xl">Mulai dalam 3 Langkah</h2>

        <div className="mt-12 grid grid-cols-1 gap-6 md:grid-cols-3">
          {steps.map((s, i) => (
            <div
              key={i}
              className="rounded-2xl border border-[#08CB00]/30 bg-[#0a0a0a]/50 p-6 backdrop-blur-sm"
            >
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

// Section 4: Studio Showcase
function StudioSection() {
  const studios = ["Studio Gitar", "Studio Piano", "Studio Vokal", "Studio Drum", "Ruang Rekaman", "Ruang Latihan"];

  return (
    <section id="studio" className="px-4 py-24">
      <div className="mx-auto max-w-6xl">
        <h2 className="text-center text-3xl font-bold md:text-4xl">Studio Kami</h2>
        <p className="mt-2 text-center text-gray-400">Fasilitas terbaik untuk belajar musik</p>

        {/* Horizontal scroll gallery */}
        <div className="mt-12 flex gap-4 overflow-x-auto pb-4">
          {studios.map((name, i) => (
            <div
              key={i}
              className="flex h-48 w-64 flex-shrink-0 items-center justify-center rounded-2xl border border-[#253900] bg-[#0a0a0a] text-lg font-semibold text-gray-400 transition-all hover:border-[#08CB00] hover:text-[#08CB00]"
            >
              {name}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

// Section 5: Testimonial
function Testimonial() {
  const testimonials = [
    { name: "Ananda Putra", role: "Orang Tua", quote: "Sangat membantu mengatur jadwal les anak saya" },
    { name: "Rizky Pratama", role: "Guru Musik", quote: "Sistem absensi memudahkan monitoring kelas" },
    { name: "Putri Wulandari", role: "Wali Murid", quote: "Tagihan otomatis dan notifikasi sangat praktis" },
  ];

  return (
    <section className="bg-[#EEEEEE] px-4 py-24">
      <div className="mx-auto max-w-6xl">
        <h2 className="text-center text-3xl font-bold text-[#000000] md:text-4xl">Apa Kata Mereka</h2>

        <div className="mt-12 grid grid-cols-1 gap-6 md:grid-cols-3">
          {testimonials.map((t, i) => (
            <div key={i} className="rounded-2xl bg-white p-6 shadow-sm">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-[#08CB00]/10 text-lg font-bold text-[#08CB00]">
                {t.name[0]}
              </div>
              <p className="mt-4 text-gray-600">&ldquo;{t.quote}&rdquo;</p>
              <div className="mt-4">
                <p className="font-semibold text-[#000000]">{t.name}</p>
                <p className="text-sm text-gray-500">{t.role}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

// Section 6: CTA
function CtaSection() {
  return (
    <section className="relative overflow-hidden px-4 py-24">
      <div className="absolute inset-0 bg-gradient-to-b from-[#253900]/20 via-[#000000] to-[#000000]" />
      <div className="relative z-10 mx-auto max-w-2xl text-center">
        <h2 className="text-4xl font-bold text-[#08CB00] md:text-5xl">Siap Mulai?</h2>
        <p className="mt-4 text-lg text-gray-400">
          Daftar sekarang dan nikmati kemudahan penjadwalan kursus musik
        </p>
        <div className="mt-8">
          <a
            href="#hubungi"
            className="inline-block rounded-full border-2 border-[#08CB00] bg-transparent px-8 py-3 text-lg font-semibold text-[#08CB00] transition-all hover:bg-[#08CB00] hover:text-black"
          >
            Hubungi Kami
          </a>
        </div>
      </div>
    </section>
  );
}

// Section 7: Footer
function Footer() {
  return (
    <footer id="hubungi" className="border-t border-[#253900] bg-[#000000] px-4 py-12">
      <div className="mx-auto max-w-6xl">
        <div className="grid grid-cols-1 gap-8 md:grid-cols-3">
          <div>
            <h3 className="text-xl font-bold text-[#08CB00]">BMD MUSIC SCHOOL</h3>
            <p className="mt-2 text-sm text-gray-400">Sistem Penjadwalan Kursus Musik</p>
          </div>
          <div>
            <h4 className="font-semibold">Kontak</h4>
            <p className="mt-2 text-sm text-gray-400">Seturan, Yogyakarta</p>
            <p className="text-sm text-[#08CB00]">+62 xxx-xxxx-xxxx</p>
            <p className="text-sm text-[#08CB00]">info@bmdmusic.com</p>
          </div>
          <div>
            <h4 className="font-semibold">Navigasi</h4>
            <div className="mt-2 flex flex-col gap-2 text-sm text-gray-400">
              <a href="#layanan" className="hover:text-[#08CB00]">Layanan</a>
              <a href="#studio" className="hover:text-[#08CB00]">Studio</a>
              <a href="/login" className="hover:text-[#08CB00]">Masuk</a>
            </div>
          </div>
        </div>
        <div className="mt-8 border-t border-[#253900] pt-4 text-center text-xs text-gray-500">
          © 2026 BMD Music School. All rights reserved.
        </div>
      </div>
    </footer>
  );
}

// RotatingText component
function RotatingText({ texts }: { texts: string[] }) {
  const [index, setIndex] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      setIndex((prev) => (prev + 1) % texts.length);
    }, 2000);
    return () => clearInterval(interval);
  }, [texts.length]);

  return <span>{texts[index]}</span>;
}
