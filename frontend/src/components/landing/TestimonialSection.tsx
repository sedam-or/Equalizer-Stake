"use client";

export default function TestimonialSection() {
  const items = [
    { name: "Ananda Putra", role: "Orang Tua", quote: "Sangat membantu mengatur jadwal les anak saya" },
    { name: "Rizky Pratama", role: "Guru Musik", quote: "Sistem absensi memudahkan monitoring kelas" },
    { name: "Putri Wulandari", role: "Wali Murid", quote: "Tagihan otomatis dan notifikasi sangat praktis" },
  ];
  return (
    <section className="px-6 py-24 md:px-8">
      <div className="mx-auto max-w-7xl text-center">
        <h2 className="text-3xl font-bold tracking-tight text-white md:text-4xl">Apa Kata Mereka</h2>
        <div className="mt-12 grid grid-cols-1 gap-6 md:grid-cols-3">
          {items.map((t, i) => (
            <div key={i} className="rounded-2xl border border-[#253900]/30 bg-transparent p-6 text-center">
              <div className="mx-auto flex h-10 w-10 items-center justify-center rounded-full border border-[#08CB00]/30 text-sm font-bold text-[#08CB00]">{t.name[0]}</div>
              <blockquote className="mt-4 text-sm leading-relaxed text-gray-300">&ldquo;{t.quote}&rdquo;</blockquote>
              <div className="mt-4"><p className="text-sm font-semibold text-white">{t.name}</p><p className="text-xs text-gray-500">{t.role}</p></div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
