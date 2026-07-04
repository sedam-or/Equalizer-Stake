import Link from "next/link";

export default function ResetPasswordError() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-[#F8FAFC] px-6">
      <div className="w-full max-w-md rounded-xl border border-red-200 bg-white p-8 text-center">
        <span className="text-5xl">❌</span>
        <h1 className="mt-4 font-heading text-2xl font-bold text-gray-900">Link Tidak Valid</h1>
        <p className="mt-2 text-sm text-gray-500">
          Link reset password tidak valid atau sudah expired. Silakan minta link baru.
        </p>
        <Link href="/forgot-password" className="mt-6 inline-block rounded-lg bg-[#2563EB] px-5 py-2.5 text-sm font-semibold text-white hover:bg-[#1d4ed8]">Minta Link Baru</Link>
      </div>
    </div>
  );
}
