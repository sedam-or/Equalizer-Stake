"use client";

import BlurText from "@/components/ui/BlurText";

export default function StudioSection() {
  const studios = ["Studio Gitar", "Studio Piano", "Studio Vokal", "Studio Drum", "Ruang Rekaman", "Ruang Latihan"];
  return (
    <section id="studio" className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl text-center">
        <BlurText text="Studio Kami" delay={120} className="text-3xl font-bold tracking-tight text-white md:text-4xl" />
        <BlurText text="Fasilitas terbaik untuk belajar musik" delay={100} className="mt-3 text-base text-gray-400 md:text-lg" />
        <div className="mt-12 flex gap-4 overflow-x-auto pb-4 scrollbar-hide">
          {studios.map((name, i) => (
            <div key={i} className="flex h-48 w-64 flex-shrink-0 items-center justify-center rounded-2xl border border-[#CBD5E1]/50 bg-transparent text-base font-semibold text-gray-400 transition-all duration-300 hover:border-[#2563EB] hover:text-[#2563EB]">{name}</div>
          ))}
        </div>
      </div>
    </section>
  );
}
