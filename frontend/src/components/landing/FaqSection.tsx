"use client";

import { useState, useRef, useCallback } from "react";
import SectionHeader from "@/components/landing/SectionHeader";

const faqs = [
  { q: "Bagaimana cara mendaftar kursus?", a: "Klik 'Daftar via WhatsApp' di halaman ini, atau isi formulir online. Tim kami akan menghubungi Anda dalam 5 menit untuk konfirmasi jadwal." },
  { q: "Berapa biaya kursus?", a: "Mulai dari Rp 150.000 untuk sesi trial. Paket reguler Rp 350.000/bulan (2x pertemuan) dan premium Rp 500.000/bulan (3x pertemuan)." },
  { q: "Jam berapa kursus tersedia?", a: "Senin-Sabtu, pukul 09:00-20:00 WIB. Anda bisa memilih jam yang sesuai jadwal Anda dan anak." },
  { q: "Apakah ada ujian berkala?", a: "Ya, ujian berkala diadakan setiap 3 bulan. Sertifikat resmi diberikan setelah lulus ujian. Hasil ujian bisa dilihat di dashboard." },
  { q: "Bagaimana cara membayar?", a: "Transfer bank dengan kode unik 3 digit. Invoice dikirim otomatis setiap tanggal 5. Denda Rp 100.000 jika terlambat hingga tanggal 11." },
  { q: "Apakah bisa ganti guru?", a: "Bisa! Hubungi admin via WhatsApp. Kami akan menyesuaikan jadwal dengan guru pengganti yang tersedia." },
];

export default function FaqSection() {
  const [open, setOpen] = useState<number | null>(null);
  const contentRefs = useRef<(HTMLDivElement | null)[]>([]);

  const getHeight = useCallback((i: number) => {
    return contentRefs.current[i]?.scrollHeight ?? 0;
  }, []);

  return (
    <section className="bg-[#F8FAFC] px-6 py-32 md:px-8">
      <div className="mx-auto max-w-3xl">
        <SectionHeader title="FAQ" subtitle="Pertanyaan yang Sering Diajukan" />
        <div className="mt-16 space-y-4" role="group" aria-label="Pertanyaan yang sering diajukan">
          {faqs.map((faq, i) => (
            <div key={i} className="rounded-xl border border-gray-200 bg-white">
              <button
                onClick={() => setOpen(open === i ? null : i)}
                aria-expanded={open === i}
                aria-controls={`faq-panel-${i}`}
                className="flex w-full items-center justify-between p-6 text-left focus-visible:outline-2 focus-visible:outline-[#2563EB]"
              >
                <span className="font-heading text-base font-bold text-gray-900">{faq.q}</span>
                <span aria-hidden="true" className="ml-4 flex-shrink-0 text-2xl text-gray-400 transition-transform duration-200" style={{ transform: open === i ? "rotate(45deg)" : "rotate(0deg)" }}>+</span>
              </button>
              <div
                id={`faq-panel-${i}`}
                role="region"
                aria-labelledby={`faq-button-${i}`}
                ref={(el) => { contentRefs.current[i] = el; }}
                className="overflow-hidden transition-all duration-300"
                style={{ maxHeight: open === i ? `${getHeight(i)}px` : "0px" }}
              >
                <div className="border-t border-gray-100 px-6 pb-6 pt-4">
                  <p className="text-sm leading-relaxed text-gray-600">{faq.a}</p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}