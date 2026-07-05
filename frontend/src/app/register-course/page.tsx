"use client";

import { useState } from "react";

// Step indicators
const steps = [
  { id: 1, label: "Data Diri" },
  { id: 2, label: "Pilih Kursus" },
  { id: 3, label: "Pilih Jadwal" },
  { id: 4, label: "Pembayaran" },
  { id: 5, label: "Selesai" },
];

// Instrument data from SRS
const instruments = [
  { value: "piano_klasik", label: "Piano Klasik", regFee: 420000, spp: 420000, duration: "30-45 menit" },
  { value: "piano_pop", label: "Piano Pop", regFee: 420000, spp: 600000, duration: "45 menit" },
  { value: "vokal", label: "Vokal", regFee: 420000, spp: 600000, duration: "45 menit" },
  { value: "biola", label: "Biola", regFee: 540000, spp: 420000, duration: "45 menit" },
  { value: "gitar", label: "Gitar Akustik", regFee: 420000, spp: 420000, duration: "45 menit" },
  { value: "keyboard", label: "Keyboard", regFee: 480000, spp: 420000, duration: "45 menit" },
  { value: "drum", label: "Drum", regFee: 420000, spp: 420000, duration: "45 menit" },
  { value: "flute", label: "Flute", regFee: 420000, spp: 420000, duration: "45 menit" },
  { value: "saxophone", label: "Saxophone", regFee: 420000, spp: 420000, duration: "45 menit" },
];

const classTypes = [
  { value: "private", label: "Private (1 siswa)", priceMultiplier: 1 },
  { value: "intensif_2", label: "Intensif (2 siswa)", priceMultiplier: 0.65 },
  { value: "semi_intensif_4", label: "Semi Intensif (4 siswa)", priceMultiplier: 0.5 },
];

// Mock schedule data (will be replaced with API call in T-159)
const mockSchedules = [
  { id: "1", day: "Senin", time: "09:00-09:45", room: "A1", teacher: "Pak Dimas" },
  { id: "2", day: "Senin", time: "10:00-10:45", room: "A2", teacher: "Mbak Sari" },
  { id: "3", day: "Selasa", time: "14:00-14:45", room: "B1", teacher: "Pak Rizky" },
  { id: "4", day: "Rabu", time: "16:00-16:45", room: "A1", teacher: "Pak Dimas" },
  { id: "5", day: "Kamis", time: "09:00-09:45", room: "B2", teacher: "Mbak Sari" },
  { id: "6", day: "Jumat", time: "15:00-15:45", room: "A1", teacher: "Pak Rizky" },
  { id: "7", day: "Sabtu", time: "09:00-09:45", room: "A2", teacher: "Pak Dimas" },
];

function formatRupiah(amount: number): string {
  return `Rp ${amount.toLocaleString("id-ID")}`;
}

