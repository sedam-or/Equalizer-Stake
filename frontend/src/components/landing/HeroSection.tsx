"use client";

export default function HeroSection() {
  return (
    <section className="relative overflow-hidden bg-gradient-to-br from-[#2563EB] to-[#1E40AF] px-6 pt-28 pb-20 md:px-8 md:pt-36 md:pb-28">
      <div className="mx-auto grid max-w-7xl items-center gap-12 lg:grid-cols-2">
        {/* Left: Text */}
        <div className="text-center lg:text-left">
          <h1 className="text-balance font-heading text-4xl font-bold tracking-tight text-white sm:text-5xl lg:text-6xl">
            Bakat Musik Anak Anda, Layak Dikembangkan dengan Benar.
          </h1>
          <p className="mt-6 max-w-lg text-lg leading-relaxed text-blue-100 sm:text-xl lg:mx-0">
            BMD Music School, Yogyakarta. Guru berpengalaman, metode terstruktur, dan lingkungan yang menginspirasi anak untuk mencintai musik seumur hidup.
          </p>

          <div className="mt-10 flex flex-col items-center gap-4 sm:flex-row lg:justify-start">
            <a
              href="/register-course"
              className="inline-flex items-center gap-2 rounded-lg bg-white px-8 py-4 text-base font-bold text-[#2563EB] shadow-lg transition-all duration-200 hover:shadow-xl hover:scale-105"
            >
              Mulai Perjalanan Musik Anak
            </a>
            <a
              href="#instrumen"
              className="inline-flex items-center gap-2 rounded-lg border-2 border-white/30 px-8 py-4 text-base font-semibold text-white transition-all duration-200 hover:bg-white/10"
            >
              Lihat Program
            </a>
          </div>

          {/* Trust signals */}
          <div className="mt-12 flex flex-wrap items-center justify-center gap-6 lg:justify-start">
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold text-white">150+</span>
              <span className="text-sm text-blue-200">Keluarga</span>
            </div>
            <div className="h-4 w-px bg-white/20" />
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold text-white">10+</span>
              <span className="text-sm text-blue-200">Tahun</span>
            </div>
            <div className="h-4 w-px bg-white/20" />
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold text-white">4.9 ★</span>
              <span className="text-sm text-blue-200">Google</span>
            </div>
          </div>
        </div>

        {/* Right: Visual */}
        <div className="relative hidden lg:block">
          <div className="relative overflow-hidden rounded-2xl shadow-2xl">
            <img
              src="/images/studio-1.jpg"
              alt="Studio BMD Music School"
              className="h-[500px] w-full object-cover"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-[#2563EB]/30 to-transparent" />
          </div>
        </div>
      </div>
    </section>
  );
}
