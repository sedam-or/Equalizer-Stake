"use client";

import BlurText from "@/components/ui/BlurText";

const studios = [
  { name: "Studio Gitar", desc: "Gitar akustik & elektrik, amplifier Fender" },
  { name: "Studio Piano", desc: "Yamaha Grand Piano + Digital Piano" },
  { name: "Studio Vokal", desc: "Booth rekaman + microphone condenser" },
  { name: "Studio Drum", desc: "Pearl Export + Zildjian cymbals" },
  { name: "Ruang Rekaman", desc: "Recording booth + mixing console" },
  { name: "Ruang Latihan", desc: "Full band setup untuk jam session" },
];

export default function StudioTourSection() {
  return (
    <section id="studio" className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl text-center">
        <BlurText text="Studio Kami" delay={120} className="text-3xl font-bold tracking-tight text-white md:text-4xl" />
        <BlurText text="Fasilitas profesional untuk pengalaman belajar terbaik" delay={100} className="mt-3 text-base text-gray-400 md:text-lg" />
        <div className="mt-12 flex gap-4 overflow-x-auto pb-4 scrollbar-hide">
          {studios.map((s, i) => (
            <div key={i} className="flex h-56 w-72 flex-shrink-0 flex-col items-center justify-center rounded-2xl border border-[#253900]/50 bg-transparent p-6 text-center transition-all duration-300 hover:border-[#08CB00]">
              <h3 className="text-lg font-bold text-white">{s.name}</h3>
              <p className="mt-2 text-sm text-gray-400">{s.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
