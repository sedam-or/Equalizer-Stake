"use client";

const testimonials = [
  {
    name: "Ibu Rina",
    child: "Andi, Gitar",
    quote: "Anak saya jadi lebih semangat belajar musik sejak bergabung dengan BMD. Jadwal fleksibel dan gurunya sabar banget.",
    stars: 5,
  },
  {
    name: "Bapak Dimas",
    child: "Sari, Piano",
    quote: "Sistem notifikasi otomatis sangat membantu. Saya tidak pernah lagi lupa jadwal les anak. Tagihan juga jelas.",
    stars: 5,
  },
  {
    name: "Ibu Wati",
    child: "Rizky, Drum",
    quote: "Studio BMD lengkap dan nyaman. Anak saya betah belajar di sini. Rekaman sesi juga jadi kenangan berharga.",
    stars: 5,
  },
];

export default function TestimonialSection() {
  return (
    <section className="bg-[#F8FAFC] px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl text-center">
        <h2 className="font-heading text-3xl font-bold tracking-tight text-gray-900 md:text-4xl">
          Apa Kata Mereka
        </h2>
        <p className="mt-3 text-base text-gray-500 md:text-lg">
          Testimoni dari orang tua siswa BMD Music School
        </p>
        <div className="mt-12 grid grid-cols-1 gap-6 md:grid-cols-3">
          {testimonials.map((t, i) => (
            <div
              key={i}
              className="rounded-xl border border-gray-200 bg-white p-6 text-center transition-all duration-200 hover:translate-y-[-2px] hover:shadow-lg"
            >
              {/* Avatar */}
              <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-[#2563EB]/10 text-lg font-bold text-[#2563EB]">
                {t.name[0]}
              </div>

              {/* Stars */}
              <div className="mt-4 flex items-center justify-center gap-0.5">
                {Array.from({ length: t.stars }).map((_, j) => (
                  <span key={j} className="text-[#F59E0B]">★</span>
                ))}
              </div>

              {/* Quote */}
              <blockquote className="mt-4 text-sm leading-relaxed text-gray-600">
                &ldquo;{t.quote}&rdquo;
              </blockquote>

              {/* Name + Child */}
              <div className="mt-4">
                <p className="text-sm font-semibold text-gray-900">{t.name}</p>
                <p className="text-xs text-gray-500">Orang tua {t.child}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
