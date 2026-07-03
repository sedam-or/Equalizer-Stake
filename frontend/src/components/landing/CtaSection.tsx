"use client";

export default function CtaSection() {
  return (
    <section className="bg-gradient-to-br from-[#2563EB] to-[#7C3AED] px-6 py-24 md:px-8">
      <div className="mx-auto max-w-3xl text-center">
        <h2 className="font-heading text-3xl font-bold text-white md:text-4xl">
          Mulai Gratis — Kuota Trial Terbatas
        </h2>
        <p className="mt-4 text-lg text-white/80">
          Hanya 5 slot trial gratis tersisa bulan ini. Jangan sampai kehabisan.
        </p>
        <div className="mt-8">
          <a
            href="https://wa.me/6281234567890?text=Halo%2C%20saya%20mau%20daftar%20trial%20gratis"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-block rounded-full bg-[#F59E0B] px-10 py-4 text-lg font-bold text-black shadow-lg transition-all hover:bg-[#D97706] hover:shadow-xl"
          >
            Daftar via WhatsApp
          </a>
        </div>
        <p className="mt-6 text-sm text-white/60">
          Atau hubungi langsung: +62 812-3456-7890 (Balasan dalam 5 menit)
        </p>
      </div>
    </section>
  );
}
