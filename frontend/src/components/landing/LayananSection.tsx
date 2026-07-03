"use client";

import BlurText from "@/components/ui/BlurText";

export default function LayananSection() {
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
      <div className="mx-auto max-w-7xl text-center">
        <BlurText text="Layanan Kami" delay={120} className="text-3xl font-bold tracking-tight text-white md:text-4xl" />
        <BlurText text="Solusi lengkap untuk kursus musik Anda" delay={100} className="mt-3 text-base text-gray-400 md:text-lg" />
        <div className="mt-12 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {services.map((s, i) => (
            <div key={i} className="group rounded-2xl border border-[#CBD5E1]/50 bg-transparent p-6 text-center transition-all duration-300 hover:border-[#2563EB] hover:bg-[#2563EB]/5">
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
