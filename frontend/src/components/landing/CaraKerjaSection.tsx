"use client";

export default function CaraKerjaSection() {
  const steps = [
    { num: "01", title: "Daftar & Pilih Instrumen", desc: "Buat akun dan pilih kursus musik yang diinginkan" },
    { num: "02", title: "Pilih Jadwal Otomatis", desc: "Sistem memilihkan jadwal terbaik tanpa konflik" },
    { num: "03", title: "Pantau Progress Anak", desc: "Monitoring kehadiran dan pembayaran real-time" },
  ];
  return (
    <section className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl text-center">
        <h2 className="text-3xl font-bold tracking-tight text-white md:text-4xl">Mulai dalam 3 Langkah</h2>
        <div className="mt-12 grid grid-cols-1 gap-6 md:grid-cols-3">
          {steps.map((s, i) => (
            <div key={i} className="rounded-2xl border border-[#08CB00]/30 bg-transparent p-6 text-center">
              <div className="text-4xl font-bold text-[#08CB00]">{s.num}</div>
              <h3 className="mt-4 text-lg font-semibold text-white">{s.title}</h3>
              <p className="mt-2 text-sm leading-relaxed text-gray-300">{s.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
