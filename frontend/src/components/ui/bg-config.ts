/**
 * Background Config — Ubah warna dan settings di sini saja.
 * Semua section menggunakan config ini.
 */

export const BG_CONFIG = {
  // Warna garis animasi (ubah di sini saja)
  color: "#08CB00",

  // Opacity garis (0-1, semakin kecil semakin transparan)
  opacity: 0.1,

  // Jumlah garis per section
  lines: {
    hero: 14,
    layanan: 10,
    caraKerja: 8,
    studio: 12,
    testimonial: 8,
    cta: 14,
  },

  // Kecepatan animasi per section
  speed: {
    hero: 0.5,
    layanan: 0.3,
    caraKerja: 0.4,
    studio: 0.35,
    testimonial: 0.3,
    cta: 0.5,
  },
} as const;
