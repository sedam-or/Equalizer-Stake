"use client";

export default function Footer() {
  return (
    <footer id="hubungi" className="border-t border-[#253900]/50 px-6 py-12 md:px-8">
      <div className="mx-auto max-w-7xl text-center">
        <div className="grid grid-cols-1 gap-8 md:grid-cols-3">
          <div><h3 className="text-lg font-bold text-[#08CB00]">BMD MUSIC SCHOOL</h3><p className="mt-2 text-sm text-gray-400">Sistem Penjadwalan Kursus Musik</p></div>
          <div><h4 className="text-sm font-semibold text-white">Kontak</h4><p className="mt-2 text-sm text-gray-400">Seturan, Yogyakarta</p><p className="mt-1 text-sm text-[#08CB00]">+62 xxx-xxxx-xxxx</p><p className="mt-1 text-sm text-[#08CB00]">info@bmdmusic.com</p></div>
          <div><h4 className="text-sm font-semibold text-white">Navigasi</h4><div className="mt-2 flex flex-col gap-2"><a href="#layanan" className="text-sm text-gray-400 hover:text-[#08CB00]">Layanan</a><a href="#studio" className="text-sm text-gray-400 hover:text-[#08CB00]">Studio</a><a href="/login" className="text-sm text-gray-400 hover:text-[#08CB00]">Masuk</a><a href="/register" className="text-sm text-gray-400 hover:text-[#08CB00]">Daftar</a><a href="/forgot-password" className="text-sm text-gray-400 hover:text-[#08CB00]">Lupa Password</a><a href="/dashboard" className="text-sm text-gray-400 hover:text-[#08CB00]">Dashboard</a></div></div>
        </div>
        <div className="mt-8 border-t border-[#253900]/30 pt-4 text-center text-xs text-gray-500">© 2026 BMD Music School. All rights reserved.</div>
      </div>
    </footer>
  );
}
