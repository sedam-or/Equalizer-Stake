import SectionHeader from "@/components/landing/SectionHeader";

export default function StudioTourSection() {
  return (
    <section className="bg-white px-6 py-32 md:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="grid grid-cols-1 items-center gap-12 lg:grid-cols-2">
          <div>
            <h2 className="font-heading text-4xl font-extrabold uppercase tracking-tighter text-gray-900 sm:text-5xl md:text-6xl">
              Tentang BMD
            </h2>
            <p className="mt-6 text-lg leading-relaxed text-gray-500">
              BMD Music School didirikan pada tahun 2015 di Seturan, Yogyakarta. Dengan pengalaman lebih dari 10 tahun, kami telah membantu ratusan siswa mengembangkan bakat musik mereka.
            </p>
            <p className="mt-4 text-lg leading-relaxed text-gray-500">
              Kami percaya bahwa musik bukan sekadar hobi, tapi sebuah investasi masa depan.
            </p>
            <div className="mt-8 grid grid-cols-3 gap-6">
              <div><div className="text-3xl font-extrabold text-[#2563EB]">10+</div><div className="mt-1 text-sm text-gray-500">Tahun Pengalaman</div></div>
              <div><div className="text-3xl font-extrabold text-[#2563EB]">500+</div><div className="mt-1 text-sm text-gray-500">Alumni</div></div>
              <div><div className="text-3xl font-extrabold text-[#2563EB]">9</div><div className="mt-1 text-sm text-gray-500">Instrumen</div></div>
            </div>
          </div>
          <div className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-[#2563EB] to-[#7C3AED] p-12 text-center">
            <div aria-hidden="true" className="text-8xl">🎵</div>
            <p className="mt-4 text-xl font-bold text-white">Studio BMD Music School</p>
            <p className="mt-2 text-sm text-white/70">Seturan, Yogyakarta</p>
          </div>
        </div>
      </div>
    </section>
  );
}