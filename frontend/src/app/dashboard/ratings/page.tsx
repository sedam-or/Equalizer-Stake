"use client";
/**
 * Guardian Rating Page — T-147
 * Wali Murid → Guru rating
 */

import { useState } from "react";
import { Breadcrumbs } from "@/components/Breadcrumbs";

export default function RatingsPage() {
  const [teacherId, setTeacherId] = useState("");
  const [rating, setRating] = useState(5);
  const [comment, setComment] = useState("");
  const [success, setSuccess] = useState("");
  const [error, setError] = useState("");

  const handleSubmit = async () => {
    if (!teacherId) { setError("Masukkan ID guru"); return; }
    setError(""); setSuccess("");
    const token = localStorage.getItem("token");
    try {
      const res = await fetch("/api/v1/v1/ratings/", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify({ teacher_id: teacherId, rating, comment }),
      });
      if (!res.ok) throw new Error("Gagal");
      setSuccess("Rating berhasil dikirim");
      setTeacherId(""); setComment(""); setRating(5);
    } catch { setError("Gagal mengirim rating"); }
  };

  return (
    <div className="space-y-6">
      <Breadcrumbs items={[{ label: "Rating Guru" }]} />
      <h1 className="font-heading text-2xl font-bold text-gray-900">Rate Guru</h1>

      <div className="max-w-lg space-y-4 rounded-xl border border-gray-200 bg-white p-6">
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">ID Guru</label>
          <input value={teacherId} onChange={e => setTeacherId(e.target.value)} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-[#2563EB] focus:outline-none" placeholder="ID guru" />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">Rating (1-5)</label>
          <input type="range" min={1} max={5} value={rating} onChange={e => setRating(Number(e.target.value))} className="w-full" />
          <span className="text-sm text-gray-500">{rating}/5 ⭐</span>
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">Komentar</label>
          <textarea value={comment} onChange={e => setComment(e.target.value)} rows={3} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-[#2563EB] focus:outline-none" placeholder="Tulis komentar..." />
        </div>
        {error && <p className="text-sm text-red-600">{error}</p>}
        {success && <p className="text-sm text-green-600">{success}</p>}
        <button onClick={handleSubmit} className="w-full rounded-lg bg-[#F59E0B] py-2.5 text-sm font-bold text-black hover:bg-[#D97706]">Kirim Rating</button>
      </div>
    </div>
  );
}
