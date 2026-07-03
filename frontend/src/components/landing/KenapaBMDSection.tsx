"use client";

import BlurText from "@/components/ui/BlurText";

const reasons = [
  { title: "Guru Berpengalaman", desc: "Semua guru memiliki pengalaman mengajar minimal 5 tahun dan sertifikasi musik.", icon: "👨‍🏫" },
  { title: "Studio Profesional", desc: "6 studio dengan peralatan standar industri untuk pengalaman belajar terbaik.", icon: "🏠" },
  { title: "Jadwal Fleksibel", desc: "Atur jadwal sesuai keinginan. Sistem otomatis menghindari konflik.", icon: "📅" },
  { title: "Monitoring Orang Tua", desc: "Pantau kehadiran dan progress anak secara real-time melalui dashboard.", icon: "📱" },
];

export default function KenapaBMDSection() {
  return (
    <section className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl text-center">
        <BlurText text="Kenapa BMD Music?" delay={120} className="text-3xl font-bold tracking-tight text-white md:text-4xl" />
        <BlurText text="Keunggulan yang membuat kami berbeda" delay={100} className="mt-3 text-base text-gray-400 md:text-lg" />
        <div className="mt-12 grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-4">
          {reasons.map((r, i) => (
            <div key={i} className="rounded-2xl border border-[#2563EB]/20 bg-transparent p-6 text-center transition-all duration-300 hover:border-[#2563EB]/60">
              <div className="text-4xl">{r.icon}</div>
              <h3 className="mt-4 text-lg font-bold text-white">{r.title}</h3>
              <p className="mt-2 text-sm leading-relaxed text-gray-400">{r.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
