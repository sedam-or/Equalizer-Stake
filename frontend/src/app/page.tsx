"use client";
/**
 * Landing Page — T-136 Redesign
 * Light theme, emotional copy, no snap scroll
 */

import Navbar from "@/components/landing/Navbar";
import Footer from "@/components/landing/Footer";
import HeroSection from "@/components/landing/HeroSection";
import ProgramKursusSection from "@/components/landing/ProgramKursusSection";
import KenapaBMDSection from "@/components/landing/KenapaBMDSection";
import StudioTourSection from "@/components/landing/StudioTourSection";
import TestimonialSection from "@/components/landing/TestimonialSection";
import PaketHargaSection from "@/components/landing/PaketHargaSection";
import CtaSection from "@/components/landing/CtaSection";

export default function LandingPage() {
  return (
    <div className="relative bg-[#F8FAFC]">
      <Navbar />

      {/* Hero */}
      <HeroSection />

      {/* Trust bar — social proof */}
      <section className="border-y border-gray-200 bg-white py-8">
        <div className="mx-auto flex max-w-4xl flex-wrap items-center justify-center gap-8 text-center">
          <div>
            <div className="font-heading text-3xl font-bold text-[#2563EB]">150+</div>
            <div className="text-sm text-gray-500">Siswa Aktif</div>
          </div>
          <div>
            <div className="font-heading text-3xl font-bold text-[#2563EB]">9</div>
            <div className="text-sm text-gray-500">Instrumen</div>
          </div>
          <div>
            <div className="font-heading text-3xl font-bold text-[#2563EB]">10+</div>
            <div className="text-sm text-gray-500">Tahun Pengalaman</div>
          </div>
          <div>
            <div className="font-heading text-3xl font-bold text-[#2563EB]">4.9★</div>
            <div className="text-sm text-gray-500">Rating Google</div>
          </div>
        </div>
      </section>

      {/* Program Kursus */}
      <ProgramKursusSection />

      {/* Kenapa BMD */}
      <KenapaBMDSection />

      {/* Studio Tour */}
      <StudioTourSection />

      {/* Testimonial */}
      <TestimonialSection />

      {/* Paket & Harga */}
      <PaketHargaSection />

      {/* CTA */}
      <CtaSection />

      <Footer />
    </div>
  );
}
