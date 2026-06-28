"use client";

import Image from "next/image";
import BlurText from "@/components/ui/BlurText";

const studios = [
  { name: "Studio Gitar", desc: "Gitar akustik & elektrik, amplifier Fender", img: "/images/studio-1.jpg" },
  { name: "Studio Piano", desc: "Yamaha Grand Piano + Digital Piano", img: "/images/studio-2.jpg" },
  { name: "Studio Vokal", desc: "Booth rekaman + microphone condenser", img: "/images/studio-3.jpg" },
  { name: "Studio Drum", desc: "Pearl Export + Zildjian cymbals", img: "/images/studio-4.jpg" },
  { name: "Ruang Rekaman", desc: "Recording booth + mixing console", img: "/images/studio-5.jpg" },
  { name: "Ruang Latihan", desc: "Full band setup untuk jam session", img: "/images/studio-6.jpg" },
];

export default function StudioTourSection() {
  return (
    <section id="studio" className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl text-center">
        <BlurText text="Studio Kami" delay={120} className="text-3xl font-bold tracking-tight text-white md:text-4xl" />
        <BlurText text="Fasilitas profesional untuk pengalaman belajar terbaik" delay={100} className="mt-3 text-base text-gray-400 md:text-lg" />
        <div className="mt-12 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {studios.map((s, i) => (
            <div key={i} className="group overflow-hidden rounded-2xl border border-[#253900]/50 bg-transparent transition-all duration-300 hover:border-[#08CB00]">
              <div className="relative h-48 w-full overflow-hidden">
                <Image
                  src={s.img}
                  alt={s.name}
                  fill
                  className="object-cover transition-transform duration-500 group-hover:scale-110"
                  sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw"
                />
                <div className="absolute inset-0 bg-gradient-to-t from-black/60 to-transparent" />
              </div>
              <div className="p-4 text-left">
                <h3 className="text-lg font-bold text-white">{s.name}</h3>
                <p className="mt-1 text-sm text-gray-400">{s.desc}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
