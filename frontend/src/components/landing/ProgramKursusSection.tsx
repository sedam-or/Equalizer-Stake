"use client";

const instruments = [
  { name: "Piano", emoji: "🎹", desc: "Klasik & Pop", color: "bg-blue-50 text-blue-700 border-blue-100" },
  { name: "Gitar", emoji: "🎸", desc: "Akustik & Elektrik", color: "bg-amber-50 text-amber-700 border-amber-100" },
  { name: "Vokal", emoji: "🎤", desc: "Pop & Klasik", color: "bg-rose-50 text-rose-700 border-rose-100" },
  { name: "Biola", emoji: "🎻", desc: "Klasik", color: "bg-purple-50 text-purple-700 border-purple-100" },
  { name: "Drum", emoji: "🥁", desc: "Modern & Klasik", color: "bg-emerald-50 text-emerald-700 border-emerald-100" },
  { name: "Keyboard", emoji: "🎹", desc: "Pop & Jazz", color: "bg-cyan-50 text-cyan-700 border-cyan-100" },
  { name: "Flute", emoji: "🪈", desc: "Klasik", color: "bg-teal-50 text-teal-700 border-teal-100" },
  { name: "Saxophone", emoji: "🎷", desc: "Jazz & Pop", color: "bg-orange-50 text-orange-700 border-orange-100" },
  { name: "Bass", emoji: "🎸", desc: "Gitar Bass", color: "bg-slate-50 text-slate-700 border-slate-100" },
];

export default function ProgramKursusSection() {
  return (
    <section id="instrumen" className="bg-gray-50 px-6 py-20 md:px-8 md:py-28">
      <div className="mx-auto max-w-7xl">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-balance font-heading text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
            Pilih Instrumen Favorit Anak
          </h2>
          <p className="mt-4 text-lg text-gray-600">
            9 instrumen tersedia dengan guru spesialis di bidangnya masing-masing.
          </p>
        </div>

        <div className="mt-16 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {instruments.map((inst) => (
            <a
              key={inst.name}
              href="/register-course"
              className={`group flex items-center gap-4 rounded-xl border p-5 transition-all duration-200 hover:shadow-md ${inst.color}`}
            >
              <span className="text-3xl">{inst.emoji}</span>
              <div>
                <div className="font-heading text-base font-bold">{inst.name}</div>
                <div className="text-sm opacity-70">{inst.desc}</div>
              </div>
            </a>
          ))}
        </div>
      </div>
    </section>
  );
}
