export default function CtaSection() {
  return (
    <section className="bg-gradient-to-br from-[#F59E0B] to-[#EA580C] px-6 py-20 md:px-8 md:py-28">
      <div className="mx-auto max-w-3xl text-center">
        <h2 className="font-heading text-3xl font-bold text-white sm:text-4xl">
          Siap Memberikan yang Terbaik untuk Anak?
        </h2>
        <p className="mt-4 text-lg text-white/80">
          Daftar kursus musik sekarang dan wujudkan potensi anak Anda.
        </p>
        <div className="mt-8">
          <a
            href="/register-course"
            className="inline-flex items-center gap-2 rounded-full bg-white px-10 py-4 text-base font-bold text-[#EA580C] shadow-lg transition-all duration-200 hover:shadow-xl hover:scale-105"
          >
            🎵 Daftar Kursus Sekarang
          </a>
        </div>
      </div>
    </section>
  );
}
