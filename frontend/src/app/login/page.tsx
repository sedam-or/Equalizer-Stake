"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";

export default function Login() {
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
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password }),
      });

      const data = await res.json();

      if (!res.ok) {
        setError(data.detail || "Username atau password salah");
        return;
      }

      // Store token
      localStorage.setItem("token", data.access_token);
      localStorage.setItem("role", data.role);

      // Redirect based on role
      switch (data.role) {
        case "owner":
          router.push("/dashboard/owner");
          break;
        case "admin":
          router.push("/dashboard");
          break;
        case "teacher":
          router.push("/dashboard/attendance");
          break;
        case "guardian":
          router.push("/dashboard");
          break;
        default:
          router.push("/dashboard");
      }
    } catch {
      setError("Gagal terhubung ke server");
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="flex min-h-screen items-center justify-center bg-[#000000] p-4">
      <div className="w-full max-w-md rounded-2xl border border-[#253900] bg-[#0a0a0a] p-8 shadow-lg">
        <div className="mb-8 text-center">
          <h1 className="text-2xl font-bold text-[#08CB00]">BMD Music School</h1>
          <p className="mt-1 text-sm text-[#EEEEEE]">Sistem Penjadwalan Kursus</p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <div className="rounded-lg bg-red-500/10 p-3 text-sm text-red-400">
              {error}
            </div>
          )}

          <div>
            <label className="mb-1 block text-sm font-medium text-[#EEEEEE]">
              Username
            </label>
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              className="w-full rounded-lg border border-[#253900] bg-[#111] px-4 py-3 text-sm text-[#EEEEEE] placeholder-gray-500 focus:border-[#08CB00] focus:outline-none focus:ring-1 focus:ring-[#08CB00]"
              placeholder="Masukkan username"
              required
            />
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium text-[#EEEEEE]">
              Password
            </label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full rounded-lg border border-[#253900] bg-[#111] px-4 py-3 text-sm text-[#EEEEEE] placeholder-gray-500 focus:border-[#08CB00] focus:outline-none focus:ring-1 focus:ring-[#08CB00]"
              placeholder="Masukkan password"
              required
            />
          </div>

          <button
            type="submit"
            disabled={loading}
            className="w-full rounded-lg bg-[#08CB00] py-3 text-sm font-semibold text-black transition-colors hover:bg-[#0aad00] disabled:opacity-50"
          >
            {loading ? "Masuk..." : "Masuk"}
          </button>

          <div className="text-center">
            <a href="#" className="text-sm text-[#08CB00] hover:underline">
              Lupa Password?
            </a>
          </div>
        </form>
      </div>
    </main>
  );
}
