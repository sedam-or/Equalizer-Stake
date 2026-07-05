"use client";

export default function TentangSection() {
  return (
    <section className="bg-white px-6 py-20 md:px-8 md:py-28">
      <div className="mx-auto max-w-7xl">
        <div className="grid items-center gap-12 lg:grid-cols-2">
          {/* Left: Text */}
          <div>
            <h2 className="text-balance font-heading text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
              Bukan Sekadar Kursus Musik
            </h2>
            <p className="mt-6 text-lg leading-relaxed text-gray-600">
              BMD Music School hadir untuk membantu anak-anak di Yogyakarta mengembangkan bakat musik mereka dengan cara yang benar — terstruktur, menyenangkan, dan berkelanjutan.
            </p>
            <div className="mt-8 space-y-4">
              <div className="flex items-start gap-3">
                <div className="mt-1 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-blue-100 text-sm text-blue-600">✓</div>
                <p className="text-gray-700">Kurikulum berstandar internasional dengan metode yang disesuaikan untuk anak-anak</p>
              </div>
              <div className="flex items-start gap-3">
                <div className="mt-1 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-blue-100 text-sm text-blue-600">✓</div>
                <p className="text-gray-700">Jadwal fleksibel — pagi, siang, atau sore sesuai ketersediaan keluarga</p>
              </div>
              <div className="flex items-start gap-3">
                <div className="mt-1 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-blue-100 text-sm text-blue-600">✓</div>
                <p className="text-gray-700">Laporan perkembangan berkala agar Anda tahu persis kemajuan anak</p>
              </div>
            </div>
          </div>

          {/* Right: Visual */}
          <div className="relative">
            <div className="overflow-hidden rounded-2xl bg-gray-100 shadow-lg">
              <img
                src="/images/studio-2.jpg"
                alt="Kelas musik di BMD Music School"
                className="h-[400px] w-full object-cover"
              />
            </div>
            {/* Floating stat */}
            <div className="absolute -bottom-6 -left-6 rounded-xl border border-gray-200 bg-white p-4 shadow-lg">
              <div className="font-heading text-2xl font-bold text-[#2563EB]">10+</div>
              <div className="text-xs text-gray-500">Tahun Pengalaman</div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
