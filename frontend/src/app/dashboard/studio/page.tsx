"use client";

import { useState } from "react";

interface StudioSlot {
  id: string;
  time: string;
  available: boolean;
  bookedBy?: string;
  type: "rehearsal" | "recording";
}

interface Studio {
  id: string;
  name: string;
  type: string;
  rate: number;
}

export default function StudioPage() {
  const [selectedDate, setSelectedDate] = useState(new Date().toISOString().split("T")[0]);
  const [selectedStudio, setSelectedStudio] = useState<string>("1");
  const [bookingType, setBookingType] = useState<"rehearsal" | "recording">("rehearsal");
  const [showBookingForm, setShowBookingForm] = useState(false);
  const [selectedSlot, setSelectedSlot] = useState<StudioSlot | null>(null);

  const studios: Studio[] = [
    { id: "1", name: "Studio Gitar", type: "rehearsal", rate: 50000 },
    { id: "2", name: "Studio Piano", type: "rehearsal", rate: 60000 },
    { id: "3", name: "Studio Vokal", type: "rehearsal", rate: 45000 },
    { id: "4", name: "Studio Drum", type: "rehearsal", rate: 55000 },
    { id: "5", name: "Ruang Rekaman", type: "recording", rate: 150000 },
  ];

  // Generate time slots 09:00 - 21:00
  const timeSlots: StudioSlot[] = Array.from({ length: 13 }, (_, i) => {
    const hour = 9 + i;
    const isBooked = [12, 14, 17].includes(hour); // mock booked slots
    return {
      id: `${hour}`,
      time: `${hour.toString().padStart(2, "0")}:00`,
      available: !isBooked,
      bookedBy: isBooked ? "Andi Pratama" : undefined,
      type: "rehearsal",
    };
  });

  const handleBook = (slot: StudioSlot) => {
    setSelectedSlot(slot);
    setShowBookingForm(true);
  };

  return (
    <div>
      <h1 className="text-2xl font-bold text-[#000000]">Reservasi Studio</h1>
      <p className="mt-1 text-sm text-gray-500">Booking studio rehearsal atau recording</p>

      {/* Studio selector */}
      <div className="mt-6">
        <label className="mb-2 block text-sm font-medium text-gray-700">Pilih Studio</label>
        <div className="flex flex-wrap gap-2">
          {studios.map((s) => (
            <button
              key={s.id}
              onClick={() => setSelectedStudio(s.id)}
              className={`rounded-lg px-4 py-2 text-sm font-medium transition-colors ${
                selectedStudio === s.id
                  ? "bg-[#08CB00] text-black"
                  : "bg-gray-100 text-gray-600 hover:bg-gray-200"
              }`}
            >
              {s.name}
              <span className="ml-2 text-xs opacity-70">
                Rp {s.rate.toLocaleString("id-ID")}/jam
              </span>
            </button>
          ))}
        </div>
      </div>

      {/* Booking type */}
      <div className="mt-4">
        <label className="mb-2 block text-sm font-medium text-gray-700">Tipe Booking</label>
        <div className="flex gap-2">
          <button
            onClick={() => setBookingType("rehearsal")}
            className={`rounded-lg px-4 py-2 text-sm font-medium ${
              bookingType === "rehearsal"
                ? "bg-[#08CB00] text-black"
                : "bg-gray-100 text-gray-600"
            }`}
          >
            🎸 Rehearsal (Self-booking)
          </button>
          <button
            onClick={() => setBookingType("recording")}
            className={`rounded-lg px-4 py-2 text-sm font-medium ${
              bookingType === "recording"
                ? "bg-[#08CB00] text-black"
                : "bg-gray-100 text-gray-600"
            }`}
          >
            🎙️ Recording (Admin approval)
          </button>
        </div>
      </div>

      {/* Date picker */}
      <div className="mt-4">
        <label className="mb-2 block text-sm font-medium text-gray-700">Tanggal</label>
        <input
          type="date"
          value={selectedDate}
          onChange={(e) => setSelectedDate(e.target.value)}
          className="rounded-lg border border-[#253900] bg-white px-4 py-2 text-sm text-[#000000]"
        />
      </div>

      {/* Time slots */}
      <div className="mt-6">
        <h2 className="text-lg font-semibold text-[#000000]">Slot Waktu</h2>
        <p className="text-sm text-gray-500">Klik slot untuk booking</p>

        <div className="mt-4 grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4">
          {timeSlots.map((slot) => (
            <button
              key={slot.id}
              onClick={() => slot.available && handleBook(slot)}
              disabled={!slot.available}
              className={`rounded-lg border p-3 text-center transition-colors ${
                slot.available
                  ? "border-[#253900] bg-white hover:border-[#08CB00] hover:bg-[#f0fff0]"
                  : "border-gray-200 bg-gray-50 cursor-not-allowed"
              }`}
            >
              <div className={`text-lg font-bold ${slot.available ? "text-[#08CB00]" : "text-gray-400"}`}>
                {slot.time}
              </div>
              <div className="mt-1 text-xs text-gray-500">
                {slot.available ? "Tersedia" : slot.bookedBy}
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Info */}
      <div className="mt-6 rounded-lg bg-[#f0fff0] p-4">
        <h3 className="font-semibold text-[#000000]">Ketentuan</h3>
        <ul className="mt-2 space-y-1 text-sm text-gray-600">
          <li>• Jam operasional studio: 09:00 - 21:00</li>
          <li>• Rehearsal: booking langsung, slot terkunci otomatis</li>
          <li>• Recording: perlu persetujuan Admin terlebih dahulu</li>
          <li>• Pembatalan ≥24 jam sebelum: kredit dikembalikan 50%</li>
          <li>• Pembatalan &lt;24 jam: kredit hangus</li>
        </ul>
      </div>

      {/* Booking form modal */}
      {showBookingForm && selectedSlot && (
        <BookingModal
          slot={selectedSlot}
          studio={studios.find((s) => s.id === selectedStudio)!}
          date={selectedDate}
          type={bookingType}
          onClose={() => setShowBookingForm(false)}
        />
      )}
    </div>
  );
}

function BookingModal({ slot, studio, date, type, onClose }: {
  slot: StudioSlot;
  studio: Studio;
  date: string;
  type: string;
  onClose: () => void;
}) {
  const [submitting, setSubmitting] = useState(false);
  const [result, setResult] = useState<"success" | "pending" | null>(null);

  const handleSubmit = async () => {
    setSubmitting(true);
    try {
      // TODO: POST /api/studios/book
      await new Promise((r) => setTimeout(r, 1000));
      setResult(type === "recording" ? "pending" : "success");
    } catch {
      // handle error
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md rounded-2xl bg-white p-6 shadow-xl">
        <h2 className="text-lg font-bold text-[#000000]">Konfirmasi Booking</h2>

        <div className="mt-4 space-y-3">
          <div className="flex justify-between text-sm">
            <span className="text-gray-500">Studio</span>
            <span className="font-medium">{studio.name}</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-gray-500">Tanggal</span>
            <span className="font-medium">{date}</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-gray-500">Jam</span>
            <span className="font-medium">{slot.time} - {String(parseInt(slot.time) + 1).padStart(2, "0")}:00</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-gray-500">Tipe</span>
            <span className="font-medium capitalize">{type}</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-gray-500">Biaya</span>
            <span className="font-bold text-[#08CB00]">Rp {studio.rate.toLocaleString("id-ID")}</span>
          </div>
        </div>

        {type === "recording" && (
          <div className="mt-4 rounded-lg bg-yellow-50 p-3 text-sm text-yellow-700">
            ⏳ Booking recording perlu persetujuan Admin. Anda akan diberitahu setelah disetujui.
          </div>
        )}

        {result === "success" && (
          <div className="mt-4 rounded-lg bg-[#08CB00]/10 p-3 text-sm text-[#08CB00]">
            ✅ Studio berhasil dibooking!
          </div>
        )}
        {result === "pending" && (
          <div className="mt-4 rounded-lg bg-yellow-50 p-3 text-sm text-yellow-700">
            ⏳ Booking menunggu persetujuan Admin
          </div>
        )}

        <div className="mt-6 flex gap-3">
          <button
            onClick={onClose}
            className="flex-1 rounded-lg border border-gray-300 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
          >
            Batal
          </button>
          <button
            onClick={handleSubmit}
            disabled={submitting || result !== null}
            className="flex-1 rounded-lg bg-[#08CB00] py-2 text-sm font-semibold text-black disabled:opacity-50 hover:bg-[#0aad00]"
          >
            {submitting ? "Mengirim..." : result ? "Selesai" : "Booking Sekarang"}
          </button>
        </div>
      </div>
    </div>
  );
}
