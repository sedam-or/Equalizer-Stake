"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";

export default function LoginPage() {
  const router = useRouter();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [errors, setErrors] = useState<{ username?: string; password?: string; general?: string }>({});
  const [loading, setLoading] = useState(false);

  const validate = () => {
    const e: typeof errors = {};
    if (!username.trim()) e.username = "Username wajib diisi";
    if (!password) e.password = "Password wajib diisi";
    return e;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const v = validate();
    setErrors(v);
    if (Object.keys(v).length > 0) return;

    setLoading(true);
    try {
      const res = await fetch("/api/v1/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password }),
      });
      const data = await res.json();
      if (!res.ok) {
        setErrors({ general: data.detail || "Login gagal" });
        return;
      }
      localStorage.setItem("access_token", data.access_token);
      localStorage.setItem("refresh_token", data.refresh_token);
      router.push("/dashboard");
    } catch {
      setErrors({ general: "Terjadi kesalahan. Coba lagi." });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-[#F8FAFC] px-4">
      <div className="w-full max-w-md rounded-xl border border-gray-200 bg-white p-8 shadow-sm">
        {/* Header */}
        <div className="text-center">
          <a href="/" className="font-heading text-2xl font-bold text-gray-900">BMD</a>
          <h1 className="mt-4 font-heading text-xl font-bold text-gray-900">Masuk ke Akun Anda</h1>
          <p className="mt-1 text-sm text-gray-500">Masukkan username dan password untuk melanjutkan</p>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="mt-6 space-y-4">
          {/* General error */}
          {errors.general && (
            <div className="rounded-lg bg-red-50 p-3 text-sm text-red-600">{errors.general}</div>
          )}

          {/* Username */}
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700">Username</label>
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              className={`w-full rounded-lg border px-3 py-2.5 text-sm focus:border-[#2563EB] focus:outline-none focus:ring-2 focus:ring-[#2563EB]/20 ${errors.username ? "border-red-300" : "border-gray-300"}`}
              placeholder="Masukkan username"
            />
            {errors.username && <p className="mt-1 text-xs text-red-500">{errors.username}</p>}
          </div>

          {/* Password */}
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700">Password</label>
            <div className="relative">
              <input
                type={showPassword ? "text" : "password"}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className={`w-full rounded-lg border px-3 py-2.5 pr-10 text-sm focus:border-[#2563EB] focus:outline-none focus:ring-2 focus:ring-[#2563EB]/20 ${errors.password ? "border-red-300" : "border-gray-300"}`}
                placeholder="Masukkan password"
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
              >
                {showPassword ? "🙈" : "👁️"}
              </button>
            </div>
            {errors.password && <p className="mt-1 text-xs text-red-500">{errors.password}</p>}
          </div>

          {/* Submit */}
          <button
            type="submit"
            disabled={loading}
            className="w-full rounded-lg bg-[#2563EB] py-2.5 text-sm font-semibold text-white hover:bg-[#1D4ED8] disabled:opacity-50"
          >
            {loading ? "Masuk..." : "Masuk"}
          </button>
        </form>

        {/* Footer links */}
        <div className="mt-6 flex items-center justify-between text-sm">
          <a href="/register-course" className="text-[#2563EB] hover:underline">Daftar Akun</a>
          <a href="/forgot-password" className="text-gray-500 hover:text-gray-700">Lupa Password?</a>
        </div>
      </div>
    </div>
  );
}
