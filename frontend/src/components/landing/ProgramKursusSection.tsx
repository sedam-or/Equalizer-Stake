"use client";

const instruments = [
  {
    name: "Piano",
    desc: "Klasik & Pop — dari pemula hingga performa",
    image: "/images/instruments/piano.jpg",
  },
  {
    name: "Gitar",
    desc: "Akustik & Elektrik — fondasi musik modern",
    image: "/images/instruments/guitar.jpg",
  },
  {
    name: "Vokal",
    desc: "Pop & Klasik — temukan suara unik anak",
    image: "/images/instruments/vocal.jpg",
  },
  {
    name: "Biola",
    desc: "Klasik — disiplin dan keindahan nada",
    image: "/images/instruments/violin.jpg",
  },
];

export default function ProgramKursusSection() {
  return (
    <section id="instrumen" className="snap-section flex min-h-[100dvh] items-center bg-[#F8FAFC] px-6 md:px-8">
      <div className="mx-auto w-full max-w-7xl">
        {/* Header */}
        <div className="mb-12 max-w-2xl">
          <p className="mb-4 text-sm font-semibold uppercase tracking-[0.3em] text-[#D4AF37]">
            Program
          </p>
          <h2 className="text-balance font-heading text-3xl font-bold leading-tight tracking-tight text-[#0F172A] sm:text-4xl">
            Pilih Instrumen.
            <br />
            <span className="text-gray-400">Mulai Perjalanan Musik Anak.</span>
          </h2>
        </div>

        {/* Instruments — horizontal scroll on mobile */}
        <div className="flex gap-4 overflow-x-auto pb-4 sm:grid sm:grid-cols-2 sm:overflow-visible lg:grid-cols-4">
          {instruments.map((inst) => (
            <a
              key={inst.name}
              href="/register-course"
              className="group relative flex w-[280px] shrink-0 flex-col overflow-hidden rounded-sm border border-gray-200 bg-white transition-all duration-300 hover:border-[#D4AF37]/30 hover:shadow-lg sm:w-auto"
            >
              {/* Image */}
              <div className="relative h-48 overflow-hidden">
                <img
                  src={inst.image}
                  alt={inst.name}
                  className="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
                />
                <div className="absolute inset-0 bg-gradient-to-t from-[#0F172A]/60 to-transparent" />
                <span className="absolute bottom-3 left-4 font-heading text-lg font-bold text-white">
                  {inst.name}
                </span>
              </div>
              {/* Text */}
              <div className="p-4">
                <p className="text-sm text-gray-500">{inst.desc}</p>
                <div className="mt-3 flex items-center gap-1 text-xs font-semibold text-[#2563EB] opacity-0 transition-all duration-300 group-hover:opacity-100">
                  Lihat Detail <span className="transition-transform duration-300 group-hover:translate-x-1">→</span>
                </div>
              </div>
            </a>
          ))}
        </div>

        {/* Link to all instruments */}
        <div className="mt-8">
          <a
            href="/instruments"
            className="inline-flex items-center gap-2 text-sm font-semibold text-gray-400 transition-colors duration-300 hover:text-[#2563EB]"
          >
            Dan 5 instrumen lainnya
            <span className="transition-transform duration-300 group-hover:translate-x-1">→</span>
          </a>
        </div>
      </div>
    </section>
  );
}
