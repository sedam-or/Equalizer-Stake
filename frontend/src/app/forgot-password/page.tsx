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
      setSuccess(true);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-[#0F172A] px-6">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="mb-8 text-center">
          <Link href="/" className="font-heading text-2xl font-bold text-white">
            BMD <span className="text-[#D4AF37]">Music</span>
          </Link>
        </div>

        {/* Card */}
        <div className="rounded-xl border border-white/10 bg-white/5 p-8 backdrop-blur-sm">
          {success ? (
            <>
              <div className="mb-4 flex justify-center">
                <div className="flex h-12 w-12 items-center justify-center rounded-full bg-[#D4AF37]/10 text-2xl text-[#D4AF37]">
                  ✓
                </div>
              </div>
              <h1 className="text-center font-heading text-xl font-bold text-white">
                Email Terkirim
              </h1>
              <p className="mt-2 text-center text-sm text-gray-400">
                Kami telah mengirimkan link reset password ke email Anda. Silakan cek inbox atau spam folder.
              </p>
              <Link
                href="/login"
                className="mt-6 block w-full rounded-lg bg-[#D4AF37] py-3 text-center text-sm font-bold text-[#0F172A] transition-all hover:bg-[#D4AF37]/90"
              >
                Kembali ke Login
              </Link>
            </>
          ) : (
            <>
              <h1 className="text-center font-heading text-xl font-bold text-white">
                Lupa Password?
              </h1>
              <p className="mt-2 text-center text-sm text-gray-400">
                Masukkan email Anda dan kami akan mengirimkan link reset password.
              </p>

              <form onSubmit={handleSubmit} className="mt-6 space-y-4">
                <div>
                  <label className="mb-1.5 block text-sm font-medium text-gray-300">Email</label>
                  <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="email@contoh.com"
                    required
                    className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white placeholder-gray-500 transition-colors focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20"
                  />
                </div>

                <button
                  type="submit"
                  disabled={loading}
                  className="w-full rounded-lg bg-[#D4AF37] py-3 text-sm font-bold text-[#0F172A] transition-all hover:bg-[#D4AF37]/90 disabled:opacity-50"
                >
                  {loading ? "Mengirim..." : "Kirim Link Reset"}
                </button>
              </form>

              <p className="mt-4 text-center text-sm text-gray-400">
                Ingat password?{" "}
                <Link href="/login" className="font-semibold text-[#D4AF37] hover:underline">
                  Login
                </Link>
              </p>
            </>
          )}
        </div>

        <p className="mt-6 text-center">
          <Link href="/" className="text-sm text-gray-500 hover:text-white">
            ← Kembali ke Beranda
          </Link>
        </p>
      </div>
    </div>
  );
}
