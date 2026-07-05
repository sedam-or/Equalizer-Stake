"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { Eye, EyeOff } from "lucide-react";
import AuthLayout from "@/components/ui/AuthLayout";
import styles from "./page.module.css";

interface Errors {
  username?: string;
  password?: string;
  general?: string;
}

export default function LoginPage() {
  const router = useRouter();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [errors, setErrors] = useState<Errors>({});
  const [loading, setLoading] = useState(false);

  const validate = (): Errors => {
    const e: Errors = {};
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
      localStorage.setItem("token", data.access_token);
      localStorage.setItem("role", data.role);

      if (data.role === "teacher" || data.role === "guardian") {
        router.push("/dashboard/my-schedule");
      } else {
        router.push("/dashboard");
      }
    } catch {
      setErrors({ general: "Gagal terhubung ke server" });
    } finally {
      setLoading(false);
    }
  };

  return (
    <AuthLayout>
      <div className={styles.card}>
        <h1 className={styles.title}>Masuk</h1>
        <p className={styles.subtitle}>Kelola jadwal musik Anda dalam satu tempat</p>

        {errors.general && (
          <div className={`${styles.serverMessage} ${styles.serverError}`}>
            {errors.general}
          </div>
        )}

        <form onSubmit={handleSubmit} noValidate>
          <div className={styles.field}>
            <label className={styles.label} htmlFor="username">Username</label>
            <input
              className={styles.input}
              id="username"
              name="username"
              type="text"
              placeholder="Masukkan username"
              autoComplete="off"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
            />
            {errors.username && <span className={styles.errorText}>{errors.username}</span>}
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="password">Password</label>
            <div className={styles.passwordWrapper}>
              <input
                className={styles.input}
                id="password"
                name="password"
                type={showPassword ? "text" : "password"}
                placeholder="Masukkan password"
                autoComplete="current-password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
              <button
                type="button"
                className={styles.toggleBtn}
                onClick={() => setShowPassword(!showPassword)}
                tabIndex={-1}
              >
                {showPassword ? <EyeOff size={18} /> : <Eye size={18} />}
              </button>
            </div>
            {errors.password && <span className={styles.errorText}>{errors.password}</span>}
          </div>

          <button type="submit" className={styles.button} disabled={loading}>
            {loading ? "Masuk..." : "Masuk"}
          </button>
        </form>

        <p className={styles.footer}>
          Belum punya akun?{" "}
          <a href="/register-course" className={styles.footerLink}>Daftar</a>
        </p>
        <p className={styles.footer}>
          <a href="/forgot-password" className={styles.footerLink}>Lupa Password?</a>
        </p>
      </div>
    </AuthLayout>
  );
}
