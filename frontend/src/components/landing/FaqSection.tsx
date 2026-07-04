"use client";

import { useState } from "react";

const faqs = [
  {
    q: "Bagaimana cara mendaftar kursus?",
    a: "Klik 'Daftar via WhatsApp' di halaman ini, atau isi formulir online. Tim kami akan menghubungi Anda dalam 5 menit untuk konfirmasi jadwal.",
  },
  {
    q: "Berapa biaya kursus?",
    a: "Mulai dari Rp 150.000 untuk sesi trial. Paket reguler Rp 350.000/bulan (2x pertemuan) dan premium Rp 500.000/bulan (3x pertemuan).",
  },
  {
    q: "Jam berapa kursus tersedia?",
    a: "Senin-Sabtu, pukul 09:00-20:00 WIB. Anda bisa memilih jam yang sesuai jadwal Anda dan anak.",
  },
  {
    q: "Apakah ada ujian berkala?",
    a: "Ya, ujian berkala diadakan setiap 3 bulan. Sertifikat resmi diberikan setelah lulus ujian. Hasil ujian bisa dilihat di dashboard.",
  },
  {
    q: "Bagaimana cara membayar?",
    a: "Transfer bank dengan kode unik 3 digit. Invoice dikirim otomatis setiap tanggal 5. Denda Rp 100.000 jika terlambat hingga tanggal 11.",
  },
  {
    q: "Apakah bisa ganti guru?",
    a: "Bisa! Hubungi admin via WhatsApp. Kami akan menyesuaikan jadwal dengan guru pengganti yang tersedia.",
  },
];

export default function FaqSection() {
  const [open, setOpen] = useState<number | null>(null);

  return (
    <section className="bg-[#F8FAFC] px-6 py-24 md:px-8">
      <div className="mx-auto max-w-3xl text-center">
        <h2 className="font-heading text-3xl font-bold tracking-tight text-gray-900 md:text-4xl">
          Pertanyaan yang Sering Diajukan
        </h2>
        <p className="mt-3 text-base text-gray-500 md:text-lg">
          Belum menemukan jawaban? Hubungi kami via WhatsApp
        </p>

        <div className="mt-12 space-y-3 text-left">
          {faqs.map((faq, i) => (
            <div key={i} className="rounded-xl border border-gray-200 bg-white">
              <button
                onClick={() => setOpen(open === i ? null : i)}
                className="flex w-full items-center justify-between p-4 text-left"
              >
                <span className="font-medium text-gray-900">{faq.q}</span>
                <span className="ml-4 flex-shrink-0 text-gray-400">{open === i ? "−" : "+"}</span>
              </button>
              {open === i && (
                <div className="border-t border-gray-100 px-4 pb-4 pt-3">
                  <p className="text-sm text-gray-600">{faq.a}</p>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
