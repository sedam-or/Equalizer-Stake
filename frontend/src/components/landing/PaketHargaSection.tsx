"use client";

const packages = [
  {
    name: "Trial",
    price: "Rp 150.000",
    period: "/1 sesi",
    features: ["1x pertemuan (60 menit)", "1 instrumen pilihan", "Konsultasi level", "Sertifikat试课"],
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
    <section id="harga" className="bg-white px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl text-center">
        <h2 className="font-heading text-3xl font-bold tracking-tight text-gray-900 md:text-4xl">
          Paket & Harga
        </h2>
        <p className="mt-3 text-base text-gray-500 md:text-lg">
          Pilih paket yang sesuai kebutuhan Anda
        </p>
        <div className="mt-12 grid grid-cols-1 gap-6 md:grid-cols-3">
          {packages.map((pkg, i) => (
            <div
              key={i}
              className={`rounded-xl border p-6 text-center transition-all duration-200 hover:translate-y-[-2px] hover:shadow-lg ${
                pkg.popular
                  ? "border-[#2563EB] bg-white shadow-md"
                  : "border-gray-200 bg-white"
              }`}
            >
              {pkg.popular && (
                <span className="mb-4 inline-block rounded-full bg-[#EFF6FF] px-3 py-1 text-xs font-bold text-[#2563EB]">
                  POPULER
                </span>
              )}
              <h3 className="font-heading text-xl font-bold text-gray-900">{pkg.name}</h3>
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
              <div className="mt-6">
                <a
                  href={`https://wa.me/6281234567890?text=Halo%2C%20saya%20ingin%20daftar%20${encodeURIComponent(pkg.name)}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className={`inline-flex w-full items-center justify-center rounded-full py-3 text-sm font-bold transition-all ${
                    pkg.popular
                      ? "bg-[#F59E0B] text-black hover:bg-[#D97706]"
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
