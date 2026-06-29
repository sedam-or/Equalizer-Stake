"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import styles from "./page.module.css";

interface Errors {
  fullName?: string;
  username?: string;
  email?: string;
  phone?: string;
  password?: string;
  confirmPassword?: string;
  terms?: string;
}

export default function Register() {
  const router = useRouter();
  const [fullName, setFullName] = useState("");
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [phone, setPhone] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [terms, setTerms] = useState(false);
  const [errors, setErrors] = useState<Errors>({});
  const [serverMessage, setServerMessage] = useState("");
  const [serverSuccess, setServerSuccess] = useState(false);
  const [loading, setLoading] = useState(false);

  const validate = (): Errors => {
    const e: Errors = {};
    if (!fullName.trim()) e.fullName = "Nama lengkap wajib diisi";
    if (username.length < 6) e.username = "Username minimal 6 karakter";
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) e.email = "Email tidak valid";
    if (phone.length < 10) e.phone = "Nomor WhatsApp minimal 10 karakter";
    if (password.length < 8) e.password = "Password minimal 8 karakter";
    if (password !== confirmPassword) e.confirmPassword = "Password tidak cocok";
    if (!terms) e.terms = "Anda harus menyetujui syarat & ketentuan";
    return e;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setServerMessage("");
    setServerSuccess(false);

    const validationErrors = validate();
    setErrors(validationErrors);
    if (Object.keys(validationErrors).length > 0) return;

    setLoading(true);
    try {
      const res = await fetch("/api/auth/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          username,
          email,
          password,
          full_name: fullName,
          phone,
        }),
      });

      const data = await res.json();

      if (!res.ok) {
        const msg =
          typeof data.detail === "string"
            ? data.detail
            : typeof data.message === "string"
              ? data.message
              : "Registration failed";
        setServerMessage(msg);
        setServerSuccess(false);
        return;
      }

      setServerMessage("Registrasi berhasil! Mengarahkan ke halaman login...");
      setServerSuccess(true);
      setTimeout(() => router.push("/login"), 1500);
    } catch {
      setServerMessage("Gagal terhubung ke server");
      setServerSuccess(false);
    } finally {
      setLoading(false);
    }
  };

  const fields: {
    key: keyof Errors;
    label: string;
    name: string;
    type: string;
    placeholder: string;
    value: string;
    setter: (v: string) => void;
  }[] = [
    { key: "fullName", label: "Nama Lengkap", name: "fullName", type: "text", placeholder: "Masukkan nama lengkap", value: fullName, setter: setFullName },
    { key: "username", label: "Username", name: "username", type: "text", placeholder: "Masukkan username", value: username, setter: setUsername },
    { key: "email", label: "Email", name: "email", type: "email", placeholder: "Masukkan email", value: email, setter: setEmail },
    { key: "phone", label: "WhatsApp", name: "phone", type: "tel", placeholder: "Masukkan nomor WhatsApp", value: phone, setter: setPhone },
    { key: "password", label: "Password", name: "password", type: "password", placeholder: "Masukkan password", value: password, setter: setPassword },
    { key: "confirmPassword", label: "Konfirmasi Password", name: "confirmPassword", type: "password", placeholder: "Ulangi password", value: confirmPassword, setter: setConfirmPassword },
  ];

  return (
    <main className={styles.container}>
      <div className={styles.card}>
        <h1 className={styles.title}>Daftar</h1>

        {serverMessage && (
          <div className={`${styles.serverMessage} ${serverSuccess ? styles.serverSuccess : styles.serverError}`}>
            {serverMessage}
          </div>
        )}

        <form onSubmit={handleSubmit} noValidate>
          {fields.map((f) => (
            <div key={f.key} className={styles.field}>
              <label className={styles.label} htmlFor={f.name}>
                {f.label}
              </label>
              <input
                id={f.name}
                className={`${styles.input} ${errors[f.key] ? styles.inputError : ""}`}
                type={f.type}
                placeholder={f.placeholder}
                value={f.value}
                onChange={(e) => f.setter(e.target.value)}
              />
              {errors[f.key] && <p className={styles.error}>{errors[f.key]}</p>}
            </div>
          ))}

          <div className={styles.checkboxRow}>
            <input
              id="terms"
              type="checkbox"
              className={styles.checkbox}
              checked={terms}
              onChange={(e) => setTerms(e.target.checked)}
            />
            <label htmlFor="terms" className={styles.checkboxLabel}>
              Saya menyetujui{" "}
              <a href="/terms" target="_blank" rel="noopener noreferrer">
                Syarat &amp; Ketentuan
              </a>
            </label>
          </div>
          {errors.terms && <p className={styles.error}>{errors.terms}</p>}

          <button type="submit" className={styles.button} disabled={loading}>
            {loading ? "Memproses..." : "REGISTER"}
          </button>
        </form>

        <p className={styles.footer}>
          Sudah punya akun?{" "}
          <a href="/login">Masuk</a>
        </p>
      </div>
    </main>
  );
}
