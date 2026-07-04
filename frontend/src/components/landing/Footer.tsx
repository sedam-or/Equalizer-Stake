"use client";

export default function Footer() {
  return (
    <footer id="kontak" className="bg-gray-900 px-6 py-12 md:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="grid grid-cols-1 gap-8 md:grid-cols-4">
          {/* Brand */}
          <div>
            <h3 className="font-heading text-lg font-bold text-white">BMD Music School</h3>
            <p className="mt-2 text-sm text-gray-400">Sistem Penjadwalan Kursus Musik</p>
            <p className="mt-2 text-sm text-gray-400">Seturan, Yogyakarta</p>
          </div>

          {/* Quick Links */}
          <div>
            <h4 className="text-sm font-semibold text-white">Navigasi</h4>
            <div className="mt-3 flex flex-col gap-2">
              <a href="#instrumen" className="text-sm text-gray-400 hover:text-white transition-colors">Instrumen</a>
              <a href="#tentang" className="text-sm text-gray-400 hover:text-white transition-colors">Tentang</a>
              <a href="#harga" className="text-sm text-gray-400 hover:text-white transition-colors">Harga</a>
              <a href="#kontak" className="text-sm text-gray-400 hover:text-white transition-colors">Kontak</a>
            </div>
          </div>

          {/* Contact */}
          <div>
            <h4 className="text-sm font-semibold text-white">Kontak</h4>
            <div className="mt-3 flex flex-col gap-2">
              <p className="text-sm text-gray-400">📱 +62 812-3456-7890</p>
              <p className="text-sm text-gray-400">✉️ info@bmdmusic.com</p>
              <p className="text-sm text-gray-400">📍 Seturan, Yogyakarta</p>
            </div>
          </div>

          {/* Social */}
          <div>
            <h4 className="text-sm font-semibold text-white">Jam Operasional</h4>
            <div className="mt-3 flex flex-col gap-2">
              <p className="text-sm text-gray-400">Senin - Sabtu</p>
              <p className="text-sm text-gray-400">09:00 - 20:00 WIB</p>
              <p className="text-sm text-gray-400">Minggu: Libur</p>
            </div>
          </div>
        </div>

        {/* Bottom bar */}
        <div className="mt-8 border-t border-gray-700 pt-6 text-center">
          <p className="text-xs text-gray-500">© 2026 BMD Music School. All rights reserved.</p>
        </div>
      </div>
    </footer>
  );
}
