"use client";

const instruments = [
  { name: "Piano", emoji: "🎹", price: 350000, desc: "Belajar piano klasik & modern" },
  { name: "Gitar", emoji: "🎸", price: 300000, desc: "Akustik, elektrik, klasik" },
  { name: "Biola", emoji: "🎻", price: 400000, desc: "Biola klasik & kontemporer" },
  { name: "Drum", emoji: "🥁", price: 300000, desc: "Drum akustik & elektrik" },
  { name: "Vokal", emoji: "🎤", price: 250000, desc: "Teknik vokal & performance" },
  { name: "Keyboard", emoji: "🎹", price: 300000, desc: "Keyboard & synthesizer" },
  { name: "Saxophone", emoji: "🎷", price: 400000, desc: "Saxophone alto & tenor" },
  { name: "Flute", emoji: "🪈", price: 350000, desc: "Flute klasik & modern" },
  { name: "Bass", emoji: "🎸", price: 300000, desc: "Bass akustik & elektrik" },
];

export default function ProgramKursusSection() {
  return (
    <section id="instrumen" className="bg-white px-6 py-32 md:px-8">
      <div className="mx-auto max-w-7xl">
        {/* Section header — oryzo-inspired: big, bold, uppercase */}
        <div className="text-center">
          <h2 className="font-heading text-4xl font-extrabold uppercase tracking-tighter text-gray-900 sm:text-5xl md:text-6xl">
            Instrumen Kami
          </h2>
          <p className="mt-4 text-lg text-gray-500 md:text-xl">
            Pilih instrumen yang ingin dipelajari
          </p>
        </div>

        {/* Instrument grid */}
        <div className="mt-16 grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {instruments.map((inst) => (
            <div
              key={inst.name}
              className="group rounded-xl border border-gray-200 bg-white p-8 transition-all duration-200 hover:-translate-y-1 hover:shadow-lg"
            >
              <span className="text-5xl">{inst.emoji}</span>
              <h3 className="mt-4 font-heading text-xl font-bold text-gray-900">
                {inst.name}
              </h3>
              <p className="mt-2 text-sm text-gray-500">{inst.desc}</p>
              <div className="mt-4 flex items-center justify-between">
                <p className="text-lg font-bold text-[#2563EB]">
                  Rp {(inst.price / 1000).toLocaleString("id-ID")}rb
                  <span className="text-sm font-normal text-gray-500">/bulan</span>
                </p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
