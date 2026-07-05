import SectionHeader from "@/components/landing/SectionHeader";

const instruments = [
  {
    name: "Piano Klasik",
    emoji: "🎹",
    registrationFee: 420000,
    classes: [
      { type: "Beginner", price: 420000, duration: "30 menit" },
      { type: "Grade 1-2", price: 480000, duration: "45 menit" },
      { type: "Grade 3-4", price: 540000, duration: "45 menit" },
      { type: "Grade 5-6", price: 630000, duration: "45 menit" },
    ],
  },
  {
    name: "Piano Pop",
    emoji: "🎹",
    registrationFee: 420000,
    classes: [
      { type: "Umum", price: 600000, duration: "45 menit" },
    ],
  },
  {
    name: "Vokal",
    emoji: "🎤",
    registrationFee: 420000,
    classes: [
      { type: "Private (1 siswa)", price: 600000, duration: "45 menit" },
      { type: "Intensif (2 siswa)", price: 390000, duration: "45 menit" },
      { type: "Semi Intensif (4 siswa)", price: 300000, duration: "45 menit" },
    ],
  },
  {
    name: "Biola",
    emoji: "🎻",
    registrationFee: 540000,
    classes: [
      { type: "Private", price: 420000, duration: "45 menit" },
    ],
  },
  {
    name: "Gitar Akustik",
    emoji: "🎸",
    registrationFee: 420000,
    classes: [
      { type: "Private", price: 420000, duration: "45 menit" },
    ],
  },
  {
    name: "Keyboard",
    emoji: "🎹",
    registrationFee: 480000,
    classes: [
      { type: "Private", price: 420000, duration: "45 menit" },
    ],
  },
  {
    name: "Drum",
    emoji: "🥁",
    registrationFee: 420000,
    classes: [
      { type: "Private", price: 420000, duration: "45 menit" },
    ],
  },
  {
    name: "Flute",
    emoji: "🪈",
    registrationFee: 420000,
    classes: [
      { type: "Private", price: 420000, duration: "45 menit" },
    ],
  },
  {
    name: "Saxophone",
    emoji: "🎷",
    registrationFee: 420000,
    classes: [
      { type: "Private", price: 420000, duration: "45 menit" },
    ],
  },
  {
    name: "Paket Liburan",
    emoji: "🏖️",
    registrationFee: 60000,
    classes: [
      { type: "Kelas A (1 siswa)", price: 780000, duration: "45 menit" },
      { type: "Kelas B (2 siswa)", price: 900000, duration: "45 menit" },
      { type: "Kelas C (3 siswa)", price: 1020000, duration: "45 menit" },
      { type: "Kelas D (4 siswa)", price: 1260000, duration: "45 menit" },
    ],
  },
];

function formatRupiah(amount: number): string {
  return `Rp ${amount.toLocaleString("id-ID")}`;
}

export default function HargaSection() {
  return (
    <section id="harga" className="bg-[#F8FAFC] px-6 py-20 md:px-8 md:py-28">
      <div className="mx-auto max-w-7xl">
        <SectionHeader
          title="Investasi untuk Masa Depan Anak"
          subtitle="Biaya transparan, tanpa biaya tersembunyi. Registration fee dibayar sekali saat daftar."
        />

        <div className="mt-12 grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {instruments.map((inst) => (
            <div
              key={inst.name}
              className="rounded-xl border border-gray-200 bg-white p-6 transition-all duration-200 hover:-translate-y-1 hover:shadow-lg"
            >
              {/* Header */}
              <div className="flex items-center gap-3">
                <span aria-hidden="true" className="text-3xl">{inst.emoji}</span>
                <div>
                  <h3 className="font-heading text-lg font-bold text-gray-900">{inst.name}</h3>
                  <p className="text-xs text-gray-500">
                    Registration Fee: {formatRupiah(inst.registrationFee)}
                  </p>
                </div>
              </div>

              {/* Class types */}
              <div className="mt-4 space-y-2">
                {inst.classes.map((cls) => (
                  <div
                    key={cls.type}
                    className="flex items-center justify-between rounded-lg border border-gray-100 px-3 py-2"
                  >
                    <div>
                      <p className="text-sm font-medium text-gray-900">{cls.type}</p>
                      <p className="text-xs text-gray-500">{cls.duration}</p>
                    </div>
                    <p className="text-sm font-bold text-[#2563EB]">
                      {formatRupiah(cls.price)}
                      <span className="text-xs font-normal text-gray-500">/bulan</span>
                    </p>
                  </div>
                ))}
              </div>

              {/* CTA */}
              <a
                href={`/register-course?instrument=${encodeURIComponent(inst.name)}`}
                className="mt-4 inline-flex w-full items-center justify-center rounded-lg bg-[#2563EB] px-4 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-[#1D4ED8]"
              >
                Daftar {inst.name}
              </a>
            </div>
          ))}
        </div>

        {/* Info */}
        <div className="mt-8 rounded-xl border border-gray-200 bg-white p-6 text-center">
          <p className="text-sm text-gray-600">
            <strong>Catatan:</strong> Registration fee dibayar sekali saat pendaftaran.
            SPP dibayar setiap bulan. Harga sudah termasuk materi pembelajaran dan monitoring online.
          </p>
        </div>
      </div>
    </section>
  );
}
