import SectionHeader from "@/components/landing/SectionHeader";

const teachers = [
  { name: "Pak Dimas", instrument: "Gitar & Piano", experience: "12 tahun mengajar", photo: "/images/studio-2.jpg" },
  { name: "Mbak Sari", instrument: "Vokal & Biola", experience: "10 tahun mengajar", photo: "/images/studio-3.jpg" },
  { name: "Pak Rizky", instrument: "Drum & Bass", experience: "8 tahun mengajar", photo: "/images/studio-4.jpg" },
];

const gallery = [
  { src: "/images/studio-2.jpg", alt: "Ruang les piano" },
  { src: "/images/studio-3.jpg", alt: "Ruang les gitar" },
  { src: "/images/studio-4.jpg", alt: "Studio rekaman" },
  { src: "/images/studio-5.jpg", alt: "Ruang tunggu" },
];

export default function TentangSection() {
  return (
    <section id="tentang" className="bg-[#F8FAFC] px-6 py-20 md:px-8 md:py-28">
      <div className="mx-auto max-w-7xl">
        <SectionHeader title="Tentang BMD" subtitle="Bukan hanya kursus musik. Kami membangun karakter anak melalui musik." />

        {/* Story */}
        <div className="mx-auto mt-12 max-w-3xl text-center">
          <p className="text-lg leading-relaxed text-gray-600">
            Dimulai dari sebuah ruangan kecil di Seturan, Yogyakarta, BMD Music School kini telah menjadi rumah bagi 500+ alumni.
            Kami percaya musik bukan sekadar hobi — ia adalah bahasa universal yang membentuk karakter, disiplin, dan kreativitas anak.
          </p>
        </div>

        {/* Stats */}
        <div className="mt-12 grid grid-cols-3 gap-6">
          <div className="text-center">
            <div className="text-3xl font-bold text-[#2563EB]">10+</div>
            <div className="mt-1 text-sm text-gray-500">Tahun Pengalaman</div>
          </div>
          <div className="text-center">
            <div className="text-3xl font-bold text-[#2563EB]">500+</div>
            <div className="mt-1 text-sm text-gray-500">Alumni</div>
          </div>
          <div className="text-center">
            <div className="text-3xl font-bold text-[#2563EB]">9</div>
            <div className="mt-1 text-sm text-gray-500">Instrumen</div>
          </div>
        </div>

        {/* Teacher profiles */}
        <div className="mt-16">
          <h3 className="text-center font-heading text-2xl font-bold text-gray-900">Guru Kami</h3>
          <p className="mt-2 text-center text-gray-500">Berpengalaman, sabar, dan mencintai mengajar</p>
          <div className="mt-8 grid grid-cols-1 gap-6 sm:grid-cols-3">
            {teachers.map((teacher) => (
              <div key={teacher.name} className="rounded-xl border border-gray-200 bg-white p-6 text-center">
                <div className="mx-auto h-24 w-24 overflow-hidden rounded-full">
                  <img src={teacher.photo} alt={teacher.name} className="h-full w-full object-cover" />
                </div>
                <h4 className="mt-4 font-heading text-lg font-bold text-gray-900">{teacher.name}</h4>
                <p className="text-sm text-[#2563EB]">{teacher.instrument}</p>
                <p className="text-xs text-gray-500">{teacher.experience}</p>
              </div>
            ))}
          </div>
        </div>

        {/* Studio gallery */}
        <div className="mt-16">
          <h3 className="text-center font-heading text-2xl font-bold text-gray-900">Studio Kami</h3>
          <p className="mt-2 text-center text-gray-500">Peralatan profesional, ruangan nyaman</p>
          <div className="mt-8 grid grid-cols-2 gap-4 lg:grid-cols-4">
            {gallery.map((img) => (
              <div key={img.src} className="overflow-hidden rounded-xl">
                <img src={img.src} alt={img.alt} className="h-48 w-full object-cover transition-transform duration-300 hover:scale-105" />
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
