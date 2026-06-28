"use client";

import { useEffect, useState } from "react";

export default function HeroSection() {
  return (
    <section className="flex min-h-screen flex-col items-center justify-center px-6 pt-20 md:px-8">
      <div className="mx-auto max-w-4xl text-center">
        <p className="text-sm font-semibold uppercase tracking-widest text-[#08CB00] md:text-base">BMD Music School</p>
        <h1 className="mt-4 text-4xl font-bold leading-tight tracking-tight text-white md:text-6xl lg:text-7xl">
          Sistem Penjadwalan<br />Kursus Musik
        </h1>
        <p className="mt-4 text-lg text-gray-400 md:text-xl">& Reservasi Studio</p>
        <p className="mt-3 text-lg text-gray-500 md:text-xl">
          Belajar{" "}<span className="font-semibold text-[#08CB00]"><RotatingText texts={["Gitar", "Piano", "Vokal", "Drum", "Bass", "Keyboard"]} /></span>{" "}jadi lebih mudah
        </p>
        <div className="mt-10 flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
          <a href="#layanan" className="inline-block rounded-full border-2 border-[#08CB00] px-8 py-3.5 text-base font-semibold text-[#08CB00] transition-all hover:bg-[#08CB00] hover:text-black">Mulai Sekarang</a>
          <a href="/login" className="inline-block rounded-full bg-[#08CB00] px-8 py-3.5 text-base font-semibold text-black transition-all hover:bg-[#0aad00]">Masuk</a>
        </div>
      </div>
      <div className="mt-auto pb-8 animate-bounce text-[#08CB00]">
        <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" /></svg>
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
