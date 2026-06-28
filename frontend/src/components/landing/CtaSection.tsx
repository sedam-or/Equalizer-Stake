"use client";

import BlurText from "@/components/ui/BlurText";

export default function CtaSection() {
  return (
    <section className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-3xl text-center">
        <BlurText text="Siap Mulai?" delay={120} className="text-3xl font-bold tracking-tight text-[#08CB00] md:text-5xl" />
        <BlurText text="Daftar sekarang dan nikmati kemudahan penjadwalan kursus musik" delay={100} className="mt-4 text-base text-gray-400 md:text-lg" />
        <div className="mt-8"><a href="#hubungi" className="inline-block rounded-full border-2 border-[#08CB00] px-8 py-3.5 text-base font-semibold text-[#08CB00] transition-all hover:bg-[#08CB00] hover:text-black">Hubungi Kami</a></div>
      </div>
    </section>
  );
}
