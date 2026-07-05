import SectionHeader from "@/components/landing/SectionHeader";

const testimonials = [
  {
    name: "Ibu Rina",
    child: "Andi, Gitar",
    quote: "Dulu Andi malas latihan. Sekarang dia yang nagih mau les. Gurunya tahu cara membuat anak enjoy belajar — bukan dipaksa.",
    stars: 5,
  },
  {
    name: "Bapak Dimas",
    child: "Sari, Piano",
    quote: "Sebagai orang tua yang sibuk, saya sangat terbantu dengan sistem BMD. Jadwal, pembayaran, laporan — semua di satu tempat.",
    stars: 5,
  },
  {
    name: "Ibu Wati",
    child: "Rizky, Drum",
    quote: "Rizky awalnya cuma coba-coba drum. Sekarang dia sudah gabung band sekolah dan sering manggung. BMD membentuk karakter.",
    stars: 5,
  },
];

export default function TestimonialSection() {
  return (
    <section className="bg-white px-6 py-20 md:px-8 md:py-28">
      <div className="mx-auto max-w-7xl">
        <SectionHeader title="Apa Kata Mereka?" subtitle="Dengarkan langsung dari orang tua yang anaknya belajar di BMD." />
        <div className="mt-12 grid grid-cols-1 gap-8 md:grid-cols-3">
          {testimonials.map((t, i) => (
            <div key={i} className="rounded-xl border border-gray-200 bg-white p-6">
              <div aria-label={`${t.stars} dari 5 bintang`} className="flex items-center gap-0.5">
                {Array.from({ length: t.stars }).map((_, j) => (
                  <span key={j} aria-hidden="true" className="text-[#F59E0B]">★</span>
                ))}
              </div>
              <blockquote className="mt-4 text-sm leading-relaxed text-gray-600">
                &ldquo;{t.quote}&rdquo;
              </blockquote>
              <div className="mt-4 flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-[#2563EB]/10 text-sm font-bold text-[#2563EB]">
                  {t.name[0]}
                </div>
                <div>
                  <p className="text-sm font-semibold text-gray-900">{t.name}</p>
                  <p className="text-xs text-gray-500">Orang tua {t.child}</p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
