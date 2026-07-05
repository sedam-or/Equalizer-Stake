"use client";

const reasons = [
  {
    icon: "🎯",
    title: "Metode Terstruktur",
    desc: "Kurikulum yang dirancang khusus untuk perkembangan musik anak, bukan sekadar mengajarkan notasi.",
  },
  {
    icon: "👨‍🏫",
    title: "Guru Profesional",
    desc: "Semua guru memiliki latar belakang pendidikan musik formal dan pengalaman mengajar anak.",
  },
  {
    icon: "📊",
    title: "Progres Terukur",
    desc: "Laporan perkembangan berkala agar orang tua tahu persis seberapa jauh kemajuan anak.",
  },
  {
    icon: "🎹",
    title: "Studio Berstandar",
    desc: "Ruang kursus berakustik baik dengan instrumen berkualitas untuk pengalaman belajar optimal.",
  },
];

export default function WhyBMD() {
  return (
    <section className="bg-white px-6 py-20 md:px-8 md:py-28">
      <div className="mx-auto max-w-7xl">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-balance font-heading text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
            Mengapa BMD Music School?
          </h2>
          <p className="mt-4 text-lg text-gray-600">
            Bukan hanya kursus musik. Kami membangun fondasi yang benar agar anak mencintai musik seumur hidup.
          </p>
        </div>

        <div className="mt-16 grid grid-cols-1 gap-8 sm:grid-cols-2 lg:grid-cols-4">
          {reasons.map((r, i) => (
            <div key={i} className="group rounded-xl border border-gray-200 bg-gray-50 p-6 transition-all duration-200 hover:border-blue-200 hover:bg-blue-50/50">
              <div className="text-3xl">{r.icon}</div>
              <h3 className="mt-4 font-heading text-lg font-bold text-gray-900">{r.title}</h3>
              <p className="mt-2 text-sm leading-relaxed text-gray-600">{r.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
