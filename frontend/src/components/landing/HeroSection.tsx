"use client";

import { useEffect, useState } from "react";

export default function HeroSection() {
  return (
    <section className="flex h-screen items-center justify-center px-6">
      <div className="w-full max-w-[780px] text-center">
        {/* Brand */}
        <p className="mb-6 text-sm font-semibold uppercase tracking-[0.2em] text-[#08CB00] md:text-base">
          BMD Music School
        </p>

        {/* Title */}
        <h1 className="mb-6 text-4xl font-bold leading-[1.15] tracking-tight text-white md:text-[3.5rem]">
          Sistem Penjadwalan
          <br />
          Kursus Musik & Reservasi Studio
        </h1>

        {/* Subtitle */}
        <p className="mb-10 text-lg leading-relaxed text-gray-400 md:text-xl">
          Belajar{" "}
          <span className="font-semibold text-[#08CB00]">
            <RotatingText texts={["Gitar", "Piano", "Vokal", "Drum", "Bass", "Keyboard"]} />
          </span>{" "}
          jadi lebih mudah
        </p>

        {/* Buttons */}
        <div className="flex flex-wrap items-center justify-center gap-4">
          <a
            href="#layanan"
            className="rounded-full bg-[#08CB00] px-8 py-3.5 text-base font-semibold text-black transition-all duration-300 hover:bg-[#0aad00]"
          >
            Mulai Sekarang
          </a>
          <a
            href="/login"
            className="rounded-full border-2 border-[#08CB00] px-8 py-3.5 text-base font-semibold text-[#08CB00] transition-all duration-300 hover:bg-[#08CB00] hover:text-black"
          >
            Masuk
          </a>
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
  return <span>{texts[index]}</span>;
}
