"use client";

export default function HeroSection() {
  return (
    <section className="relative flex min-h-[80vh] items-center justify-center overflow-hidden bg-gradient-to-br from-[#2563EB] to-[#7C3AED] px-6 py-24 md:px-8">
      <div className="relative z-10 mx-auto max-w-3xl text-center">
        <h1 className="font-heading text-4xl font-extrabold tracking-tight text-white sm:text-5xl md:text-6xl">
          Musik Bukan Sekadar Hobi.
        </h1>
        <p className="mt-4 text-lg font-bold text-white sm:text-xl">
          Musik Membangun Masa Depan.
        </p>
        <p className="mx-auto mt-6 max-w-xl text-base text-white/70 sm:text-lg">
          Bersama BMD Music School, anak Anda belajar musik dari guru profesional dengan metode yang menyenangkan. Jadikan musik sebagai bekal masa depan.
        </p>

        <div className="mt-10 flex flex-col items-center justify-center gap-4 sm:flex-row">
          <a
            href="https://wa.me/6281234567890?text=Halo%2C%20saya%20tertarik%20dengan%20kursus%20musik%20di%20BMD"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 rounded-full bg-[#F59E0B] px-8 py-3 text-sm font-bold text-black shadow-lg transition-all hover:bg-[#D97706] hover:shadow-xl"
          >
            🎵 Daftar via WhatsApp
          </a>
          <a
            href="#instrumen"
            className="inline-flex items-center gap-2 rounded-full border-2 border-white px-8 py-3 text-sm font-semibold text-white transition-all hover:bg-white hover:text-[#2563EB]"
          >
            Lihat Instrumen
          </a>
        </div>

        {/* Trust bar */}
        <div className="mt-16 grid grid-cols-2 gap-6 sm:grid-cols-4">
          <div className="text-center">
            <div className="text-2xl font-bold text-white sm:text-3xl">150+</div>
            <div className="mt-1 text-sm text-white/70">Siswa Aktif</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-white sm:text-3xl">9</div>
            <div className="mt-1 text-sm text-white/70">Instrumen</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-white sm:text-3xl">10+</div>
            <div className="mt-1 text-sm text-white/70">Tahun Pengalaman</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-white sm:text-3xl">4.9 ★</div>
            <div className="mt-1 text-sm text-white/70">Rating Google</div>
          </div>
        </div>
      </div>
    </section>
  );
}