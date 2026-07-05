"use client";

export default function Footer() {
  return (
    <footer className="bg-[#0A0F1A] px-6 py-16 md:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="grid gap-12 md:grid-cols-3">
          {/* Brand */}
          <div>
            <a href="/" className="font-heading text-xl font-bold tracking-tight text-white">
              BMD <span className="text-[#D4AF37]">Music</span>
            </a>
            <p className="mt-4 max-w-xs text-sm leading-relaxed text-gray-500">
              Kelas privat musik premium untuk keluarga di Yogyakarta. Pengalaman 10+ tahun mengembangkan bakat musik anak.
            </p>
          </div>

          {/* Links */}
          <div>
            <h4 className="text-sm font-semibold text-gray-400">Program</h4>
            <ul className="mt-4 space-y-3">
              <li><a href="/instruments" className="text-sm text-gray-500 transition-colors hover:text-white">Semua Instrumen</a></li>
              <li><a href="/register-course" className="text-sm text-gray-500 transition-colors hover:text-white">Daftar Kursus</a></li>
              <li><a href="/help" className="text-sm text-gray-500 transition-colors hover:text-white">FAQ</a></li>
            </ul>
          </div>

          {/* Contact */}
          <div>
            <h4 className="text-sm font-semibold text-gray-400">Kontak</h4>
            <ul className="mt-4 space-y-3">
              <li className="text-sm text-gray-500">📍 Seturan, Yogyakarta</li>
              <li className="text-sm text-gray-500">📞 0812-3456-7890</li>
              <li className="text-sm text-gray-500">⏰ Sen-Sab, 08:00-20:00</li>
            </ul>
          </div>
        </div>

        {/* Bottom */}
        <div className="mt-12 border-t border-white/5 pt-8 text-center">
          <p className="text-xs text-gray-600">
            © 2026 BMD Music School. Hak cipta dilindungi.
          </p>
        </div>
      </div>
    </footer>
  );
}
