"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import styles from "./page.module.css";

export default function LoginPage() {
  const router = useRouter();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [errors, setErrors] = useState<{ username?: string; password?: string; general?: string }>({});
  const [loading, setLoading] = useState(false);

  const validate = () => {
    const e: typeof errors = {};
    if (!username.trim()) e.username = "Username wajib diisi";
    if (!password) e.password = "Password wajib diisi";
    setErrors(e);
    return Object.keys(e).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});
    if (!validate()) return;

    setLoading(true);
    try {
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username: username.trim(), password }),
      });

      const data = await res.json();

      if (!res.ok) {
        setErrors({ general: data.detail || "Username atau password tidak valid" });
        return;
      }

      localStorage.setItem("access_token", data.access_token);
      localStorage.setItem("role", data.role);
      router.push("/dashboard");
    } catch {
      setErrors({ general: "Gagal terhubung ke server" });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.wrapper}>
        {/* Header */}
        <div className={styles.header}>
          <h1 className={styles.title}>Daftar Akun</h1>
          <p className={styles.subtitle}>Masuk ke sistem BMD Musik</p>
        </div>

        {/* Form Card */}
        <form onSubmit={handleSubmit} noValidate className={styles.card}>
          {errors.general && (
            <div className={styles.errorBox}>{errors.general}</div>
          )}

          {/* Username */}
          <div className={styles.field}>
            <label className={styles.label}>Username</label>
            <input
              type="text"
              autoComplete="off"
              value={username}
              onChange={(e) => { setUsername(e.target.value); setErrors(p => ({ ...p, username: undefined, general: undefined })); }}
              className={styles.input}
              placeholder="Masukkan username"
            />
            {errors.username && <span className={styles.errorBox}>{errors.username}</span>}
          </div>

          {/* Password */}
          <div className={styles.field}>
            <label className={styles.label}>Password</label>
            <input
              type="password"
              autoComplete="off"
              value={password}
              onChange={(e) => { setPassword(e.target.value); setErrors(p => ({ ...p, password: undefined, general: undefined })); }}
              className={styles.input}
              placeholder="Masukkan password"
            />
            {errors.password && <span className={styles.errorBox}>{errors.password}</span>}
          </div>

          {/* Submit */}
          <button type="submit" disabled={loading} className={styles.submitBtn}>
            {loading ? "Masuk..." : "Masuk"}
          </button>
        </form>

        {/* Links */}
        <p className={styles.footer}>
          Belum punya akun?{" "}
          <a href="/register" className={styles.footerLink}>Daftar</a>
        </p>
        <p className={styles.forgotLink}>
          <a href="/forgot-password">Lupa Password?</a>
        </p>
      </div>
    </div>
  );
}
