import Link from "next/link";

export default function ResetPasswordSuccess() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-[#F8FAFC] px-6">
      <div className="w-full max-w-md rounded-xl border border-gray-200 bg-white p-8 text-center">
        <span className="text-5xl">✉️</span>
        <h1 className="mt-4 font-heading text-2xl font-bold text-gray-900">Email Terkirim</h1>
        <p className="mt-2 text-sm text-gray-500">
          Kami telah mengirim link reset password ke email Anda. Silakan cek inbox atau spam folder.
        </p>
        <Link href="/login" className="mt-6 inline-block rounded-lg bg-[#2563EB] px-5 py-2.5 text-sm font-semibold text-white hover:bg-[#1d4ed8]">Kembali ke Login</Link>
      </div>
    </div>
  );
}
