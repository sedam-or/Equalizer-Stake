"use client";

import { useState } from "react";
import { Lock } from "lucide-react";
import AuthLayout from "@/components/ui/AuthLayout";

export default function ForgotPassword() {
  const [email, setEmail] = useState("");
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setSuccess("");
    setLoading(true);

    try {
      const res = await fetch("/api/auth/forgot-password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
      });

      if (!res.ok) {
        setSuccess("Jika email terdaftar, link reset telah dikirim");
        return;
      }

      setSuccess("Link reset telah dikirim ke email Anda");
    } catch {
      setError("Gagal terhubung ke server");
    } finally {
      setLoading(false);
    }
  };

  return (
    <AuthLayout>
      <div className="rounded-2xl border border-[#253900]/60 bg-[#111]/75 p-8 shadow-lg backdrop-blur-xl">
        {/* Header */}
        <div className="mb-6 text-center">
          <h1 className="text-2xl font-bold text-[#08CB00]">Reset Password</h1>
          <p className="mt-1 text-sm text-gray-400">Masukkan email untuk menerima link pemulihan</p>
        </div>

        {!success ? (
          <>
            {/* Icon */}
            <div className="mb-6 flex justify-center">
              <div className="rounded-full bg-[#08CB00]/10 p-4">
                <Lock className="h-8 w-8 text-[#08CB00]" />
              </div>
            </div>

            <form onSubmit={handleSubmit} className="space-y-4">
              {error && (
                <div className="rounded-lg bg-red-500/10 p-3 text-sm text-red-400">
                  {error}
                </div>
              )}

              <div>
                <label className="mb-1 block text-sm font-medium text-[#EEEEEE]">
                  Email
                </label>
                <input
                  type="email"
                  autoComplete="off"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="w-full rounded-lg border border-[#333] bg-[#1a1a1a]/80 px-4 py-3 text-sm text-[#EEEEEE] placeholder-gray-500 focus:border-[#08CB00] focus:outline-none"
                  placeholder="Masukkan email"
                  required
                />
              </div>

              <button
                type="submit"
                disabled={loading}
                className="w-full rounded-lg bg-[#08CB00] py-3 text-sm font-semibold text-black transition-colors hover:bg-[#0aad00] disabled:opacity-50"
              >
                {loading ? (
                  <span className="flex items-center justify-center gap-2">
                    <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                    </svg>
                    Mengirim...
                  </span>
                ) : (
                  "Kirim Link Reset"
                )}
              </button>
            </form>

            <p className="mt-4 text-center text-sm text-gray-400">
              Ingat password?{" "}
              <a href="/login" className="font-medium text-[#08CB00] hover:underline">
                Masuk
              </a>
            </p>
          </>
        ) : (
          /* Success State */
          <div className="space-y-4 text-center">
            <div className="mb-6 flex justify-center">
              <div className="rounded-full bg-[#08CB00]/10 p-4">
                <Lock className="h-8 w-8 text-[#08CB00]" />
              </div>
            </div>

            <div className="rounded-lg bg-[#08CB00]/10 p-4 text-sm text-[#08CB00]">
              {success}
            </div>

            <a
              href="/login"
              className="mt-4 inline-block w-full rounded-lg bg-[#08CB00] py-3 text-sm font-semibold text-black transition-colors hover:bg-[#0aad00]"
            >
              Kembali ke Login
            </a>
          </div>
        )}

        {/* Footer Links */}
        <div className="mt-6 flex justify-center gap-4 border-t border-[#253900]/40 pt-4 text-sm">
          <a href="/login" className="text-[#08CB00] hover:underline">Kembali ke Login</a>
          <span className="text-gray-600">•</span>
          <a href="/register" className="text-[#08CB00] hover:underline">Daftar akun baru</a>
        </div>
      </div>
    </AuthLayout>
  );
}
