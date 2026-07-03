"use client";

export default function Footer() {
  return (
    <footer id="hubungi" className="border-t border-[#CBD5E1]/50 px-6 py-12 md:px-8">
      <div className="mx-auto max-w-7xl text-center">
        <div className="grid grid-cols-1 gap-8 md:grid-cols-3">
          <div><h3 className="text-lg font-bold text-[#2563EB] flex items-center justify-center gap-2"><img src="/logo.jpg" alt="BMD Music" className="h-6 w-auto" />BMD MUSIC SCHOOL</h3><p className="mt-2 text-sm text-gray-400">Sistem Penjadwalan Kursus Musik</p></div>
          <div><h4 className="text-sm font-semibold text-white">Kontak</h4><p className="mt-2 text-sm text-gray-400">Seturan, Yogyakarta</p><p className="mt-1 text-sm text-[#2563EB]">+62 xxx-xxxx-xxxx</p><p className="mt-1 text-sm text-[#2563EB]">info@bmdmusic.com</p></div>
          <div><h4 className="text-sm font-semibold text-white">Navigasi</h4><div className="mt-2 flex flex-col gap-2"><a href="#layanan" className="text-sm text-gray-400 hover:text-[#2563EB]">Layanan</a><a href="#studio" className="text-sm text-gray-400 hover:text-[#2563EB]">Studio</a><a href="#hubungi" className="text-sm text-gray-400 hover:text-[#2563EB]">Hubungi</a><a href="/login" className="text-sm text-gray-400 hover:text-[#2563EB]">Masuk</a></div></div>
        </div>
        <div className="mt-8 border-t border-[#CBD5E1]/30 pt-4 text-center text-xs text-gray-500">© 2026 BMD Music School. All rights reserved.</div>
      </div>
    </footer>
  );
}
