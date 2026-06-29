"use client";

import BlurText from "@/components/ui/BlurText";
import GlowButton from "@/components/ui/GlowButton";

export default function CtaSection() {
  return (
    <section className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-3xl text-center">
        <BlurText text="Siap Belajar Musik?" delay={120} className="text-3xl font-bold tracking-tight text-[#08CB00] md:text-5xl" />
        <BlurText text="Daftar sekarang dan mulai perjalanan musikmu bersama BMD Music School" delay={100} className="mt-4 text-base text-gray-400 md:text-lg" />
        <div className="mt-8 flex flex-wrap items-center justify-center gap-4">
          <GlowButton href="/register" variant="primary">Daftar Sekarang</GlowButton>
          <GlowButton href="#hubungi" variant="secondary">Hubungi Kami</GlowButton>
        </div>
      </div>
    </section>
  );
}
