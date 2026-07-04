"use client";

import { useState } from "react";

const INSTRUMENTS = [
  { value: "piano_klasik", label: "Piano Klasik" },
  { value: "piano_pop", label: "Piano Pop" },
  { value: "vokal", label: "Vokal" },
  { value: "biola", label: "Biola" },
  { value: "gitar", label: "Gitar" },
  { value: "keyboard", label: "Keyboard" },
  { value: "drum", label: "Drum" },
  { value: "flute", label: "Flute" },
  { value: "saxophone", label: "Saxophone" },
] as const;

type Instrument = (typeof INSTRUMENTS)[number]["value"];

const LEVELS: Record<Instrument, { value: string; label: string }[]> = {
  piano_klasik: [
    { value: "pra_elementary", label: "Pra-Elementary" },
    { value: "elementary", label: "Elementary" },
    { value: "grade_1", label: "Grade 1" },
    { value: "grade_2", label: "Grade 2" },
    { value: "grade_3", label: "Grade 3" },
    { value: "grade_4", label: "Grade 4" },
    { value: "grade_5", label: "Grade 5" },
    { value: "grade_6", label: "Grade 6" },
  ],
  piano_pop: [
    { value: "beginner", label: "Beginner" },
    { value: "intermediate", label: "Intermediate" },
    { value: "intermediate_2", label: "Intermediate 2" },
  ],
  vokal: [
    { value: "basic", label: "Basic" },
    { value: "intermediate", label: "Intermediate" },
    { value: "advance", label: "Advance" },
  ],
  biola: [
    { value: "basic", label: "Basic" },
    { value: "intermediate", label: "Intermediate" },
    { value: "advance", label: "Advance" },
  ],
  gitar: [
    { value: "basic", label: "Basic" },
    { value: "intermediate", label: "Intermediate" },
    { value: "advance", label: "Advance" },
  ],
  keyboard: [
    { value: "basic", label: "Basic" },
    { value: "intermediate", label: "Intermediate" },
    { value: "advance", label: "Advance" },
  ],
  drum: [
    { value: "basic", label: "Basic" },
    { value: "intermediate", label: "Intermediate" },
    { value: "advance", label: "Advance" },
  ],
  flute: [
    { value: "basic", label: "Basic" },
    { value: "intermediate", label: "Intermediate" },
    { value: "advance", label: "Advance" },
  ],
  saxophone: [
    { value: "basic", label: "Basic" },
    { value: "intermediate", label: "Intermediate" },
    { value: "advance", label: "Advance" },
  ],
};

const CLASS_TYPES = [
  { value: "privat_30", label: "Privat 30 Menit" },
  { value: "privat_60", label: "Privat 60 Menit" },
  { value: "grup_2_30", label: "Grup 2 Orang 30 Menit" },
  { value: "grup_2_60", label: "Grup 2 Orang 60 Menit" },
  { value: "grup_3_30", label: "Grup 3 Orang 30 Menit" },
  { value: "grup_3_60", label: "Grup 3 Orang 60 Menit" },
  { value: "grup_4_30", label: "Grup 4 Orang 30 Menit" },
  { value: "grup_4_60", label: "Grup 4 Orang 60 Menit" },
  { value: "grup_5_plus_60", label: "Grup 5+ Orang 60 Menit" },
] as const;

const selectClass =
  "w-full rounded-lg border border-gray-200 bg-white px-4 py-3 text-gray-900 placeholder-gray-500 outline-none transition-colors focus:border-[#2563EB]";

