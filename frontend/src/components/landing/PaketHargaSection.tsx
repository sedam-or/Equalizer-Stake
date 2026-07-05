import SectionHeader from "@/components/landing/SectionHeader";

const packages = [
  {
    name: "Trial",
    price: "Rp 150.000",
    period: "/1 sesi",
    features: ["1x pertemuan (60 menit)", "1 instrumen pilihan", "Konsultasi level", "Tanpa komitmen"],
    popular: false,
  },
  {
    name: "Reguler",
    price: "Rp 350.000",
    period: "/bulan",
    features: ["2x pertemuan/minggu", "45 menit/sesi", "1 instrumen", "Monitoring online", "Laporan bulanan"],
    popular: true,
  },
  {
    name: "Premium",
    price: "Rp 500.000",
    period: "/bulan",
    features: ["3x pertemuan/minggu", "60 menit/sesi", "2 instrumen", "Free studio rehearsal", "Recording session", "Laporan bulanan"],
    popular: false,
  },
];

export default function PaketHargaSection() {
  return (
    <section id="harga" className="bg-[#F8FAFC] px-6 py-20 md:px-8 md:py-28">
      <div className="mx-auto max-w-7xl">
        <SectionHeader title="Investasi untuk Masa Depan Anak" subtitle="Mulai dari Rp 150.000 untuk sesi trial. Tidak cocok? Uang kembali." />
        <div className="mt-12 grid grid-cols-1 gap-8 md:grid-cols-3">
          {packages.map((pkg, i) => (
            <div key={i} className={`relative rounded-xl border p-6 text-center ${pkg.popular ? "border-[#2563EB] bg-white shadow-md" : "border-gray-200 bg-white"}`}>
              {pkg.popular && (
                <span className="absolute -top-3 left-1/2 -translate-x-1/2 rounded-full bg-[#EFF6FF] px-4 py-1 text-xs font-bold text-[#2563EB]">
                  Populer
                </span>
              )}
              <h3 className="font-heading text-lg font-bold text-gray-900">{pkg.name}</h3>
              <div className="mt-3">
                <span className="text-3xl font-bold text-[#2563EB]">{pkg.price}</span>
                <span className="text-sm text-gray-500">{pkg.period}</span>
              </div>
              <ul className="mt-6 space-y-2 text-left">
                {pkg.features.map((f, j) => (
                  <li key={j} className="flex items-center gap-2 text-sm text-gray-600">
                    <span className="text-[#2563EB]">✓</span> {f}
                  </li>
                ))}
              </ul>
              <a
                href="/register-course"
                className={`mt-6 inline-flex w-full items-center justify-center rounded-full py-3 text-sm font-bold transition-all duration-200 ${pkg.popular ? "bg-[#F59E0B] text-black hover:bg-[#D97706]" : "border border-gray-300 bg-white text-gray-900 hover:bg-gray-50"}`}
              >
                {pkg.popular ? "🎵 Daftar Sekarang" : "Daftar Kursus"}
              </a>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
