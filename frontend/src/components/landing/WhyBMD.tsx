"use client";

const reasons = [
  {
    number: "01",
    title: "Metode Terstruktur",
    desc: "Kurikulum yang dirancang khusus untuk perkembangan musik anak — bukan sekadar mengajarkan notasi, tapi membangun kecintaan pada musik seumur hidup.",
  },
  {
    number: "02",
    title: "Guru Berstandar Internasional",
    desc: "Semua guru memiliki latar belakang pendidikan musik formal dan pengalaman mengajar anak. Metode yang sabar, terstruktur, dan menyenangkan.",
  },
  {
    number: "03",
    title: "Progres yang Terukur",
    desc: "Laporan perkembangan berkala agar Anda tahu persis seberapa jauh kemajuan anak. Transparan, tanpa kejutan.",
  },
];

export default function WhyBMD() {
  return (
    <section data-theme="light" className="snap-section flex min-[100dvh] items-center bg-white px-6 md:px-8">
      <div className="mx-auto w-full max-w-7xl">
        <div className="grid items-center gap-16 lg:grid-cols-2">
          {/* Left: Text */}
          <div>
            <p className="mb-4 text-sm font-semibold uppercase tracking-[0.3em] text-[#D4AF37]">
              Mengapa BMD
            </p>
            <h2 className="text-balance font-heading text-3xl font-bold leading-tight tracking-tight text-[#0F172A] sm:text-4xl lg:text-5xl">
              Bukan Sekadar Kursus Musik.
              <br />
              <span className="text-gray-400">Investasi untuk Masa Depan.</span>
            </h2>
            <p className="mt-6 max-w-md text-base leading-relaxed text-gray-500">
              Kami tidak hanya mengajarkan anak bermain instrumen. Kami membangun fondasi yang benar agar anak mencintai musik seumur hidup.
            </p>
          </div>

          {/* Right: Points */}
          <div className="space-y-8">
            {reasons.map((r) => (
              <div
                key={r.number}
                className="group flex gap-6 border-b border-gray-100 pb-8 transition-all duration-300 last:border-b-0 hover:border-[#D4AF37]/30"
              >
                <span className="font-heading text-sm font-bold text-[#D4AF37]/40 transition-colors duration-300 group-hover:text-[#D4AF37]">
                  {r.number}
                </span>
                <div>
                  <h3 className="font-heading text-lg font-bold text-[#0F172A] transition-colors duration-300 group-hover:text-[#2563EB]">
                    {r.title}
                  </h3>
                  <p className="mt-2 text-sm leading-relaxed text-gray-500">{r.desc}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
