"use client";
/**
 * Guru Feedback Page — T-146
 * Guru → Wali Murid feedback
 */

import { useState } from "react";
import { Breadcrumbs } from "@/components/Breadcrumbs";

export default function FeedbackPage() {
  const [sessionId, setSessionId] = useState("");
  const [rating, setRating] = useState(5);
  const [comment, setComment] = useState("");
  const [category, setCategory] = useState("progress");
  const [success, setSuccess] = useState("");
  const [error, setError] = useState("");

  const handleSubmit = async () => {
    if (!sessionId) { setError("Pilih sesi terlebih dahulu"); return; }
    setError(""); setSuccess("");
    const token = localStorage.getItem("access_token");
    try {
      const res = await fetch("/api/v1/feedback/", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify({ session_id: sessionId, rating, comment, category }),
      });
      if (!res.ok) throw new Error("Gagal");
      setSuccess("Feedback berhasil dikirim");
      setSessionId(""); setComment(""); setRating(5);
    } catch { setError("Gagal mengirim feedback"); }
  };

  return (
    <div className="space-y-6">
      <Breadcrumbs items={[{ label: "Feedback" }]} />
      <h1 className="font-heading text-2xl font-bold text-gray-900">Kirim Feedback</h1>

      <div className="max-w-lg space-y-4 rounded-xl border border-gray-200 bg-white p-6">
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">Session ID</label>
          <input value={sessionId} onChange={e => setSessionId(e.target.value)} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-[#2563EB] focus:outline-none" placeholder="ID sesi" />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">Rating (1-5)</label>
          <input type="range" min={1} max={5} value={rating} onChange={e => setRating(Number(e.target.value))} className="w-full" />
          <span className="text-sm text-gray-500">{rating}/5</span>
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">Kategori</label>
          <select value={category} onChange={e => setCategory(e.target.value)} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm">
            <option value="progress">Progress</option>
            <option value="behavior">Behavior</option>
            <option value="recommendation">Recommendation</option>
            <option value="general">General</option>
          </select>
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">Komentar</label>
          <textarea value={comment} onChange={e => setComment(e.target.value)} rows={3} className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-[#2563EB] focus:outline-none" placeholder="Tulis komentar..." />
        </div>
        {error && <p className="text-sm text-red-600">{error}</p>}
        {success && <p className="text-sm text-green-600">{success}</p>}
        <button onClick={handleSubmit} className="w-full rounded-lg bg-[#2563EB] py-2.5 text-sm font-semibold text-white hover:bg-[#1d4ed8]">Kirim Feedback</button>
      </div>
    </div>
  );
}
