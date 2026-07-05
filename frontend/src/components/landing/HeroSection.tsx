"use client";

export default function HeroSection() {
  return (
    <section className="relative overflow-hidden bg-gradient-to-br from-[#F59E0B] to-[#EA580C] px-6 pt-24 pb-16 md:px-8 md:pt-32 md:pb-24">
      <div className="mx-auto grid max-w-7xl items-center gap-12 lg:grid-cols-2">
        {/* Left: Text */}
        <div className="text-center lg:text-left">
          <h1 className="text-balance font-heading text-4xl font-bold tracking-tight text-white sm:text-5xl lg:text-6xl">
            Anak Anda Punya Bakat Musik?
          </h1>
          <p className="mt-4 text-xl font-semibold text-white/90 sm:text-2xl">
            Kami Bantu Mengembangkannya.
          </p>
          <p className="mx-auto mt-6 max-w-lg text-base leading-relaxed text-white/80 sm:text-lg lg:mx-0">
            Bergabung dengan 150+ keluarga di Yogyakarta yang sudah mempercayakan pendidikan musik anak mereka kepada BMD Music School.
          </p>

          <div className="mt-10 flex flex-col items-center gap-4 sm:flex-row lg:justify-start">
            <a
              href="/register-course"
              className="inline-flex items-center gap-2 rounded-full bg-white px-8 py-4 text-base font-bold text-[#EA580C] shadow-lg transition-all duration-200 hover:shadow-xl hover:scale-105"
            >
              🎵 Daftar Kursus
            </a>
            <a
              href="#instrumen"
              className="inline-flex items-center gap-2 rounded-full border-2 border-white px-8 py-4 text-base font-semibold text-white transition-all duration-200 hover:bg-white/10"
            >
              Lihat Instrumen
            </a>
          </div>

          {/* Trust bar */}
          <div className="mt-12 flex flex-wrap items-center justify-center gap-6 lg:justify-start">
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold text-white">150+</span>
              <span className="text-sm text-white/70">Keluarga</span>
            </div>
            <div className="h-4 w-px bg-white/30" />
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold text-white">9</span>
              <span className="text-sm text-white/70">Instrumen</span>
            </div>
            <div className="h-4 w-px bg-white/30" />
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold text-white">10+</span>
              <span className="text-sm text-white/70">Tahun</span>
            </div>
            <div className="h-4 w-px bg-white/30" />
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold text-white">4.9 ★</span>
              <span className="text-sm text-white/70">Google</span>
            </div>
          </div>
        </div>

        {/* Right: Studio photo */}
        <div className="relative hidden lg:block">
          <div className="relative overflow-hidden rounded-2xl shadow-2xl">
            <img
              src="/images/studio-1.jpg"
              alt="Studio BMD Music School"
              className="h-[500px] w-full object-cover"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-black/20 to-transparent" />
          </div>
          {/* Floating badge */}
          <div className="absolute -bottom-4 -left-4 rounded-xl bg-white p-4 shadow-lg">
            <div className="text-sm font-bold text-[#2563EB]">🎵 Kursus Musik</div>
            <div className="text-xs text-gray-500">Kursus musik</div>
          </div>
        </div>
      </div>
    </section>
  );
}
