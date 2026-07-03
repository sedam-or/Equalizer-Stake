"use client";

import { useEffect, useState } from "react";

export default function HeroSection() {
  return (
    <section className="relative flex min-h-[80vh] items-center justify-center overflow-hidden bg-gradient-to-br from-[#2563EB]/5 via-[#F8FAFC] to-[#7C3AED]/5 px-6">
      {/* Content */}
      <div className="mx-auto w-full max-w-4xl text-center">
        <div className="mb-4 inline-block rounded-full bg-[#2563EB]/10 px-4 py-1.5 text-sm font-medium text-[#2563EB]">
          Kursus Musik Premium di Yogyakarta
        </div>

        <h1 className="mb-6 font-heading text-4xl font-extrabold leading-tight text-gray-900 md:text-5xl lg:text-6xl">
          Musik Bukan Sekadar Hobi.
          <br />
          <span className="text-[#2563EB]">Musik Membangun Masa Depan.</span>
        </h1>

        <p className="mx-auto mb-8 max-w-2xl text-lg leading-relaxed text-gray-600 md:text-xl">
          Kursus musik premium untuk anak usia 5-18 tahun di Yogyakarta.
          Guru berpengalaman, sistem digital, hasil nyata.
        </p>

        <p className="mb-10 text-base text-gray-500">
          Sudah dipercaya <span className="font-semibold text-[#2563EB]">150+ keluarga</span> di Yogyakarta sejak 2014.
        </p>

        <div className="flex flex-wrap items-center justify-center gap-4">
          <a
            href="https://wa.me/6281234567890?text=Halo%2C%20saya%20tertarik%20dengan%20kursus%20musik%20di%20BMD"
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-full bg-[#2563EB] px-8 py-3.5 text-base font-semibold text-white shadow-lg shadow-[#2563EB]/25 transition-all hover:bg-[#1d4ed8] hover:shadow-xl"
          >
            Mulai Gratis Trial
          </a>
          <a
            href="#program"
            className="rounded-full border-2 border-gray-300 px-8 py-3.5 text-base font-semibold text-gray-700 transition-all hover:border-[#2563EB] hover:text-[#2563EB]"
          >
            Lihat Program
          </a>
        </div>
      </div>
    </section>
  );
}
