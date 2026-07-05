"use client";

const packages = [
  {
    name: "Trial",
    price: "Rp 150.000",
    period: "/1 sesi",
    features: ["1x pertemuan (60 menit)", "1 instrumen pilihan", "Konsultasi level", "Sertifikat Trial"],
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
    <section id="harga" className="bg-white px-6 py-32 md:px-8">
      <div className="mx-auto max-w-7xl">
        {/* Section header */}
        <div className="text-center">
          <h2 className="font-heading text-4xl font-extrabold uppercase tracking-tighter text-gray-900 sm:text-5xl md:text-6xl">
            Paket & Harga
          </h2>
          <p className="mt-4 text-lg text-gray-500 md:text-xl">
            Pilih paket yang sesuai kebutuhan Anda
          </p>
        </div>

        {/* Pricing cards */}
        <div className="mt-16 grid grid-cols-1 gap-8 md:grid-cols-3">
          {packages.map((pkg, i) => (
            <div
              key={i}
              className={`relative rounded-xl border p-8 text-center transition-all duration-200 hover:-translate-y-1 hover:shadow-lg ${
                pkg.popular
                  ? "border-[#2563EB] bg-white shadow-md"
                  : "border-gray-200 bg-white"
              }`}
            >
              {pkg.popular && (
                <span className="absolute -top-3 left-1/2 -translate-x-1/2 rounded-full bg-[#EFF6FF] px-4 py-1 text-xs font-bold uppercase tracking-wider text-[#2563EB]">
                  Populer
                </span>
              )}
              <h3 className="font-heading text-xl font-bold text-gray-900">{pkg.name}</h3>
              <div className="mt-4">
                <span className="text-4xl font-extrabold text-[#2563EB]">{pkg.price}</span>
                <span className="text-sm text-gray-500">{pkg.period}</span>
              </div>
              <ul className="mt-6 space-y-3 text-left">
                {pkg.features.map((f, j) => (
                  <li key={j} className="flex items-center gap-3 text-sm text-gray-600">
                    <span className="flex h-5 w-5 items-center justify-center rounded-full bg-[#2563EB]/10 text-xs text-[#2563EB]">✓</span>
                    {f}
                  </li>
                ))}
              </ul>
              <div className="mt-8">
                <a
                  href={`https://wa.me/6281234567890?text=Halo%2C%20saya%20ingin%20daftar%20${encodeURIComponent(pkg.name)}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className={`inline-flex w-full items-center justify-center rounded-full py-3.5 text-sm font-bold uppercase tracking-wide transition-all duration-200 ${
                    pkg.popular
                      ? "bg-[#F59E0B] text-black hover:bg-[#D97706] hover:shadow-lg"
                      : "border border-gray-300 bg-white text-gray-900 hover:bg-gray-50"
                  }`}
                >
                  {pkg.popular ? "🎵 Daftar Sekarang" : "Pilih Paket"}
                </a>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
