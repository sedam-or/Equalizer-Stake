"use client";

import BlurText from "@/components/BlurText";

const teachers = [
  {
    name: "Dimas Prasetyo",
    instrument: "Gitar & Piano",
    experience: "12 tahun",
    photo: "/images/teachers/dimas.jpg",
    quote: "Musik adalah bahasa yang paling jujur. Saya mengajarkan anak untuk mendengarkan dan merasakan, bukan hanya menekan tombol.",
  },
  {
    name: "Sari Dewi",
    instrument: "Vokal & Biola",
    experience: "10 tahun",
    photo: "/images/teachers/sari.jpg",
    quote: "Setiap anak punya suara unik. Tugas saya adalah menemukan dan mengembangkannya, bukan membentuk mereka menjadi orang lain.",
  },
  {
    name: "Rizky Firmansyah",
    instrument: "Drum & Bass",
    experience: "8 tahun",
    photo: "/images/teachers/rizky.jpg",
    quote: "Drum mengajarkan disiplin dan ritme hidup. Saya melihat anak-anak tumbuh lebih fokus dan percaya diri melalui musik.",
  },
];

const badges = [
  { icon: "🎓", text: "Pendidikan Musik Formal" },
  { icon: "🏆", text: "Sertifikat Internasional" },
  { icon: "🎵", text: "Pernah Tampil di Konser Nasional" },
  { icon: "👶", text: "Berpengalaman Mengajar Anak" },
];

export default function TeachersSection() {
  return (
    <section className="snap-section flex min-h-[100dvh] items-center bg-[#0F172A] px-6 md:px-8">
      <div className="mx-auto w-full max-w-7xl">
        {/* Header */}
        <div className="mb-12 max-w-2xl">
          <BlurText
            text="Guru yang Menginspirasi."
            animateBy="words"
            direction="top"
            delay={100}
            className="text-balance font-heading text-3xl font-bold leading-tight tracking-tight text-white sm:text-4xl"
          />
          <BlurText
            text="Yang Mengajarkan Lebih dari Sekadar Notasi."
            animateBy="words"
            direction="top"
            delay={150}
            className="mt-2 text-balance font-heading text-3xl font-bold leading-tight tracking-tight text-[#D4AF37] sm:text-4xl"
          />
          <p className="mt-6 max-w-lg text-gray-400">
            Semua instructor kami memiliki latar belakang pendidikan musik formal dan pengalaman mengajar anak. Mereka tidak hanya mengajarkan teknik, tapi juga membangun kecintaan pada musik.
          </p>
        </div>

        {/* Trust Badges */}
        <div className="mb-10 flex flex-wrap gap-4">
          {badges.map((badge, i) => (
            <div key={i} className="flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-4 py-2">
              <span className="text-lg">{badge.icon}</span>
              <span className="text-xs font-medium text-gray-300">{badge.text}</span>
            </div>
          ))}
        </div>

        {/* Teacher Cards */}
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {teachers.map((teacher, i) => (
            <div
              key={i}
              className="group rounded-xl border border-white/10 bg-white/5 p-6 transition-all duration-300 hover:border-[#D4AF37]/30 hover:bg-white/[0.07]"
            >
              {/* Photo */}
              <div className="relative mb-4">
                <div className="h-32 w-full overflow-hidden rounded-lg bg-gray-800">
                  <img
                    src={teacher.photo}
                    alt={teacher.name}
                    className="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
                    onError={(e) => {
                      (e.target as HTMLImageElement).src = `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><rect fill="%23334155" width="100" height="100"/><text x="50" y="55" text-anchor="middle" fill="%2394A3B8" font-size="32">🎵</text></svg>')}`;
                    }}
                  />
                </div>
                {/* Experience badge */}
                <div className="absolute -bottom-3 right-3 rounded-full bg-[#D4AF37] px-3 py-1 text-xs font-bold text-[#0F172A]">
                  {teacher.experience}
                </div>
              </div>

              {/* Info */}
              <div className="mt-4">
                <h3 className="font-heading text-lg font-bold text-white">{teacher.name}</h3>
                <p className="text-sm font-medium text-[#D4AF37]">{teacher.instrument}</p>
              </div>

              {/* Quote */}
              <p className="mt-4 text-sm leading-relaxed text-gray-400 italic">
                &ldquo;{teacher.quote}&rdquo;
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
