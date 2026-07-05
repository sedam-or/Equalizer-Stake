export default function Footer() {
  return (
    <footer id="kontak" className="bg-gray-900 px-6 py-16 md:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="grid grid-cols-1 gap-12 md:grid-cols-4">
          {/* Brand */}
          <div>
            <h3 className="font-heading text-xl font-bold text-white">BMD Music School</h3>
            <p className="mt-4 text-sm leading-relaxed text-gray-600">
              Sistem Penjadwalan Kursus Musik Berbasis Web untuk BMD Music School, Yogyakarta.
            </p>
          </div>

          {/* Navigation */}
          <div>
            <h4 className="text-sm font-semibold text-gray-300">Navigasi</h4>
            <div className="mt-4 flex flex-col gap-3">
              <a href="#instrumen" className="text-sm text-gray-600 transition-colors hover:text-white">Instrumen</a>
              <a href="#tentang" className="text-sm text-gray-600 transition-colors hover:text-white">Tentang</a>
              <a href="#harga" className="text-sm text-gray-600 transition-colors hover:text-white">Harga</a>
              <a href="/register-course" className="text-sm text-gray-600 transition-colors hover:text-white">Daftar Kursus</a>
            </div>
          </div>

          {/* Contact */}
          <div>
            <h4 className="text-sm font-semibold text-gray-300">Kontak</h4>
            <address className="mt-4 flex flex-col gap-3 not-italic">
              <p className="text-sm text-gray-600">📱 +62 812-3456-7890</p>
              <p className="text-sm text-gray-600">✉️ info@bmdmusic.com</p>
              <p className="text-sm text-gray-600">📍 Seturan, Yogyakarta</p>
            </address>
          </div>

          {/* Hours + Studio Booking */}
          <div>
            <h4 className="text-sm font-semibold text-gray-300">Jam Operasional</h4>
            <div className="mt-4 flex flex-col gap-3">
              <p className="text-sm text-gray-600">Senin - Sabtu</p>
              <p className="text-sm text-gray-600">09:00 - 20:00 WIB</p>
              <p className="text-sm text-gray-600">Minggu: Libur</p>
            </div>
            <a
              href="https://wa.me/6281234567890?text=Halo%2C%20saya%20ingin%20booking%20studio%20BMD"
              target="_blank"
              rel="noopener noreferrer"
              className="mt-4 inline-flex items-center gap-2 rounded-lg bg-[#2563EB] px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-[#1D4ED8]"
            >
              🎹 Booking Studio
            </a>
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
