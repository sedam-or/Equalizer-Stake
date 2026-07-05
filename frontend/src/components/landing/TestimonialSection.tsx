"use client";
import BlurText from "@/components/BlurText";

export default function TestimonialSection() {
  return (
    <section data-theme="dark" className="snap-section flex min-[100dvh] items-center bg-[#0F172A] px-6 pt-20 md:px-8">
      <div className="mx-auto max-w-4xl text-center">
        {/* Quote mark */}
        <div className="mb-8 font-heading text-6xl text-[#D4AF37]/30">&ldquo;</div>

        {/* Quote */}
        <BlurText
          text="Anak saya sekarang lebih percaya diri tampil di depan umum. Metode mengajar di BMD membuat anak saya mencintai piano — bukan sekadar belajar."
          animateBy="words"
          direction="top"
          delay={80}
          className="text-balance font-heading text-2xl font-light leading-relaxed text-white/90 sm:text-3xl lg:text-4xl"
        />

        {/* Author */}
        <div className="mt-10 flex flex-col items-center gap-2">
          <div className="flex gap-1">
            {Array.from({ length: 5 }).map((_, i) => (
              <span key={i} className="text-[#D4AF37]">★</span>
            ))}
          </div>
          <p className="mt-2 font-heading text-sm font-semibold text-[#D4AF37]">
            Ibu Rina
          </p>
          <p className="text-xs text-gray-500">
            Wali Murid — Piano, 2 tahun bersama BMD
          </p>
        </div>

        {/* Subtle divider */}
        <div className="mx-auto mt-16 h-px w-16 bg-gradient-to-r from-transparent via-[#D4AF37]/30 to-transparent" />

        <p className="mt-6 text-sm text-gray-600">
          150+ keluarga di Yogyakarta mempercayakan pendidikan musik anak kepada BMD
        </p>
      </div>
    </section>
  );
}
