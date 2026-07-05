"use client";

import { useState } from "react";
import Link from "next/link";

export default function ForgotPasswordPage() {
  const [email, setEmail] = useState("");
  const [success, setSuccess] = useState(false);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    try {
      await fetch("/api/v1/auth/forgot-password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
      });
      setSuccess(true);
    } catch {
      setSuccess(true); // Show success even on error (security)
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-[#F8FAFC] px-4">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="mb-8 text-center">
          <Link href="/" className="font-heading text-2xl font-bold tracking-tight text-[#0F172A]">
            BMD <span className="text-[#D4AF37]">Music</span>
          </Link>
        </div>

        {/* Card */}
        <div className="rounded-xl border border-gray-200 bg-white p-8 shadow-sm">
          {success ? (
            <>
              <div className="mb-4 flex justify-center">
                <div className="flex h-12 w-12 items-center justify-center rounded-full bg-emerald-100 text-2xl text-emerald-600">
                  ✓
                </div>
              </div>
              <h1 className="text-center font-heading text-xl font-bold text-[#0F172A]">
                Email Terkirim
              </h1>
              <p className="mt-2 text-center text-sm text-gray-500">
                Kami telah mengirimkan link reset password ke email Anda. Silakan cek inbox atau spam folder.
              </p>
              <Link
                href="/login"
                className="mt-6 block w-full rounded-lg bg-[#2563EB] py-2.5 text-center text-sm font-semibold text-white hover:bg-[#1D4ED8]"
              >
                Kembali ke Login
              </Link>
            </>
          ) : (
            <>
              <h1 className="text-center font-heading text-xl font-bold text-[#0F172A]">
                Lupa Password?
              </h1>
              <p className="mt-2 text-center text-sm text-gray-500">
                Masukkan email Anda dan kami akan mengirimkan link reset password.
              </p>

              <form onSubmit={handleSubmit} className="mt-6 space-y-4">
                <div>
                  <label className="mb-1 block text-sm font-medium text-gray-700">Email</label>
                  <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="email@contoh.com"
                    required
                    className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm focus:border-[#2563EB] focus:outline-none focus:ring-2 focus:ring-[#2563EB]/20"
                  />
                </div>

                <button
                  type="submit"
                  disabled={loading}
                  className="w-full rounded-lg bg-[#2563EB] py-2.5 text-sm font-semibold text-white hover:bg-[#1D4ED8] disabled:opacity-50"
                >
                  {loading ? "Mengirim..." : "Kirim Link Reset"}
                </button>
              </form>

              <p className="mt-4 text-center text-sm text-gray-500">
                Ingat password?{" "}
                <Link href="/login" className="font-semibold text-[#2563EB] hover:underline">
                  Login
                </Link>
              </p>
            </>
          )}
        </div>

        {/* Back to home */}
        <p className="mt-6 text-center text-sm text-gray-500">
          <Link href="/" className="text-gray-400 hover:text-gray-600">
            ← Kembali ke Beranda
          </Link>
        </p>
      </div>
    </div>
  );
}
