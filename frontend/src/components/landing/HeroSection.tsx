"use client";

import { useEffect, useState } from "react";
import dynamic from "next/dynamic";
import BlurText from "@/components/ui/BlurText";
import ShinyText from "@/components/ui/ShinyText";
import GlowButton from "@/components/ui/GlowButton";

const FloatingLines = dynamic(() => import("@/components/ui/FloatingLines"), { ssr: false });
const SplashCursor = dynamic(() => import("@/components/ui/SplashCursor"), { ssr: false });

export default function HeroSection() {
  return (
    <section className="relative flex h-screen items-center justify-center overflow-hidden">
      {/* FloatingLines background — hero only */}
      <div className="absolute inset-0 z-0" aria-hidden="true">
        <FloatingLines
          linesGradient={["#08CB00", "#253900"]}
          enabledWaves={["top", "middle", "bottom"]}
          lineCount={[8, 12, 16]}
          lineDistance={[10, 8, 5]}
          animationSpeed={0.8}
          interactive={true}
          parallax={true}
          bendRadius={5.0}
          bendStrength={-0.5}
          mixBlendMode="screen"
        />
      </div>

      {/* SplashCursor — fluid effect on mouse */}
      <SplashCursor COLOR="#08CB00" RAINBOW_MODE={false} />

      {/* Content */}
      <div className="relative z-10 w-full max-w-[780px] px-6 text-center">
        <BlurText text="BMD Music School" delay={100} className="mb-6 text-sm font-semibold uppercase tracking-[0.2em] text-[#08CB00] md:text-base" />

        <BlurText text="Sistem Penjadwalan Kursus Musik & Reservasi Studio" delay={80} className="mb-6 text-4xl font-bold leading-[1.15] tracking-tight text-white md:text-[3.5rem]" />

        <p className="mb-10 text-lg leading-relaxed text-gray-400 md:text-xl">
          Belajar{" "}
          <ShinyText text="Gitar" color="#EEEEEE" shineColor="#08CB00" speed={3} className="font-semibold" />
          {" "}<RotatingText texts={["Piano", "Vokal", "Drum", "Bass", "Keyboard"]} />{" "}
          jadi lebih mudah
        </p>

        {/* Glow buttons */}
        <div className="flex flex-wrap items-center justify-center gap-4">
          <GlowButton href="#layanan" variant="primary">Mulai Sekarang</GlowButton>
          <GlowButton href="/login" variant="secondary">Masuk</GlowButton>
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
  return <span className="font-semibold text-[#08CB00]">{texts[index]}</span>;
}
