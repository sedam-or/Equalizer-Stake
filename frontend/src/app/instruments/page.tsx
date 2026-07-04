"use client";
/**
 * Instruments Public Page — T-148
 * List semua instrumen + harga
 */

import { Breadcrumbs } from "@/components/Breadcrumbs";

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

export default function InstrumentsPage() {
  return (
    <div className="min-h-screen bg-[#F8FAFC]">
      <div className="mx-auto max-w-5xl px-6 py-12">
        <Breadcrumbs items={[{ label: "Instrumen" }]} />
        <h1 className="font-heading text-3xl font-bold text-gray-900">Instrumen Kami</h1>
        <p className="mt-2 text-gray-500">Pilih instrumen yang ingin dipelajari</p>

        <div className="mt-8 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {instruments.map((inst) => (
            <div key={inst.name} className="rounded-xl border border-gray-200 bg-white p-5 transition-shadow hover:shadow-md">
              <span className="text-4xl">{inst.emoji}</span>
              <h3 className="mt-3 font-heading text-lg font-bold text-gray-900">{inst.name}</h3>
              <p className="mt-1 text-sm text-gray-500">{inst.desc}</p>
              <p className="mt-3 text-sm font-bold text-[#2563EB]">Mulai Rp {(inst.price / 1000).toLocaleString("id-ID")}rb/bulan</p>
            </div>
          ))}
        </div>

        <div className="mt-10 text-center">
          <a href="https://wa.me/6281234567890?text=Halo%2C%20saya%20tertarik%20dengan%20kursus%20musik%20di%20BMD" target="_blank" rel="noopener noreferrer" className="inline-block rounded-full bg-[#F59E0B] px-8 py-3 text-sm font-bold text-black hover:bg-[#D97706]">Daftar via WhatsApp</a>
        </div>
      </div>
    </div>
  );
}
