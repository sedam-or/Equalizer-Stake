export default function CtaSection() {
  return (
    <section className="bg-gradient-to-br from-[#F59E0B] to-[#EA580C] px-6 py-20 md:px-8 md:py-28">
      <div className="mx-auto max-w-3xl text-center">
        <h2 className="font-heading text-3xl font-bold text-white sm:text-4xl">
          Siap Memberikan yang Terbaik untuk Anak?
        </h2>
        <p className="mt-4 text-lg text-white/80">
          Daftar kursus sekarang atau hubungi kami untuk info lebih lanjut.
        </p>
        <div className="mt-8 flex flex-col items-center justify-center gap-4 sm:flex-row">
          <a
            href="/register-course"
            className="inline-flex items-center gap-2 rounded-full bg-white px-8 py-4 text-base font-bold text-[#EA580C] shadow-lg transition-all duration-200 hover:shadow-xl hover:scale-105"
          >
            🎵 Daftar Kursus
          </a>
          <a
            href="https://wa.me/6281234567890?text=Halo%2C%20saya%20ingin%20info%20kursus%20musik%20di%20BMD"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 rounded-full border-2 border-white px-8 py-4 text-base font-semibold text-white transition-all duration-200 hover:bg-white/10"
          >
            💬 Chat via WhatsApp
          </a>
        </div>
      </div>
    </section>
  );
}
