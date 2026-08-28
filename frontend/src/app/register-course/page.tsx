"use client";

import { useState } from "react";
import Link from "next/link";

const INSTRUMENTS = ["piano", "gitar", "biola", "drum", "vokal", "keyboard", "flute", "saxophone", "bass", "paket_liburan"];
const INSTRUMENT_LABELS: Record<string, string> = {
  piano: "Piano", gitar: "Gitar", biola: "Biola", drum: "Drum",
  vokal: "Vokal", keyboard: "Keyboard", flute: "Flute", saxophone: "Saxophone",
  bass: "Bass", paket_liburan: "Paket Liburan",
};
const LEVELS = ["pemula", "dasar", "menengah", "lanjut"];

const steps = [
  { id: 1, label: "Data Diri" },
  { id: 2, label: "Buat Akun" },
  { id: 3, label: "Tambah Anak" },
  { id: 4, label: "Pilih Jadwal" },
  { id: 5, label: "Pembayaran" },
];

const CLASS_TYPES = [
  { value: "private", label: "Private (1 siswa)" },
  { value: "intensif_2", label: "Intensif 2 (2 siswa)" },
  { value: "intensif_3", label: "Intensif 3 (3 siswa)" },
  { value: "semi_intensif", label: "Semi Intensif (4 siswa)" },
  { value: "tunas", label: "Tunas (anak usia dini)" },
];

interface Child {
  full_name: string;
  instrument: string;
  level: string;
  class_type: string;
}

