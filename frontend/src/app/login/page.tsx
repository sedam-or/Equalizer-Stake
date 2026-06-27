export default function Login() {
  return (
    <main className="flex min-h-screen items-center justify-center bg-gray-50 p-4">
      <div className="w-full max-w-md rounded-2xl bg-white p-8 shadow-lg">
        <div className="mb-8 text-center">
          <h1 className="text-2xl font-bold text-[#1E40AF]">BMD Music School</h1>
          <p className="mt-1 text-sm text-gray-500">Sistem Penjadwalan Kursus</p>
        </div>
        <form className="space-y-4">
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700">Username</label>
            <input
              type="text"
              className="w-full rounded-lg border border-gray-300 px-4 py-3 text-sm focus:border-[#1E40AF] focus:outline-none focus:ring-1 focus:ring-[#1E40AF]"
              placeholder="Masukkan username"
            />
          </div>
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700">Password</label>
            <input
              type="password"
              className="w-full rounded-lg border border-gray-300 px-4 py-3 text-sm focus:border-[#1E40AF] focus:outline-none focus:ring-1 focus:ring-[#1E40AF]"
              placeholder="Masukkan password"
            />
          </div>
          <button
            type="submit"
            className="w-full rounded-lg bg-[#1E40AF] py-3 text-sm font-semibold text-white hover:bg-blue-800"
          >
            Masuk
          </button>
          <div className="text-center">
            <a href="#" className="text-sm text-[#3B82F6] hover:underline">
              Lupa Password?
            </a>
          </div>
        </form>
      </div>
    </main>
  );
}
