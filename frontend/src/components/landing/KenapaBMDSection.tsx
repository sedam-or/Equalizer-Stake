import SectionHeader from "@/components/landing/SectionHeader";

const benefits = [
  { icon: "👨‍🏫", title: "Guru Profesional", description: "Guru berpengalaman dengan sertifikasi nasional. Metode belajar menyenangkan dan terstruktur." },
  { icon: "📅", title: "Jadwal Fleksibel", description: "Pilih jadwal yang sesuai aktivitas Anda. Senin-Sabtu, 09:00-20:00 WIB." },
  { icon: "🏠", title: "Studio Lengkap", description: "Studio berstandar profesional dengan peralatan lengkap. Nyaman untuk belajar." },
];

export default function KenapaBMDSection() {
  return (
    <section id="tentang" className="bg-[#F8FAFC] px-6 py-32 md:px-8">
      <div className="mx-auto max-w-7xl">
        <SectionHeader title="Kenapa BMD?" subtitle="Alasan ribuan orang tua memilih kami" />
        <div className="mt-16 grid grid-cols-1 gap-8 md:grid-cols-3">
          {benefits.map((b) => (
            <div key={b.title} className="rounded-xl border border-gray-200 bg-white p-8 text-center transition-all duration-200 hover:-translate-y-1 hover:shadow-lg">
              <span aria-hidden="true" className="text-5xl">{b.icon}</span>
              <h3 className="mt-4 font-heading text-xl font-bold text-gray-900">{b.title}</h3>
              <p className="mt-3 text-sm leading-relaxed text-gray-500">{b.description}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}