export default function RegisterCoursePage() {
  const [step, setStep] = useState(1);
  const [form, setForm] = useState({
    full_name: "", phone_whatsapp: "", email: "", address: "",
    username: "", password: "",
  });
  const [children, setChildren] = useState<Child[]>([{ full_name: "", instrument: "piano", level: "pemula", class_type: "private" }]);
  const [paymentProof, setPaymentProof] = useState<File | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState("");

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => {
    setForm({ ...form, [e.target.name]: e.target.value });
  };

  const handleChildChange = (index: number, field: keyof Child, value: string) => {
    const updated = [...children];
    updated[index] = { ...updated[index], [field]: value };
    setChildren(updated);
  };

  const addChild = () => setChildren([...children, { full_name: "", instrument: "piano", level: "pemula", class_type: "private" }]);
  const removeChild = (index: number) => setChildren(children.filter((_, i) => i !== index));

  const handleSubmit = async () => {
    setSubmitting(true);
    setError("");
    try {
      const formData = new FormData();
      Object.entries(form).forEach(([k, v]) => formData.append(k, v));
      formData.append("children", JSON.stringify(children));
      if (paymentProof) formData.append("payment_proof", paymentProof);

      const res = await fetch("/api/v1/registrations", { method: "POST", body: formData });
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        throw new Error(data.detail || "Gagal mengirim pendaftaran");
      }
      setSuccess(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Terjadi kesalahan");
    } finally {
      setSubmitting(false);
    }
  };

  if (success) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-[#0F172A] px-6">
        <div className="w-full max-w-md text-center">
          <div className="mb-6 flex justify-center">
            <div className="flex h-20 w-20 items-center justify-center rounded-full bg-[#D4AF37]/10 text-4xl text-[#D4AF37]">
              ✓
            </div>
          </div>
          <h1 className="font-heading text-2xl font-bold text-white">Pendaftaran Berhasil!</h1>
          <p className="mt-4 text-gray-400">
            Pendaftaran Anda sedang dalam antrian verifikasi. Admin akan memverifikasi pembayaran dalam 1×24 jam.
          </p>
          <div className="mt-8 flex flex-col gap-3">
            <Link href="/login" className="rounded-lg bg-[#D4AF37] py-3 text-sm font-bold text-[#0F172A] transition-all hover:bg-[#D4AF37]/90">
              Login
            </Link>
            <Link href="/" className="text-sm text-gray-500 hover:text-white">
              ← Kembali ke Beranda
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[#0F172A]">
      {/* Header */}
      <div className="border-b border-white/10 px-6 py-4">
        <div className="mx-auto flex max-w-3xl items-center justify-between">
          <Link href="/" className="font-heading text-xl font-bold text-white">
            Equalizer-Stake <span className="text-[#D4AF37]">Music</span>
          </Link>
          <Link href="/login" className="text-sm text-gray-400 hover:text-white">
            Sudah punya akun? <span className="text-[#D4AF37]">Login</span>
          </Link>
        </div>
      </div>

      <div className="mx-auto max-w-3xl px-6 py-8">
        {/* Progress */}
        <div className="mb-8 flex items-center justify-between">
          {steps.map((s, i) => (
            <div key={s.id} className="flex items-center">
              <div className={`flex h-8 w-8 items-center justify-center rounded-full text-sm font-bold transition-all ${
                step >= s.id
                  ? "bg-[#D4AF37] text-[#0F172A]"
                  : "border border-white/20 text-gray-500"
              }`}>
                {step > s.id ? "✓" : s.id}
              </div>
              <span className={`ml-2 hidden text-sm sm:block ${step >= s.id ? "text-white" : "text-gray-500"}`}>
                {s.label}
              </span>
              {i < steps.length - 1 && (
                <div className={`mx-3 hidden h-px w-8 sm:block ${step > s.id ? "bg-[#D4AF37]" : "bg-white/10"}`} />
              )}
            </div>
          ))}
        </div>

        {/* Error */}
        {error && (
          <div className="mb-6 rounded-lg border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400">
            {error}
          </div>
        )}

        {/* Step Content */}
        <div className="rounded-xl border border-white/10 bg-white/5 p-6 backdrop-blur-sm">
          {/* Step 1: Data Diri */}
          {step === 1 && (
            <div className="space-y-4">
              <h2 className="font-heading text-lg font-bold text-white">Data Diri</h2>
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <div>
                  <label className="mb-1.5 block text-sm font-medium text-gray-300">Nama Lengkap</label>
                  <input name="full_name" value={form.full_name} onChange={handleChange} placeholder="Nama lengkap" className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white placeholder-gray-500 focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20" />
                </div>
                <div>
                  <label className="mb-1.5 block text-sm font-medium text-gray-300">No. WhatsApp</label>
                  <input name="phone_whatsapp" value={form.phone_whatsapp} onChange={handleChange} placeholder="0812-3456-7890" className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white placeholder-gray-500 focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20" />
                </div>
              </div>
              <div>
                <label className="mb-1.5 block text-sm font-medium text-gray-300">Email</label>
                <input name="email" type="email" value={form.email} onChange={handleChange} placeholder="email@contoh.com" className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white placeholder-gray-500 focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20" />
              </div>
              <div>
                <label className="mb-1.5 block text-sm font-medium text-gray-300">Alamat</label>
                <textarea name="address" value={form.address} onChange={handleChange} placeholder="Alamat lengkap" rows={2} className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white placeholder-gray-500 focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20" />
              </div>
            </div>
          )}

          {/* Step 2: Buat Akun */}
          {step === 2 && (
            <div className="space-y-4">
              <h2 className="font-heading text-lg font-bold text-white">Buat Akun</h2>
              <div>
                <label className="mb-1.5 block text-sm font-medium text-gray-300">Username</label>
                <input name="username" value={form.username} onChange={handleChange} placeholder="Pilih username" className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white placeholder-gray-500 focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20" />
              </div>
              <div>
                <label className="mb-1.5 block text-sm font-medium text-gray-300">Password</label>
                <input name="password" type="password" value={form.password} onChange={handleChange} placeholder="Minimal 8 karakter" className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white placeholder-gray-500 focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20" />
              </div>
            </div>
          )}

          {/* Step 3: Tambah Anak */}
          {step === 3 && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h2 className="font-heading text-lg font-bold text-white">Tambah Anak</h2>
                <button onClick={addChild} className="text-sm text-[#D4AF37] hover:underline">+ Tambah Anak</button>
              </div>
              {children.map((child, i) => (
                <div key={i} className="rounded-lg border border-white/10 bg-white/5 p-4">
                  <div className="mb-3 flex items-center justify-between">
                    <span className="text-sm font-medium text-gray-300">Anak {i + 1}</span>
                    {children.length > 1 && (
                      <button onClick={() => removeChild(i)} className="text-xs text-red-400 hover:underline">Hapus</button>
                    )}
                  </div>
                  <div className="space-y-3">
                    <input value={child.full_name} onChange={(e) => handleChildChange(i, "full_name", e.target.value)} placeholder="Nama anak" className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white placeholder-gray-500 focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20" />
                    <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
                      <select value={child.instrument} onChange={(e) => handleChildChange(i, "instrument", e.target.value)} className="rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20">
                        {INSTRUMENTS.map((x) => <option key={x} value={x}>{INSTRUMENT_LABELS[x]}</option>)}
                      </select>
                      <select value={child.level} onChange={(e) => handleChildChange(i, "level", e.target.value)} className="rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20">
                        {LEVELS.map((x) => <option key={x} value={x}>{x.charAt(0).toUpperCase() + x.slice(1)}</option>)}
                      </select>
                      </div>
                      <div>
                        <label className="mb-1 block text-sm font-medium text-gray-300">Tipe Kelas</label>
                        <select value={child.class_type} onChange={(e) => handleChildChange(i, "class_type", e.target.value)} className="rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white focus:border-[#D4AF37]/50 focus:outline-none focus:ring-2 focus:ring-[#D4AF37]/20">
                          {CLASS_TYPES.map((ct) => <option key={ct.value} value={ct.value}>{ct.label}</option>)}
                        </select>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Step 4: Pilih Jadwal */}
          {step === 4 && (
            <div className="space-y-4">
              <h2 className="font-heading text-lg font-bold text-white">Pilih Jadwal</h2>
              <p className="text-sm text-gray-400">
                Jadwal akan ditentukan oleh admin setelah pendaftaran diverifikasi.
              </p>
              <div className="rounded-lg border border-white/10 bg-white/5 p-4">
                <p className="text-sm text-gray-300">💡 Tips:</p>
                <ul className="mt-2 space-y-1 text-sm text-gray-400">
                  <li>• Jam operasional: Senin-Sabtu, 08:00-20:00</li>
                  <li>• Durasi sesi: 30 atau 45 menit</li>
                  <li>• Konsultasikan jadwal dengan admin</li>
                </ul>
              </div>
            </div>
          )}

          {/* Step 5: Pembayaran */}
          {step === 5 && (
            <div className="space-y-4">
              <h2 className="font-heading text-lg font-bold text-white">Pembayaran</h2>
              <div className="rounded-lg border border-white/10 bg-white/5 p-4">
                <p className="text-sm text-gray-300">Transfer ke:</p>
                <p className="mt-1 font-heading text-lg font-bold text-[#D4AF37]">Bank 1234567890</p>
                <p className="text-xs text-gray-500"></p>
              </div>
              <div>
                <label className="mb-1.5 block text-sm font-medium text-gray-300">Upload Bukti Transfer</label>
                <input type="file" accept="image/*" onChange={(e) => setPaymentProof(e.target.files?.[0] || null)} className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-3 text-sm text-white file:mr-4 file:rounded-lg file:border-0 file:bg-[#D4AF37] file:px-4 file:py-2 file:text-sm file:font-bold file:text-[#0F172A]" />
              </div>
              <p className="text-xs text-gray-500">
                Registration fee + SPP akan dihitung oleh admin berdasarkan data anak yang didaftarkan.
              </p>
            </div>
          )}
        </div>

        {/* Navigation */}
        <div className="mt-6 flex items-center justify-between">
          {step > 1 ? (
            <button onClick={() => setStep(step - 1)} className="rounded-lg border border-white/20 px-6 py-3 text-sm font-semibold text-white transition-all hover:bg-white/5">
              Kembali
            </button>
          ) : (
            <Link href="/" className="text-sm text-gray-500 hover:text-white">
              ← Kembali
            </Link>
          )}

          {step < 5 ? (
            <button onClick={() => setStep(step + 1)} className="rounded-lg bg-[#D4AF37] px-6 py-3 text-sm font-bold text-[#0F172A] transition-all hover:bg-[#D4AF37]/90">
              Lanjut
            </button>
          ) : (
            <button onClick={handleSubmit} disabled={submitting} className="rounded-lg bg-[#D4AF37] px-6 py-3 text-sm font-bold text-[#0F172A] transition-all hover:bg-[#D4AF37]/90 disabled:opacity-50">
              {submitting ? "Mengirim..." : "Kirim Pendaftaran"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
