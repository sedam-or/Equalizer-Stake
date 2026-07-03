"use client";

import { useEffect, useState } from "react";
import dynamic from "next/dynamic";
import BlurText from "@/components/ui/BlurText";

const Hyperspeed = dynamic(() => import("@/components/ui/Hyperspeed"), { ssr: false });

const highwayPreset = {
  distortion: "turbulentDistortion",
  length: 400,
  roadWidth: 10,
  islandWidth: 2,
  lanesPerRoad: 3,
  fov: 90,
  fovSpeedUp: 150,
  speedUp: 2,
  carLightsFade: 0.4,
  totalSideLightSticks: 20,
  lightPairsPerRoadWay: 40,
  shoulderLinesWidthPercentage: 0.05,
  brokenLinesWidthPercentage: 0.1,
  brokenLinesLengthPercentage: 0.5,
  lightStickWidth: [0.12, 0.5] as [number, number],
  lightStickHeight: [1.3, 1.7] as [number, number],
  movingAwaySpeed: [60, 80] as [number, number],
  movingCloserSpeed: [-120, -160] as [number, number],
  carLightsLength: [400 * 0.03, 400 * 0.2] as [number, number],
  carLightsRadius: [0.05, 0.14] as [number, number],
  carWidthPercentage: [0.3, 0.5] as [number, number],
  carShiftX: [-0.8, 0.8] as [number, number],
  carFloorSeparation: [0, 5] as [number, number],
  colors: {
    roadColor: 0x080808,
    islandColor: 0x0a0a0a,
    background: 0x000000,
    shoulderLines: 0x08cb00,
    brokenLines: 0x08cb00,
    leftCars: [0x08cb00, 0x253900, 0x0aad00],
    rightCars: [0x03b3c3, 0x0e5ea5, 0x324555],
    sticks: 0x08cb00,
  },
};

export default function HeroSection() {
  return (
    <section className="relative flex h-screen items-center justify-center overflow-hidden px-6">
      {/* Hyperspeed highway background */}
      <div className="absolute inset-0 z-0" aria-hidden="true">
        <Hyperspeed effectOptions={highwayPreset} />
      </div>

      {/* Content — forced center */}
      <div className="relative z-10 mx-auto w-full max-w-[780px] text-center">
        <BlurText text="BMD Music School" delay={100} className="mb-6 text-sm font-semibold uppercase tracking-[0.2em] text-[#2563EB] md:text-base" />

        <BlurText text="Sistem Penjadwalan Kursus Musik & Reservasi Studio" delay={80} className="mb-6 text-4xl font-bold leading-[1.15] tracking-tight text-white md:text-[3.5rem]" />

        <p className="mb-10 text-lg leading-relaxed text-gray-400 md:text-xl">
          Belajar{" "}
          <span className="font-semibold text-[#2563EB]">
            <RotatingText texts={["Gitar", "Piano", "Vokal", "Drum", "Bass", "Keyboard"]} />
          </span>{" "}
          jadi lebih mudah
        </p>

        <div className="flex flex-wrap items-center justify-center gap-4">
          <a href="#layanan" className="rounded-full bg-[#2563EB] px-8 py-3.5 text-base font-semibold text-black transition-all duration-300 hover:bg-[#0aad00]">Mulai Sekarang</a>
          <a href="/login" className="rounded-full border-2 border-[#2563EB] px-8 py-3.5 text-base font-semibold text-[#2563EB] transition-all duration-300 hover:bg-[#2563EB] hover:text-black">Masuk</a>
        </div>
      </div>
    </section>
  );
}

function RotatingText({ texts }: { texts: string[] }) {
  const [index, setIndex] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setIndex((p) => (p + 1) % texts.length), 2000);
    return () => clearInterval(id);
  }, [texts.length]);
  return <span className="font-semibold text-[#2563EB]">{texts[index]}</span>;
}
