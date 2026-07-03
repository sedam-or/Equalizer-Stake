"use client";

import BlurText from "@/components/ui/BlurText";
import GlowButton from "@/components/ui/GlowButton";

const packages = [
  { name: "Paket Basic", price: "Rp 300.000", period: "/bulan", features: ["1x pertemuan/minggu", "30 menit/sesi", "1 instrumen", "Monitoring online"], popular: false },
  { name: "Paket Standard", price: "Rp 500.000", period: "/bulan", features: ["2x pertemuan/minggu", "45 menit/sesi", "1 instrumen", "Monitoring online", "Free studio rehearsal"], popular: true },
  { name: "Paket Premium", price: "Rp 800.000", period: "/bulan", features: ["3x pertemuan/minggu", "60 menit/sesi", "2 instrumen", "Monitoring online", "Free studio rehearsal", "Recording session"], popular: false },
];

export default function PaketHargaSection() {
  return (
    <section id="harga" className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl text-center">
        <BlurText text="Paket & Harga" delay={120} className="text-3xl font-bold tracking-tight text-white md:text-4xl" />
        <BlurText text="Pilih paket yang sesuai kebutuhanmu" delay={100} className="mt-3 text-base text-gray-400 md:text-lg" />
        <div className="mt-12 grid grid-cols-1 gap-6 md:grid-cols-3">
          {packages.map((pkg, i) => (
            <div key={i} className={`rounded-2xl border p-6 text-center transition-all duration-300 ${pkg.popular ? "border-[#2563EB] bg-[#2563EB]/5 shadow-[0_0_30px_rgba(8,203,0,0.15)]" : "border-[#CBD5E1]/50 bg-transparent hover:border-[#2563EB]/50"}`}>
              {pkg.popular && <span className="mb-4 inline-block rounded-full bg-[#2563EB] px-3 py-1 text-xs font-bold text-black">POPULER</span>}
              <h3 className="text-xl font-bold text-white">{pkg.name}</h3>
              <div className="mt-3">
                <span className="text-3xl font-bold text-[#2563EB]">{pkg.price}</span>
                <span className="text-sm text-gray-400">{pkg.period}</span>
              </div>
              <ul className="mt-6 space-y-2 text-left">
                {pkg.features.map((f, j) => (
                  <li key={j} className="flex items-center gap-2 text-sm text-gray-300">
                    <span className="text-[#2563EB]">✓</span> {f}
                  </li>
                ))}
              </ul>
              <div className="mt-6">
                <GlowButton href="/login" variant={pkg.popular ? "primary" : "secondary"} className="w-full text-sm">Pilih Paket</GlowButton>
              <div className="mt-3">
                <GlowButton href="/register" variant={pkg.popular ? "primary" : "secondary"} className="w-full text-sm">Daftar</GlowButton>
              </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
