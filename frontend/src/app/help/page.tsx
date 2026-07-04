"use client";
/**
 * Help Page — T-149
 * FAQ accordion + contact info
 */

import { useState } from "react";

const faqs = [
  { q: "Bagaimana cara mendaftar kursus?", a: "Klik 'Daftar via WhatsApp' di halaman utama, atau isi formulir online. Tim kami akan menghubungi Anda dalam 5 menit." },
  { q: "Berapa biaya kursus?", a: "Mulai dari Rp 250.000/bulan untuk 4 pertemuan. Trial tersedia dengan harga Rp 150.000 untuk 1 pertemuan." },
  { q: "Jam berapa kursus tersedia?", a: "Senin-Sabtu, pukul 09:00-20:00. Anda bisa memilih jam yang sesuai jadwal Anda." },
  { q: "Apakah ada ujian?", a: "Ya, ujian berkala diadakan setiap 3 bulan. Sertifikat resmi diberikan setelah lulus ujian." },
  { q: "Bagaimana cara membayar?", a: "Transfer bank dengan kode unik 3 digit. Invoice dikirim otomatis setiap tanggal 5." },
];

export default function HelpPage() {
  const [open, setOpen] = useState<number | null>(null);

  return (
    <div className="min-h-screen bg-[#F8FAFC]">
      <div className="mx-auto max-w-3xl px-6 py-12">
        <h1 className="font-heading text-3xl font-bold text-gray-900">Bantuan</h1>
        <p className="mt-2 text-gray-500">Pertanyaan yang sering ditanyakan</p>

        <div className="mt-8 space-y-3">
          {faqs.map((faq, i) => (
            <div key={i} className="rounded-xl border border-gray-200 bg-white">
              <button onClick={() => setOpen(open === i ? null : i)} className="flex w-full items-center justify-between p-4 text-left">
                <span className="font-medium text-gray-900">{faq.q}</span>
                <span className="text-gray-400">{open === i ? "−" : "+"}</span>
              </button>
              {open === i && <p className="px-4 pb-4 text-sm text-gray-600">{faq.a}</p>}
            </div>
          ))}
        </div>

        <div className="mt-10 rounded-xl border border-gray-200 bg-white p-6">
          <h2 className="font-heading text-lg font-bold text-gray-900">Hubungi Kami</h2>
          <p className="mt-2 text-sm text-gray-500">
            📍 Jl. Seturan No. XX, Yogyakarta
            <br />📱 +62 812-3456-7890 (WhatsApp)
            <br />🕐 Senin-Sabtu 09:00-20:00
          </p>
        </div>
      </div>
    </div>
  );
}
