export default function DashboardPage() {
  return (
    <div>
      <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
      <p className="mt-1 text-sm text-gray-500">Selamat datang di Equalizer-Stake</p>

      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-xl bg-white p-6 shadow-sm">
          <div className="text-sm text-gray-500">Sesi Hari Ini</div>
          <div className="mt-1 text-3xl font-bold text-[#1E40AF]">12</div>
        </div>
        <div className="rounded-xl bg-white p-6 shadow-sm">
          <div className="text-sm text-gray-500">Pending Verifikasi</div>
          <div className="mt-1 text-3xl font-bold text-[#F59E0B]">3</div>
        </div>
        <div className="rounded-xl bg-white p-6 shadow-sm">
          <div className="text-sm text-gray-500">Tagihan Belum Bayar</div>
          <div className="mt-1 text-3xl font-bold text-[#EF4444]">5</div>
        </div>
        <div className="rounded-xl bg-white p-6 shadow-sm">
          <div className="text-sm text-gray-500">Notifikasi Gagal</div>
          <div className="mt-1 text-3xl font-bold text-gray-400">0</div>
        </div>
      </div>
    </div>
  );
}
