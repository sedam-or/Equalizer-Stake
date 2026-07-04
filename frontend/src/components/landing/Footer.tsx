"use client";

export default function Footer() {
  return (
    <footer id="kontak" className="bg-gray-900 px-6 py-16 md:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="grid grid-cols-1 gap-12 md:grid-cols-4">
          {/* Brand */}
          <div>
            <h3 className="font-heading text-xl font-extrabold uppercase tracking-tight text-white">
              BMD Music School
            </h3>
            <p className="mt-4 text-sm leading-relaxed text-gray-400">
              Sistem Penjadwalan Kursus Musik Berbasis Web untuk BMD Music School, Yogyakarta.
            </p>
          </div>

          {/* Quick Links */}
          <div>
            <h4 className="text-sm font-bold uppercase tracking-wider text-white">Navigasi</h4>
            <div className="mt-4 flex flex-col gap-3">
              <a href="#instrumen" className="text-sm text-gray-400 transition-colors hover:text-white">Instrumen</a>
              <a href="#tentang" className="text-sm text-gray-400 transition-colors hover:text-white">Tentang</a>
              <a href="#harga" className="text-sm text-gray-400 transition-colors hover:text-white">Harga</a>
              <a href="#kontak" className="text-sm text-gray-400 transition-colors hover:text-white">Kontak</a>
            </div>
          </div>

          {/* Contact */}
          <div>
            <h4 className="text-sm font-bold uppercase tracking-wider text-white">Kontak</h4>
            <div className="mt-4 flex flex-col gap-3">
              <p className="text-sm text-gray-400">📱 +62 812-3456-7890</p>
              <p className="text-sm text-gray-400">✉️ info@bmdmusic.com</p>
              <p className="text-sm text-gray-400">📍 Seturan, Yogyakarta</p>
            </div>
          </div>

          {/* Hours */}
          <div>
            <h4 className="text-sm font-bold uppercase tracking-wider text-white">Jam Operasional</h4>
            <div className="mt-4 flex flex-col gap-3">
              <p className="text-sm text-gray-400">Senin - Sabtu</p>
              <p className="text-sm text-gray-400">09:00 - 20:00 WIB</p>
              <p className="text-sm text-gray-400">Minggu: Libur</p>
            </div>
          </div>
        </div>

        {/* Bottom bar */}
        <div className="mt-12 border-t border-gray-700 pt-8 text-center">
          <p className="text-xs text-gray-500">© 2026 BMD Music School. All rights reserved.</p>
        </div>
      </div>
    </footer>
  );
}
