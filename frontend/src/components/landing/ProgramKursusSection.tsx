import SectionHeader from "@/components/landing/SectionHeader";

const instruments = [
  { name: "Piano", emoji: "🎹", price: 350000, desc: "Dasar musik yang kuat untuk semua genre" },
  { name: "Gitar", emoji: "🎸", price: 300000, desc: "Paling populer, langsung bisa main lagu" },
  { name: "Biola", emoji: "🎻", price: 400000, desc: "Kembangkan ketelitian dan ekspresi" },
  { name: "Drum", emoji: "🥁", price: 300000, desc: "Salurkan energi anak secara positif" },
  { name: "Vokal", emoji: "🎤", price: 250000, desc: "Percaya diri tampil di depan umum" },
  { name: "Keyboard", emoji: "🎹", price: 300000, desc: "Fleksibel, bisa bermain di mana saja" },
  { name: "Saxophone", emoji: "🎷", price: 400000, desc: "Unik, beda dari yang lain" },
  { name: "Flute", emoji: "🪈", price: 350000, desc: "Elegan, melatih pernapasan" },
  { name: "Bass", emoji: "🎸", price: 300000, desc: "Backbone setiap band, selalu dibutuhkan" },
];

export default function ProgramKursusSection() {
  return (
    <section id="instrumen" className="bg-white px-6 py-20 md:px-8 md:py-28">
      <div className="mx-auto max-w-7xl">
        <SectionHeader title="Pilih Instrumen Favorit Anak Anda" subtitle="9 instrumen tersedia. Anak Anda bebas memilih yang paling disukai." />
        <div className="mt-12 grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {instruments.map((inst) => (
            <div key={inst.name} className="group rounded-xl border border-gray-200 bg-white p-6 transition-all duration-200 hover:-translate-y-1 hover:shadow-lg">
              <span aria-hidden="true" className="text-4xl">{inst.emoji}</span>
              <h3 className="mt-3 font-heading text-lg font-bold text-gray-900">{inst.name}</h3>
              <p className="mt-1 text-sm text-gray-500">{inst.desc}</p>
              <div className="mt-4 flex items-center justify-between">
                <p className="text-base font-bold text-[#2563EB]">
                  Rp {inst.price.toLocaleString("id-ID")}
                  <span className="text-sm font-normal text-gray-500">/bulan</span>
                </p>
                <a
                  href={`/register-course?instrument=${encodeURIComponent(inst.name)}`}
                  className="rounded-lg bg-[#2563EB]/10 px-3 py-1.5 text-xs font-semibold text-[#2563EB] transition-colors hover:bg-[#2563EB] hover:text-white"
                >
                  Pelajari
                </a>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
