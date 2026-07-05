"use client";

const testimonials = [
  {
    name: "Ibu Rina",
    role: "Wali Murid Piano",
    text: "Anak saya sekarang lebih percaya diri tampil di depan umum. Metode mengajar di BMD membuat anak saya mencintai piano, bukan sekadar belajar.",
    rating: 5,
  },
  {
    name: "Bapak Ahmad",
    role: "Wali Murid Gitar",
    text: "Sudah 2 tahun anak kami les di BMD. Perkembangannya luar biasa — dari tidak bisa nada sekarang sudah bisa main lagu favoritnya.",
    rating: 5,
  },
  {
    name: "Ibu Sari",
    role: "Wali Murid Vokal",
    text: "Gurunya sabar dan paham kebutuhan anak. Jadwal fleksibel banget, cocok untuk keluarga yang sibuk.",
    rating: 5,
  },
];

export default function TestimonialSection() {
  return (
    <section className="bg-gray-50 px-6 py-20 md:px-8 md:py-28">
      <div className="mx-auto max-w-7xl">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-balance font-heading text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
            Apa Kata Keluarga Kami?
          </h2>
          <p className="mt-4 text-lg text-gray-600">
            150+ keluarga di Yogyakarta sudah mempercayakan pendidikan musik anak mereka kepada kami.
          </p>
        </div>

        <div className="mt-16 grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {testimonials.map((t, i) => (
            <div key={i} className="rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
              {/* Stars */}
              <div className="flex gap-1">
                {Array.from({ length: t.rating }).map((_, j) => (
                  <span key={j} className="text-amber-400">★</span>
                ))}
              </div>
              {/* Quote */}
              <p className="mt-4 text-sm leading-relaxed text-gray-700">&ldquo;{t.text}&rdquo;</p>
              {/* Author */}
              <div className="mt-4 border-t border-gray-100 pt-4">
                <div className="font-heading text-sm font-bold text-gray-900">{t.name}</div>
                <div className="text-xs text-gray-500">{t.role}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
