import SectionHeader from "@/components/landing/SectionHeader";

export default function CtaSection() {
  return (
    <section className="bg-gradient-to-br from-[#2563EB] to-[#7C3AED] px-6 py-32 md:px-8">
      <div className="mx-auto max-w-4xl text-center">
        <h2 className="font-heading text-4xl font-extrabold uppercase tracking-tighter text-white sm:text-5xl md:text-6xl">
          Siap Memulai?
        </h2>
        <p className="mt-6 text-lg text-white/70 md:text-xl">
          Daftar sekarang dan dapatkan sesi trial gratis!
        </p>
        <div className="mt-10">
          <a href="https://wa.me/6281234567890?text=Halo%2C%20saya%20ingin%20daftar%20kursus%20musik%20di%20BMD" target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-2 rounded-full bg-[#F59E0B] px-12 py-4 text-base font-bold uppercase tracking-wide text-black shadow-lg transition-all duration-200 hover:bg-[#D97706] hover:shadow-xl hover:scale-105 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#F59E0B]">
            🎵 Daftar via WhatsApp
          </a>
        </div>
      </div>
    </section>
  );
}