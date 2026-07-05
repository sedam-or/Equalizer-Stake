"use client";

export default function CtaSection() {
  return (
    <section data-theme="dark" className="snap-section flex min-[100dvh] items-center bg-[#2563EB] px-6 md:px-8">
      <div className="mx-auto max-w-3xl text-center">
        <p className="mb-4 text-sm font-semibold uppercase tracking-[0.3em] text-blue-300">
          Langkah Selanjutnya
        </p>

        <h2 className="text-balance font-heading text-3xl font-bold leading-tight tracking-tight text-white sm:text-4xl lg:text-5xl">
          Jadwalkan Kunjungan ke Studio Kami.
        </h2>

        <p className="mx-auto mt-6 max-w-lg text-lg leading-relaxed text-blue-100">
          Lihat langsung fasilitas kami, bertemu dengan guru, dan rasakan atmosfer belajar yang kami ciptakan untuk anak-anak.
        </p>

        {/* CTA buttons */}
        <div className="mt-10 flex flex-col items-center justify-center gap-4 sm:flex-row">
          <a
            href="https://wa.me/6281234567890?text=Halo,%20saya%20ingin%20jadwalkan%20kunjungan%20ke%20BMD%20Music%20School"
            target="_blank"
            rel="noopener noreferrer"
            className="group inline-flex items-center gap-2 rounded-lg bg-white px-8 py-4 text-sm font-bold text-[#2563EB] shadow-lg transition-all duration-300 hover:shadow-xl hover:shadow-white/20"
          >
            Jadwalkan Kunjungan
            <span className="transition-transform duration-300 group-hover:translate-x-1">→</span>
          </a>
          <a
            href="/register-course"
            className="inline-flex items-center gap-2 rounded-lg border border-white/20 px-8 py-4 text-sm font-semibold text-white transition-all duration-300 hover:border-white/40 hover:bg-white/5"
          >
            Atau Daftar Langsung
          </a>
        </div>

        {/* Trust */}
        <div className="mt-12 flex items-center justify-center gap-6 text-blue-200/50">
          <span className="text-xs">📍 Seturan, Yogyakarta</span>
          <span className="h-3 w-px bg-blue-200/20" />
          <span className="text-xs">📞 0812-3456-7890</span>
          <span className="h-3 w-px bg-blue-200/20" />
          <span className="text-xs">⏰ Sen-Sab, 08:00-20:00</span>
        </div>
      </div>
    </section>
  );
}
