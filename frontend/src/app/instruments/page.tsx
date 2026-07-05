"use client";

import Link from "next/link";

const instruments = [
  { name: "Piano Klasik", desc: "Dari pra-elementary hingga grade 6", emoji: "🎹", levels: ["Pra-Elementary", "Elementary", "Grade 1-6"] },
  { name: "Piano Pop", desc: "Pop, jazz, dan modern", emoji: "🎹", levels: ["Beginner", "Intermediate", "Intermediate 2"] },
  { name: "Gitar", desc: "Akustik dan elektro", emoji: "🎸", levels: ["Basic", "Intermediate", "Advance"] },
  { name: "Vokal", desc: "Pop dan klasik", emoji: "🎤", levels: ["Basic", "Intermediate", "Advance"] },
  { name: "Biola", desc: "Klasik dan orkestra", emoji: "🎻", levels: ["Basic", "Intermediate", "Advance"] },
  { name: "Drum", desc: "Modern dan klasik", emoji: "🥁", levels: ["Basic", "Intermediate", "Advance"] },
  { name: "Keyboard", desc: "Pop dan jazz", emoji: "🎹", levels: ["Basic", "Intermediate", "Advance"] },
  { name: "Flute", desc: "Klasik dan modern", emoji: "🪈", levels: ["Basic", "Intermediate", "Advance"] },
  { name: "Saxophone", desc: "Jazz dan pop", emoji: "🎷", levels: ["Basic", "Intermediate", "Advance"] },
];

export default function InstrumentsPage() {
  return (
    <div className="min-h-screen bg-[#F8FAFC]">
      {/* Header */}
      <div className="bg-[#0F172A] px-6 py-20 md:px-8">
        <div className="mx-auto max-w-3xl text-center">
          <Link href="/" className="mb-6 inline-block font-heading text-xl font-bold tracking-tight text-white">
            BMD <span className="text-[#D4AF37]">Music</span>
          </Link>
          <h1 className="text-balance font-heading text-3xl font-bold tracking-tight text-white sm:text-4xl">
            Instrumen yang Kami Ajarkan
          </h1>
          <p className="mt-4 text-gray-400">
            9 instrumen dengan guru spesialis di bidangnya masing-masing
          </p>
        </div>
      </div>

      {/* Instruments grid */}
      <div className="mx-auto max-w-7xl px-6 py-16 md:px-8">
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {instruments.map((inst) => (
            <div
              key={inst.name}
              className="group rounded-xl border border-gray-200 bg-white p-6 transition-all duration-300 hover:border-[#D4AF37]/30 hover:shadow-md"
            >
              <div className="text-3xl">{inst.emoji}</div>
              <h2 className="mt-4 font-heading text-lg font-bold text-[#0F172A]">{inst.name}</h2>
              <p className="mt-1 text-sm text-gray-500">{inst.desc}</p>
              <div className="mt-4 flex flex-wrap gap-2">
                {inst.levels.map((level) => (
                  <span
                    key={level}
                    className="rounded-full bg-gray-100 px-3 py-1 text-xs text-gray-600"
                  >
                    {level}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* CTA */}
        <div className="mt-16 text-center">
          <h2 className="font-heading text-2xl font-bold text-[#0F172A]">
            Tertarik dengan salah satu instrumen?
          </h2>
          <p className="mt-2 text-gray-500">
            Hubungi kami untuk informasi lebih lanjut atau langsung daftar kursus.
          </p>
          <div className="mt-6 flex flex-col items-center justify-center gap-4 sm:flex-row">
            <a
              href="/register-course"
              className="inline-flex items-center gap-2 rounded-lg bg-[#2563EB] px-8 py-3 text-sm font-semibold text-white hover:bg-[#1D4ED8]"
            >
              Daftar Kursus
            </a>
            <a
              href="https://wa.me/6281234567890"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 rounded-lg border border-gray-300 px-8 py-3 text-sm font-semibold text-[#0F172A] hover:bg-gray-50"
            >
              Chat via WhatsApp
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
