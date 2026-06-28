"use client";

import dynamic from "next/dynamic";
import BlurText from "@/components/ui/BlurText";

const CircularGallery = dynamic(() => import("@/components/ui/CircularGallery"), { ssr: false });

const studioItems = [
  { image: "/images/studio-1.jpg", text: "Studio Gitar" },
  { image: "/images/studio-2.jpg", text: "Studio Piano" },
  { image: "/images/studio-3.jpg", text: "Studio Vokal" },
  { image: "/images/studio-4.jpg", text: "Studio Drum" },
  { image: "/images/studio-5.jpg", text: "Ruang Rekaman" },
  { image: "/images/studio-6.jpg", text: "Ruang Latihan" },
];

export default function StudioTourSection() {
  return (
    <section id="studio" className="py-24">
      <div className="mx-auto max-w-7xl text-center px-6 md:px-8">
        <BlurText text="Studio Kami" delay={120} className="text-3xl font-bold tracking-tight text-white md:text-4xl" />
        <BlurText text="Fasilitas profesional untuk pengalaman belajar terbaik" delay={100} className="mt-3 text-base text-gray-400 md:text-lg" />
      </div>
      <div className="mt-12" style={{ height: "500px", position: "relative" }}>
        <CircularGallery
          items={studioItems}
          bend={3}
          textColor="#EEEEEE"
          borderRadius={0.05}
          scrollSpeed={2}
          scrollEase={0.02}
          font="bold 24px Inter"
        />
      </div>
    </section>
  );
}
