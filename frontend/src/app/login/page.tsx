"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";

export default function LoginPage() {
  const router = useRouter();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);

    try {
      const res = await fetch("/api/v1/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password }),
      });

      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.detail || "Login gagal");
      }

      const data = await res.json();
      localStorage.setItem("access_token", data.access_token);
      if (data.refresh_token) localStorage.setItem("refresh_token", data.refresh_token);
      if (data.role) localStorage.setItem("role", data.role);

      // Redirect based on role
      const role = data.role;
      if (role === "teacher") router.push("/dashboard/my-schedule");
      else if (role === "guardian") router.push("/dashboard/my-students");
      else router.push("/dashboard");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Terjadi kesalahan");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen bg-[#0F172A]">
      {/* Left: Visual */}
      <div className="hidden w-1/2 items-center justify-center lg:flex">
        <div className="max-w-md text-center">
          <div className="mb-6 flex justify-center">
            <div className="h-24 w-px bg-gradient-to-b from-transparent via-[#D4AF37]/30 to-transparent" />
          </div>
          <h1 className="font-heading text-4xl font-bold tracking-tight text-white">
            BMD <span className="text-[#D4AF37]">Music</span>
          </h1>
          <p className="mt-4 text-lg text-gray-400">
            Kelas Privat Musik Premium di Yogyakarta
          </p>
          <div className="mt-8 flex justify-center gap-8">
            <div className="text-center">
              <div className="font-heading text-2xl font-bold text-white">150+</div>
              <div className="text-xs text-gray-500">Keluarga</div>
            </div>
            <div className="text-center">
              <div className="font-heading text-2xl font-bold text-white">10+</div>
              <div className="text-xs text-gray-500">Tahun</div>
            </div>
            <div className="text-center">
              <div className="font-heading text-2xl font-bold text-white">4.9★</div>
              <div className="text-xs text-gray-500">Google</div>
            </div>
          </div>
        </div>
      </div>

      {/* Right: Form */}
      <div className="flex w-full items-center justify-center px-6 lg:w-1/2">
        <div className="w-full max-w-md">
          {/* Mobile logo */}
          <div className="mb-8 text-center lg:hidden">
            <Link href="/" className="font-heading text-2xl font-bold text-white">
              BMD <span className="text-[#D4AF37]">Music</span>
            </Link>
          </div>

          {/* Card */}
          <div className="rounded-xl border border-white/10 bg-white/5 p-8 backdrop-blur-sm">
            <h1 className="font-heading text-2xl font-bold text-white">Selamat Datang Kembali</h1>
            <p className="mt-2 text-sm text-gray-400">Masuk ke akun Anda untuk melanjutkan</p>

            {error && (
              <div className="mt-4 rounded-lg border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400">
                {error}
              </div>
            )}

            <form onSubmit={handleSubmit} className="mt-6 space-y-4">
              <div>
                <label className="mb-1.5 block text-sm font-medium text-gray-300">Username</label>
                <input
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  placeholder="Masukkan username"
                  required
                  className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white placeholder-gray-500 transition-colors focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20"
                />
              </div>

              <div>
                <label className="mb-1.5 block text-sm font-medium text-gray-300">Password</label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Masukkan password"
                  required
                  className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white placeholder-gray-500 transition-colors focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20"
                />
              </div>

              <div className="flex items-center justify-between">
                <label className="flex items-center gap-2 text-sm text-gray-400">
                  <input type="checkbox" className="rounded border-white/20 bg-white/5" />
                  Ingat saya
                </label>
                <Link href="/forgot-password" className="text-sm text-[#D4AF37] hover:underline">
                  Lupa password?
                </Link>
              </div>

              <button
                type="submit"
                disabled={loading}
                className="w-full rounded-lg bg-[#D4AF37] py-3 text-sm font-bold text-[#0F172A] transition-all hover:bg-[#D4AF37]/90 disabled:opacity-50"
              >
                {loading ? "Masuk..." : "Masuk"}
              </button>
            </form>

            <p className="mt-6 text-center text-sm text-gray-400">
              Belum punya akun?{" "}
              <Link href="/register-course" className="font-semibold text-[#D4AF37] hover:underline">
                Daftar Kursus
              </Link>
            </p>
          </div>

          {/* Back */}
          <p className="mt-6 text-center">
            <Link href="/" className="text-sm text-gray-500 hover:text-white">
              ← Kembali ke Beranda
            </Link>
          </p>
        </div>
      </div>
    </div>
  );
}