export default function RegisterCoursePage() {
  const [step, setStep] = useState(1);
  const [form, setForm] = useState({
    full_name: "",
    phone_whatsapp: "",
    email: "",
    address: "",
    instrument: "",
    class_type: "",
    schedule_id: "",
  });
  const [paymentProof, setPaymentProof] = useState<File | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState("");

  const selectedInstrument = instruments.find((i) => i.value === form.instrument);
  const selectedClassType = classTypes.find((c) => c.value === form.class_type);
  const selectedSchedule = mockSchedules.find((s) => s.id === form.schedule_id);

  const totalBiaya = selectedInstrument
    ? selectedInstrument.regFee + Math.round(selectedInstrument.spp * (selectedClassType?.priceMultiplier || 1))
    : 0;

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => {
    const { name, value } = e.target;
    setForm((prev) => ({ ...prev, [name]: value }));
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0] || null;
    setPaymentProof(file);
  };

  const nextStep = () => {
    setError("");
    if (step < 5) setStep(step + 1);
  };

  const prevStep = () => {
    setError("");
    if (step > 1) setStep(step - 1);
  };

  const handleSubmit = async () => {
    setSubmitting(true);
    setError("");
    try {
      const formData = new FormData();
      formData.append("full_name", form.full_name);
      formData.append("phone_whatsapp", form.phone_whatsapp);
      formData.append("email", form.email);
      formData.append("address", form.address);
      formData.append("instrument", form.instrument);
      formData.append("class_type", form.class_type);
      formData.append("schedule_id", form.schedule_id);
      if (paymentProof) {
        formData.append("payment_proof", paymentProof);
      }

      const res = await fetch("/api/v1/registrations/", {
        method: "POST",
        body: formData,
      });
      if (!res.ok) throw new Error("Gagal mengirim pendaftaran");
      setSuccess(true);
      setStep(5);
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
          <p className="mt-2 text-sm text-gray-500">
            Pendaftaran Anda telah diterima. Admin akan memverifikasi pembayaran dalam 1×24 jam.
            Anda akan menerima notifikasi via WhatsApp.
          </p>
          <a href="/" className="mt-6 inline-block rounded-lg bg-[#2563EB] px-5 py-2.5 text-sm font-semibold text-white hover:bg-[#1D4ED8]">
            Kembali ke Beranda
          </a>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[#F8FAFC] px-4 py-12">
      <div className="mx-auto max-w-2xl">
        {/* Header */}
        <div className="text-center">
          <h1 className="font-heading text-3xl font-bold text-gray-900">Daftar Kursus Musik</h1>
          <p className="mt-2 text-gray-500">Isi formulir di bawah untuk mendaftar kursus di BMD Music School</p>
        </div>

        {/* Progress indicator */}
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

        {/* Form card */}
        <div className="mt-8 rounded-xl border border-gray-200 bg-white p-6">
          {/* Step 1: Data Diri */}
          {step === 1 && (
            <div className="space-y-4">
              <h2 className="font-heading text-xl font-bold text-gray-900">Data Diri Wali Murid</h2>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Nama Lengkap *</label>
                <input type="text" name="full_name" value={form.full_name} onChange={handleChange} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm focus:border-[#2563EB] focus:outline-none focus:ring-2 focus:ring-[#2563EB]/20" placeholder="Masukkan nama lengkap" />
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Nomor WhatsApp *</label>
                <input type="tel" name="phone_whatsapp" value={form.phone_whatsapp} onChange={handleChange} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm focus:border-[#2563EB] focus:outline-none focus:ring-2 focus:ring-[#2563EB]/20" placeholder="08xxxxxxxxxx" />
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Email</label>
                <input type="email" name="email" value={form.email} onChange={handleChange} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm focus:border-[#2563EB] focus:outline-none focus:ring-2 focus:ring-[#2563EB]/20" placeholder="email@example.com" />
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Alamat</label>
                <textarea name="address" value={form.address} onChange={handleChange} rows={3} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm focus:border-[#2563EB] focus:outline-none focus:ring-2 focus:ring-[#2563EB]/20" placeholder="Alamat lengkap" />
              </div>
            </div>
          )}

          {/* Step 2: Pilih Kursus */}
          {step === 2 && (
            <div className="space-y-4">
              <h2 className="font-heading text-xl font-bold text-gray-900">Pilih Instrumen & Tipe Kelas</h2>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Instrumen *</label>
                <select name="instrument" value={form.instrument} onChange={handleChange} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm focus:border-[#2563EB] focus:outline-none">
                  <option value="">Pilih instrumen</option>
                  {instruments.map((inst) => (
                    <option key={inst.value} value={inst.value}>{inst.label} — {formatRupiah(inst.spp)}/bulan</option>
                  ))}
                </select>
              </div>
              {selectedInstrument && (
                <div>
                  <label className="mb-1 block text-sm font-medium text-gray-700">Tipe Kelas *</label>
                  <select name="class_type" value={form.class_type} onChange={handleChange} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm focus:border-[#2563EB] focus:outline-none">
                    <option value="">Pilih tipe kelas</option>
                    {classTypes.map((ct) => (
                      <option key={ct.value} value={ct.value}>{ct.label} — {formatRupiah(Math.round(selectedInstrument.spp * ct.priceMultiplier))}/bulan</option>
                    ))}
                  </select>
                </div>
              )}
              {selectedInstrument && selectedClassType && (
                <div className="rounded-lg bg-blue-50 p-4">
                  <p className="text-sm text-blue-800">
                    <strong>Ringkasan:</strong> {selectedInstrument.label} — {selectedClassType.label}
                    <br />
                    Registration Fee: {formatRupiah(selectedInstrument.regFee)} (sekali bayar)
                    <br />
                    SPP: {formatRupiah(Math.round(selectedInstrument.spp * selectedClassType.priceMultiplier))}/bulan
                  </p>
                </div>
              )}
            </div>
          )}

          {/* Step 3: Pilih Jadwal */}
          {step === 3 && (
            <div className="space-y-4">
              <h2 className="font-heading text-xl font-bold text-gray-900">Pilih Jadwal</h2>
              <p className="text-sm text-gray-500">Pilih jadwal yang tersedia untuk {selectedInstrument?.label}</p>
              <div className="space-y-2">
                {mockSchedules.map((s) => (
                  <label key={s.id} className={`flex cursor-pointer items-center justify-between rounded-lg border p-4 transition-colors ${form.schedule_id === s.id ? "border-[#2563EB] bg-blue-50" : "border-gray-200 hover:border-gray-300"}`}>
                    <div>
                      <p className="text-sm font-medium text-gray-900">{s.day}, {s.time}</p>
                      <p className="text-xs text-gray-500">Ruangan {s.room} • {s.teacher}</p>
                    </div>
                    <input type="radio" name="schedule_id" value={s.id} checked={form.schedule_id === s.id} onChange={handleChange} className="text-[#2563EB]" />
                  </label>
                ))}
              </div>
            </div>
          )}

          {/* Step 4: Pembayaran */}
          {step === 4 && (
            <div className="space-y-4">
              <h2 className="font-heading text-xl font-bold text-gray-900">Pembayaran</h2>
              <div className="rounded-lg bg-gray-50 p-4">
                <h3 className="text-sm font-bold text-gray-900">Transfer ke:</h3>
                <p className="mt-1 text-lg font-bold text-[#2563EB]">BCA 1234567890</p>
                <p className="text-sm text-gray-500">a/n BMD Music School</p>
              </div>
              <div className="rounded-lg bg-gray-50 p-4">
                <h3 className="text-sm font-bold text-gray-900">Total Pembayaran:</h3>
                <div className="mt-2 space-y-1">
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500">Registration Fee ({selectedInstrument?.label})</span>
                    <span className="font-medium">{formatRupiah(selectedInstrument?.regFee || 0)}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500">SPP Bulan Pertama ({selectedClassType?.label})</span>
                    <span className="font-medium">{formatRupiah(Math.round((selectedInstrument?.spp || 0) * (selectedClassType?.priceMultiplier || 1)))}</span>
                  </div>
                  <div className="border-t border-gray-200 pt-2">
                    <div className="flex justify-between">
                      <span className="font-bold text-gray-900">Total</span>
                      <span className="text-lg font-bold text-[#2563EB]">{formatRupiah(totalBiaya)}</span>
                    </div>
                  </div>
                </div>
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">Upload Bukti Transfer *</label>
                <input type="file" accept="image/*" onChange={handleFileChange} className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm focus:border-[#2563EB] focus:outline-none" />
                <p className="mt-1 text-xs text-gray-500">Format: JPG, PNG. Maks 5MB.</p>
              </div>
            </div>
          )}

          {/* Error */}
          {error && <p className="mt-4 text-sm text-red-600">{error}</p>}

          {/* Navigation buttons */}
          <div className="mt-6 flex items-center justify-between">
            {step > 1 && step < 5 && (
              <button onClick={prevStep} className="rounded-lg border border-gray-300 px-4 py-2.5 text-sm font-medium text-gray-700 hover:bg-gray-50">
                Kembali
              </button>
            )}
            {step < 4 && (
              <button onClick={nextStep} className="ml-auto rounded-lg bg-[#2563EB] px-6 py-2.5 text-sm font-semibold text-white hover:bg-[#1D4ED8]">
                Lanjut
              </button>
            )}
            {step === 4 && (
              <button onClick={handleSubmit} disabled={submitting || !paymentProof} className="ml-auto rounded-lg bg-[#2563EB] px-6 py-2.5 text-sm font-semibold text-white hover:bg-[#1D4ED8] disabled:opacity-50">
                {submitting ? "Mengirim..." : "Kirim Pendaftaran"}
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
