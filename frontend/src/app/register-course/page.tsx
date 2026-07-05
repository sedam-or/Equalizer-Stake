"use client";

import { useState } from "react";

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

interface Child {
  full_name: string;
  instrument: string;
  level: string;
}

export default function RegisterCoursePage() {
  const [step, setStep] = useState(1);
  const [form, setForm] = useState({
    full_name: "", phone_whatsapp: "", email: "", address: "",
    username: "", password: "",
  });
  const [children, setChildren] = useState<Child[]>([{ full_name: "", instrument: "piano", level: "pemula" }]);
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

  const addChild = () => {
    setChildren([...children, { full_name: "", instrument: "piano", level: "pemula" }]);
  };

  const removeChild = (index: number) => {
    if (children.length > 1) setChildren(children.filter((_, i) => i !== index));
  };

  const nextStep = () => { setError(""); if (step < 5) setStep(step + 1); };
  const prevStep = () => { setError(""); if (step > 1) setStep(step - 1); };

  const handleSubmit = async () => {
    setSubmitting(true);
    setError("");
    try {
      const formData = new FormData();
      Object.entries(form).forEach(([k, v]) => formData.append(k, v));
      formData.append("children", JSON.stringify(children));
      if (paymentProof) formData.append("payment_proof", paymentProof);

      const res = await fetch("/api/v1/registrations/", { method: "POST", body: formData });
      if (!res.ok) throw new Error("Gagal mengirim pendaftaran");
      setSuccess(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Terjadi kesalahan");
    } finally {
      setSubmitting(false);
    }
  };

  if (success) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-[#F8FAFC] px-4">
        <div className="w-full max-w-md rounded-xl border border-gray-200 bg-white p-8 text-center">
          <span className="text-5xl">✅</span>
          <h1 className="mt-4 font-heading text-2xl font-bold text-gray-900">Pendaftaran Berhasil!</h1>
          <p className="mt-2 text-sm text-gray-500">Akun Anda telah dibat. Admin akan memverifikasi pembayaran dalam 1×24 jam.</p>
          <a href="/login" className="mt-6 inline-block rounded-lg bg-[#2563EB] px-5 py-2.5 text-sm font-semibold text-white hover:bg-[#1D4ED8]">Login</a>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[#F8FAFC] px-4 py-12">
      <div className="mx-auto max-w-2xl">
        <div className="text-center">
          <h1 className="font-heading text-3xl font-bold text-gray-900">Daftar Kursus Musik</h1>
          <p className="mt-2 text-gray-500">Isi formulir untuk mendaftarkan anak Anda kursus di BMD Music School</p>
        </div>

        {/* Progress */}
        <div className="mt-8 flex items-center justify-between">
          {steps.map((s, i) => (
            <div key={s.id} className="flex items-center">
              <div className={`flex h-8 w-8 items-center justify-center rounded-full text-sm font-bold ${step >= s.id ? "bg-[#2563EB] text-white" : "bg-gray-200 text-gray-500"}`}>
                {step > s.id ? "✓" : s.id}
              </div>
              <span className={`ml-2 text-xs ${step >= s.id ? "text-[#2563EB]" : "text-gray-400"}`}>{s.label}</span>
              {i < steps.length - 1 && <div className={`mx-2 h-px w-8 ${step > s.id ? "bg-[#2563EB]" : "bg-gray-200"}`} />}
            </div>
          ))}
        </div>

        <div className="mt-8 rounded-xl border border-gray-200 bg-white p-6">
          {/* Step 1: Data Diri */}
          {step === 1 && (
            <div className="space-y-4">
              <h2 className="font-heading text-xl font-bold text-gray-900">Data Diri Wali Murid</h2>
              <div><label className="mb-1 block text-sm font-medium text-gray-700">Nama Lengkap *</label><input type="text" name="full_name" value={form.full_name} onChange={handleChange} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" placeholder="Nama lengkap" /></div>
              <div><label className="mb-1 block text-sm font-medium text-gray-700">Nomor WhatsApp *</label><input type="tel" name="phone_whatsapp" value={form.phone_whatsapp} onChange={handleChange} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" placeholder="08xxxxxxxxxx" /></div>
              <div><label className="mb-1 block text-sm font-medium text-gray-700">Email *</label><input type="email" name="email" value={form.email} onChange={handleChange} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" placeholder="email@example.com" /></div>
              <div><label className="mb-1 block text-sm font-medium text-gray-700">Alamat</label><textarea name="address" value={form.address} onChange={handleChange} rows={3} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" placeholder="Alamat lengkap" /></div>
            </div>
          )}

          {/* Step 2: Buat Akun */}
          {step === 2 && (
            <div className="space-y-4">
              <h2 className="font-heading text-xl font-bold text-gray-900">Buat Akun</h2>
              <p className="text-sm text-gray-500">Buat akun untuk memantau progress kursus anak Anda.</p>
              <div><label className="mb-1 block text-sm font-medium text-gray-700">Username *</label><input type="text" name="username" value={form.username} onChange={handleChange} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" placeholder="Minimal 3 karakter" /></div>
              <div><label className="mb-1 block text-sm font-medium text-gray-700">Password *</label><input type="password" name="password" value={form.password} onChange={handleChange} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" placeholder="Minimal 8 karakter" /></div>
            </div>
          )}

          {/* Step 3: Tambah Anak */}
          {step === 3 && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h2 className="font-heading text-xl font-bold text-gray-900">Data Anak</h2>
                <button onClick={addChild} className="rounded-lg border border-[#2563EB] px-3 py-1.5 text-xs font-semibold text-[#2563EB]">+ Tambah Anak</button>
              </div>
              <p className="text-sm text-gray-500">Daftarkan minimal 1 anak. Bisa tambah lebih dari 1.</p>
              {children.map((child, i) => (
                <div key={i} className="rounded-lg border border-gray-200 p-4">
                  <div className="flex items-center justify-between">
                    <h3 className="text-sm font-bold text-gray-900">Anak {i + 1}</h3>
                    {children.length > 1 && <button onClick={() => removeChild(i)} className="text-xs text-red-500">Hapus</button>}
                  </div>
                  <div className="mt-3 space-y-3">
                    <div><label className="mb-1 block text-xs text-gray-600">Nama *</label><input type="text" value={child.full_name} onChange={(e) => handleChildChange(i, "full_name", e.target.value)} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm" placeholder="Nama anak" /></div>
                    <div className="grid grid-cols-2 gap-3">
                      <div><label className="mb-1 block text-xs text-gray-600">Instrumen *</label>
                        <select value={child.instrument} onChange={(e) => handleChildChange(i, "instrument", e.target.value)} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm capitalize">{INSTRUMENTS.map((x) => <option key={x} value={x}>{INSTRUMENT_LABELS[x] || x}</option>)}</select>
                      </div>
                      <div><label className="mb-1 block text-xs text-gray-600">Level</label>
                        <select value={child.level} onChange={(e) => handleChildChange(i, "level", e.target.value)} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm capitalize">{LEVELS.map((x) => <option key={x} value={x}>{x}</option>)}</select>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Step 4: Pilih Jadwal */}
          {step === 4 && (
            <div className="space-y-4">
              <h2 className="font-heading text-xl font-bold text-gray-900">Pilih Jadwal</h2>
              <p className="text-sm text-gray-500">Jadwal akan diatur oleh admin setelah pendaftaran diverifikasi.</p>
              <div className="rounded-lg bg-blue-50 p-4">
                <p className="text-sm text-blue-800">Admin akan menghubungi Anda via WhatsApp untuk konfirmasi jadwal kursus anak Anda.</p>
              </div>
            </div>
          )}

          {/* Step 5: Pembayaran */}
          {step === 5 && (
            <div className="space-y-4">
              <h2 className="font-heading text-xl font-bold text-gray-900">Pembayaran</h2>
              <div className="rounded-lg bg-gray-50 p-4">
                <h3 className="text-sm font-bold text-gray-900">Transfer ke:</h3>
                <p className="mt-1 text-lg font-bold text-[#2563EB]">BCA 1234567890</p>
                <p className="text-sm text-gray-500">a/n BMD Music School</p>
              </div>
              <div className="rounded-lg bg-gray-50 p-4">
                <h3 className="text-sm font-bold text-gray-900">Total Pembayaran:</h3>
                <p className="mt-1 text-sm text-gray-500">Registration Fee + SPP akan dihitung oleh admin berdasarkan data anak yang didaftarkan.</p>
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Upload Bukti Transfer *</label>
                <input type="file" accept="image/*" onChange={(e) => setPaymentProof(e.target.files?.[0] || null)} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm" />
                <p className="mt-1 text-xs text-gray-500">Format: JPG, PNG. Maks 5MB.</p>
              </div>
            </div>
          )}

          {error && <p className="mt-4 text-sm text-red-600">{error}</p>}

          <div className="mt-6 flex items-center justify-between">
            {step > 1 && <button onClick={prevStep} className="rounded-lg border border-gray-300 px-4 py-2.5 text-sm font-medium text-gray-700 hover:bg-gray-50">Kembali</button>}
            {step < 5 && <button onClick={nextStep} className="ml-auto rounded-lg bg-[#2563EB] px-6 py-2.5 text-sm font-semibold text-white hover:bg-[#1D4ED8]">Lanjut</button>}
            {step === 5 && <button onClick={handleSubmit} disabled={submitting || !paymentProof} className="ml-auto rounded-lg bg-[#2563EB] px-6 py-2.5 text-sm font-semibold text-white hover:bg-[#1D4ED8] disabled:opacity-50">{submitting ? "Mengirim..." : "Kirim Pendaftaran"}</button>}
          </div>
        </div>
      </div>
    </div>
  );
}
