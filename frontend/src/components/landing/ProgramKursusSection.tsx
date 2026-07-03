"use client";

import BlurText from "@/components/ui/BlurText";

const programs = [
  { name: "Gitar", desc: "Akustik & Elektrik", level: "Pemula - Mahir", price: "Rp 350.000/bulan", icon: "🎸" },
  { name: "Piano", desc: "Klasik & Modern", level: "Pemula - Mahir", price: "Rp 400.000/bulan", icon: "🎹" },
  { name: "Vokal", desc: "Teknik Bernyanyi", level: "Pemula - Mahir", price: "Rp 300.000/bulan", icon: "🎤" },
  { name: "Drum", desc: "Ritme & Teknik", level: "Pemula - Mahir", price: "Rp 350.000/bulan", icon: "🥁" },
  { name: "Bass", desc: "Groove & Feel", level: "Pemula - Mahir", price: "Rp 350.000/bulan", icon: "🎸" },
  { name: "Keyboard", desc: "Pop & Worship", level: "Pemula - Mahir", price: "Rp 375.000/bulan", icon: "🎹" },
];

export default function ProgramKursusSection() {
  return (
    <section id="program" className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl text-center">
        <BlurText text="Program Kursus" delay={120} className="text-3xl font-bold tracking-tight text-white md:text-4xl" />
        <BlurText text="Pilih instrumen yang ingin kamu pelajari" delay={100} className="mt-3 text-base text-gray-400 md:text-lg" />
        <div className="mt-12 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {programs.map((p, i) => (
            <div key={i} className="group rounded-2xl border border-[#CBD5E1]/50 bg-transparent p-6 text-left transition-all duration-300 hover:border-[#2563EB] hover:bg-[#2563EB]/5 hover:shadow-[0_0_30px_rgba(8,203,0,0.1)]">
              <div className="text-3xl">{p.icon}</div>
              <h3 className="mt-3 text-xl font-bold text-white">{p.name}</h3>
              <p className="mt-1 text-sm text-gray-400">{p.desc}</p>
              <p className="mt-1 text-xs text-gray-500">{p.level}</p>
              <div className="mt-4 flex items-center justify-between">
                <span className="text-lg font-bold text-[#2563EB]">{p.price}</span>
                <a href="/login" className="rounded-full border border-[#2563EB]/50 px-4 py-1.5 text-xs font-semibold text-[#2563EB] transition-all hover:bg-[#2563EB] hover:text-black">Daftar</a>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
