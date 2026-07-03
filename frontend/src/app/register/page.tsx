"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { Eye, EyeOff } from "lucide-react";
import AuthLayout from "@/components/ui/AuthLayout";
import styles from "./page.module.css";

interface Errors {
  fullName?: string;
  username?: string;
  email?: string;
  phone?: string;
  address?: string;
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
  const [address, setAddress] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [terms, setTerms] = useState(false);
  const [errors, setErrors] = useState<Errors>({});
  const [serverMessage, setServerMessage] = useState("");
  const [serverSuccess, setServerSuccess] = useState(false);
  const [loading, setLoading] = useState(false);
  const [showTerms, setShowTerms] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);

  const validate = (): Errors => {
    const e: Errors = {};
    if (!fullName.trim()) e.fullName = "Nama lengkap wajib diisi";
    if (username.length < 6) e.username = "Username minimal 6 karakter";
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) e.email = "Email tidak valid";
    if (phone.length < 10) e.phone = "Nomor WhatsApp minimal 10 karakter";
    if (!address.trim()) e.address = "Alamat wajib diisi";
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
          address: address.trim(),
        }),
      });

      const data = await res.json();

      if (!res.ok) {
        const msg =
          typeof data.detail === "string"
            ? data.detail
            : typeof data.message === "string"
              ? data.message
              : "Registrasi gagal";
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

  return (
    <>
      <AuthLayout>
        <div className={styles.card}>
          <h1 className={styles.title}>Buat Akun Baru</h1>
          <p className={styles.subtitle}>Bergabung bersama BMD Music School — mulai perjalanan musik Anda</p>

          {serverMessage && (
            <div className={`${styles.serverMessage} ${serverSuccess ? styles.serverSuccess : styles.serverError}`}>
              {serverMessage}
            </div>
          )}

          <form onSubmit={handleSubmit} noValidate>
            {/* Row 1: Nama Lengkap + Username */}
            <div className={styles.gridRow}>
              <div className={styles.field}>
                <label className={styles.label} htmlFor="fullName">Nama Lengkap</label>
                <input className={styles.input} id="fullName" name="fullName" type="text" placeholder="Masukkan nama lengkap" autoComplete="off" value={fullName} onChange={(e) => setFullName(e.target.value)} />
                {errors.fullName && <span className={styles.errorText}>{errors.fullName}</span>}
              </div>
              <div className={styles.field}>
                <label className={styles.label} htmlFor="username">Username</label>
                <input className={styles.input} id="username" name="username" type="text" placeholder="Minimal 6 karakter" autoComplete="off" value={username} onChange={(e) => setUsername(e.target.value)} />
                {errors.username && <span className={styles.errorText}>{errors.username}</span>}
              </div>
            </div>

            {/* Row 2: Email + WhatsApp */}
            <div className={styles.gridRow}>
              <div className={styles.field}>
                <label className={styles.label} htmlFor="email">Email</label>
                <input className={styles.input} id="email" name="email" type="email" placeholder="contoh@email.com" autoComplete="off" value={email} onChange={(e) => setEmail(e.target.value)} />
                {errors.email && <span className={styles.errorText}>{errors.email}</span>}
              </div>
              <div className={styles.field}>
                <label className={styles.label} htmlFor="phone">WhatsApp</label>
                <input className={styles.input} id="phone" name="phone" type="tel" placeholder="08xxxxxxxxxx" autoComplete="off" value={phone} onChange={(e) => setPhone(e.target.value)} />
                {errors.phone && <span className={styles.errorText}>{errors.phone}</span>}
              </div>
            </div>

            {/* Row 3: Alamat (full width) */}
            <div className={styles.field}>
              <label className={styles.label} htmlFor="address">Alamat</label>
              <input className={styles.input} id="address" name="address" type="text" placeholder="Alamat lengkap" autoComplete="off" value={address} onChange={(e) => setAddress(e.target.value)} />
              {errors.address && <span className={styles.errorText}>{errors.address}</span>}
            </div>

            {/* Row 4: Password + Konfirmasi */}
            <div className={styles.gridRow}>
              <div className={styles.field}>
                <label className={styles.label} htmlFor="password">Password</label>
                <div className={styles.passwordWrapper}>
                  <input className={styles.input} id="password" name="password" type={showPassword ? "text" : "password"} placeholder="Minimal 8 karakter" autoComplete="new-password" value={password} onChange={(e) => setPassword(e.target.value)} />
                  <button type="button" className={styles.toggleBtn} onClick={() => setShowPassword(!showPassword)} tabIndex={-1}>
                    {showPassword ? <EyeOff size={18} /> : <Eye size={18} />}
                  </button>
                </div>
                {errors.password && <span className={styles.errorText}>{errors.password}</span>}
              </div>
              <div className={styles.field}>
                <label className={styles.label} htmlFor="confirmPassword">Konfirmasi Password</label>
                <div className={styles.passwordWrapper}>
                  <input className={styles.input} id="confirmPassword" name="confirmPassword" type={showConfirmPassword ? "text" : "password"} placeholder="Ulangi password" autoComplete="new-password" value={confirmPassword} onChange={(e) => setConfirmPassword(e.target.value)} />
                  <button type="button" className={styles.toggleBtn} onClick={() => setShowConfirmPassword(!showConfirmPassword)} tabIndex={-1}>
                    {showConfirmPassword ? <EyeOff size={18} /> : <Eye size={18} />}
                  </button>
                </div>
                {errors.confirmPassword && <span className={styles.errorText}>{errors.confirmPassword}</span>}
              </div>
            </div>

            {/* Terms */}
            <div className={styles.termsRow}>
              <input type="checkbox" id="terms" checked={terms} onChange={(e) => setTerms(e.target.checked)} />
              <label htmlFor="terms">
                Saya menyetujui{" "}
                <a href="#" onClick={(e) => { e.preventDefault(); setShowTerms(true); }}>
                  syarat & ketentuan
                </a>
              </label>
            </div>
            {errors.terms && <span className={styles.errorText}>{errors.terms}</span>}

            <button type="submit" className={styles.button} disabled={loading}>
              {loading ? "Memproses..." : "Daftar"}
            </button>
          </form>

          <p className={styles.footer}>
            Sudah punya akun?{" "}
            <a href="/login" className={styles.footerLink}>Masuk</a>
          </p>
        </div>
      </AuthLayout>

      {/* Terms Modal */}
      {showTerms && (
        <div className={styles.modalOverlay} onClick={() => setShowTerms(false)}>
          <div className={styles.modalContent} onClick={(e) => e.stopPropagation()}>
            <h2>Syarat & Ketentuan</h2>
            <p>Dengan mendaftar, Anda menyetujui:</p>
            <ul>
              <li>Data pribadi digunakan untuk keperluan kursus musik</li>
              <li>Password harus minimal 8 karakter</li>
              <li>Satu akun per orang tua/wali</li>
              <li>Informasi kontak WhatsApp digunakan untuk komunikasi resmi</li>
            </ul>
            <button className={styles.button} style={{ marginTop: "1rem" }} onClick={() => setShowTerms(false)}>Tutup</button>
          </div>
        </div>
      )}
    </>
  );
}
