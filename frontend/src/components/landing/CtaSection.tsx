"use client";

export default function CtaSection() {
  return (
    <section className="bg-[#2563EB] px-6 py-20 md:px-8 md:py-28">
      <div className="mx-auto max-w-3xl text-center">
        <h2 className="text-balance font-heading text-3xl font-bold tracking-tight text-white sm:text-4xl">
          Siap Memberikan yang Terbaik untuk Anak?
        </h2>
        <p className="mt-4 text-lg text-blue-100">
          Pendaftaran dibuka sepanjang tahun. Mulai perjalanan musik anak Anda hari ini.
        </p>
        <div className="mt-8 flex flex-col items-center justify-center gap-4 sm:flex-row">
          <a
            href="/register-course"
            className="inline-flex items-center gap-2 rounded-lg bg-white px-8 py-4 text-base font-bold text-[#2563EB] shadow-lg transition-all duration-200 hover:shadow-xl hover:scale-105"
          >
            Daftar Sekarang
          </a>
          <a
            href="https://wa.me/6281234567890"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 rounded-lg border-2 border-white/30 px-8 py-4 text-base font-semibold text-white transition-all duration-200 hover:bg-white/10"
          >
            Chat via WhatsApp
          </a>
        </div>
      </div>
    </section>
  );
}