export default function RegisterCoursePage() {
  const [form, setForm] = useState({
    full_name: "",
    phone_whatsapp: "",
    email: "",
    address: "",
    instrument: "" as Instrument | "",
    level: "",
    class_type: "",
  });
  const [submitting, setSubmitting] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState("");

  const handleChange = (
    e: React.ChangeEvent<
      HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement
    >
  ) => {
    const { name, value } = e.target;
    setForm((prev) => ({
      ...prev,
      [name]: value,
      ...(name === "instrument" ? { level: "" } : {}),
    }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError("");
    try {
      const res = await fetch("/api/registrations", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(form),
      });
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
        <div className="w-full max-w-md rounded-2xl border border-gray-200 bg-white p-8 text-center">
          <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-[#2563EB]/10">
            <svg
              className="h-8 w-8 text-[#2563EB]"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M5 13l4 4L19 7"
              />
            </svg>
          </div>
          <h2 className="mb-2 text-xl font-bold text-gray-900">
            Pendaftaran Berhasil!
          </h2>
          <p className="text-gray-500">
            Kami akan menghubungi Anda via WhatsApp.
          </p>
          <a
            href="/"
            className="mt-6 inline-block rounded-full bg-[#2563EB] px-6 py-3 text-sm font-semibold text-black transition-all hover:bg-[#0aad00]"
          >
            Kembali ke Beranda
          </a>
        </div>
      </div>
    );
  }

  const availableLevels =
    form.instrument && form.instrument in LEVELS
      ? LEVELS[form.instrument as Instrument]
      : [];

  return (
    <div className="min-h-screen bg-[#F8FAFC] px-4 py-16">
      <div className="mx-auto max-w-xl">
        <div className="mb-8 text-center">
          <a href="/" className="text-[#2563EB] text-sm hover:underline">
            &larr; Kembali ke Beranda
          </a>
          <h1 className="mt-4 text-3xl font-bold text-[#2563EB]">
            Daftar Kursus
          </h1>
          <p className="mt-2 text-gray-500">
            Isi formulir berikut untuk mendaftar kursus di BMD Music School
          </p>
        </div>

        <form
          onSubmit={handleSubmit}
          className="space-y-5 rounded-2xl border border-gray-200 bg-white p-6"
        >
          {/* Nama Lengkap */}
          <div>
            <label
              htmlFor="full_name"
              className="mb-1.5 block text-sm font-medium text-gray-300"
            >
              Nama Lengkap *
            </label>
            <input
              id="full_name"
              name="full_name"
              type="text"
              required
              value={form.full_name}
              onChange={handleChange}
              placeholder="Masukkan nama lengkap"
              className={selectClass}
            />
          </div>

          {/* WhatsApp */}
          <div>
            <label
              htmlFor="phone_whatsapp"
              className="mb-1.5 block text-sm font-medium text-gray-300"
            >
              No. WhatsApp *
            </label>
            <input
              id="phone_whatsapp"
              name="phone_whatsapp"
              type="tel"
              required
              value={form.phone_whatsapp}
              onChange={handleChange}
              placeholder="08xxxxxxxxxx"
              className={selectClass}
            />
          </div>

          {/* Email */}
          <div>
            <label
              htmlFor="email"
              className="mb-1.5 block text-sm font-medium text-gray-300"
            >
              Email
            </label>
            <input
              id="email"
              name="email"
              type="email"
              value={form.email}
              onChange={handleChange}
              placeholder="email@example.com"
              className={selectClass}
            />
          </div>

          {/* Alamat */}
          <div>
            <label
              htmlFor="address"
              className="mb-1.5 block text-sm font-medium text-gray-300"
            >
              Alamat
            </label>
            <textarea
              id="address"
              name="address"
              rows={3}
              value={form.address}
              onChange={handleChange}
              placeholder="Alamat lengkap"
              className={selectClass}
            />
          </div>

          {/* Instrumen */}
          <div>
            <label
              htmlFor="instrument"
              className="mb-1.5 block text-sm font-medium text-gray-300"
            >
              Instrumen *
            </label>
            <select
              id="instrument"
              name="instrument"
              required
              value={form.instrument}
              onChange={handleChange}
              className={selectClass}
            >
              <option value="">Pilih instrumen</option>
              {INSTRUMENTS.map((i) => (
                <option key={i.value} value={i.value}>
                  {i.label}
                </option>
              ))}
            </select>
          </div>

          {/* Level */}
          <div>
            <label
              htmlFor="level"
              className="mb-1.5 block text-sm font-medium text-gray-300"
            >
              Level *
            </label>
            <select
              id="level"
              name="level"
              required
              value={form.level}
              onChange={handleChange}
              disabled={!form.instrument}
              className={`${selectClass} ${!form.instrument ? "opacity-50" : ""}`}
            >
              <option value="">
                {form.instrument ? "Pilih level" : "Pilih instrumen dulu"}
              </option>
              {availableLevels.map((l) => (
                <option key={l.value} value={l.value}>
                  {l.label}
                </option>
              ))}
            </select>
          </div>

          {/* Tipe Kelas */}
          <div>
            <label
              htmlFor="class_type"
              className="mb-1.5 block text-sm font-medium text-gray-300"
            >
              Tipe Kelas *
            </label>
            <select
              id="class_type"
              name="class_type"
              required
              value={form.class_type}
              onChange={handleChange}
              className={selectClass}
            >
              <option value="">Pilih tipe kelas</option>
              {CLASS_TYPES.map((c) => (
                <option key={c.value} value={c.value}>
                  {c.label}
                </option>
              ))}
            </select>
          </div>

          {error && (
            <p className="rounded-lg bg-red-500/10 p-3 text-sm text-red-400">
              {error}
            </p>
          )}

          <button
            type="submit"
            disabled={submitting}
            className="w-full rounded-full bg-[#2563EB] py-3.5 text-base font-semibold text-black transition-all hover:bg-[#0aad00] hover:shadow-[0_0_30px_rgba(8,203,0,0.4)] disabled:opacity-50"
          >
            {submitting ? "Mengirim..." : "Daftar Sekarang"}
          </button>
        </form>
      </div>
    </div>
  );
}
