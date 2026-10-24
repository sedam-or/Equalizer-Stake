import Link from "next/link";

export default function NotFound() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center px-6 bg-[#F8FAFC] text-center">
      <span className="mb-4 text-6xl">🔍</span>
      <h1 className="font-heading text-3xl font-bold text-gray-900">404</h1>
      <p className="mt-2 text-gray-500">Halaman tidak ditemukan.</p>
      <Link
        href="/dashboard"
        className="mt-6 rounded-lg bg-[#2563EB] px-5 py-2.5 text-sm font-semibold text-white hover:bg-[#1d4ed8]"
      >
        Kembali ke Dashboard
      </Link>
    </div>
  );
}
