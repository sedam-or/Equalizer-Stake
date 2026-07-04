"use client";

export default function HeroSection() {
  return (
    <section className="relative flex min-h-screen items-center justify-center overflow-hidden bg-gradient-to-br from-[#2563EB] to-[#7C3AED] px-6 py-32 md:px-8">
      <div className="relative z-10 mx-auto max-w-5xl text-center">
        {/* Main headline — oryzo-inspired: ultra-bold, tight spacing, ALL CAPS */}
        <h1 className="font-heading text-5xl font-extrabold uppercase tracking-tighter text-white sm:text-6xl md:text-7xl lg:text-8xl">
          Musik Bukan
          <br />
          Sekadar Hobi.
        </h1>

        {/* Subheadline */}
        <p className="mt-6 text-xl font-bold uppercase tracking-wide text-white/90 sm:text-2xl md:text-3xl">
          Musik Membangun Masa Depan.
        </p>

        {/* Description */}
        <p className="mx-auto mt-8 max-w-2xl text-base leading-relaxed text-white/70 sm:text-lg md:text-xl">
          Bersama BMD Music School, anak Anda belajar musik dari guru profesional dengan metode yang menyenangkan.
          Jadikan musik sebagai bekal masa depan.
        </p>

        {/* CTAs */}
        <div className="mt-12 flex flex-col items-center justify-center gap-4 sm:flex-row">
          <a
            href="https://wa.me/6281234567890?text=Halo%2C%20saya%20tertarik%20dengan%20kursus%20musik%20di%20BMD"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 rounded-full bg-[#F59E0B] px-10 py-4 text-base font-bold uppercase tracking-wide text-black shadow-lg transition-all duration-200 hover:bg-[#D97706] hover:shadow-xl hover:scale-105"
          >
            🎵 Daftar via WhatsApp
          </a>
          <a
            href="#instrumen"
            className="inline-flex items-center gap-2 rounded-full border-2 border-white px-10 py-4 text-base font-semibold uppercase tracking-wide text-white transition-all duration-200 hover:bg-white hover:text-[#2563EB]"
          >
            Lihat Instrumen
          </a>
        </div>

        {/* Trust bar — oryzo-inspired: minimal, high-contrast */}
        <div className="mt-20 grid grid-cols-2 gap-8 sm:grid-cols-4">
          {[
            { value: "150+", label: "Siswa Aktif" },
            { value: "9", label: "Instrumen" },
            { value: "10+", label: "Tahun Pengalaman" },
            { value: "4.9 ★", label: "Rating Google" },
          ].map((item) => (
            <div key={item.label} className="text-center">
              <div className="text-3xl font-extrabold text-white sm:text-4xl md:text-5xl">
                {item.value}
              </div>
              <div className="mt-2 text-sm uppercase tracking-wider text-white/60">
                {item.label}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Subtle gradient overlay for depth */}
      <div className="absolute inset-0 bg-gradient-to-t from-black/20 to-transparent" />
    </section>
  );
}
