/**
 * Background Config — Ubah warna dan settings di sini saja.
 * Semua section menggunakan config ini.
 */

export const BG_CONFIG = {
  // Warna garis (gradient)
  colors: ["#08CB00", "#253900"],

  // Jumlah garis per wave [top, middle, bottom]
  lineCount: [8, 12, 16],

  // Jarak antar garis per wave
  lineDistance: [10, 8, 5],

  // Kecepatan animasi
  animationSpeed: 0.8,

  // Interaktif (garis mengikuti mouse)
  interactive: true,

  // Parallax effect
  parallax: true,
} as const;
