/**
 * BMD Landing Page — Rebuilt from scratch
 * Inspired by: oryzo.ai (typography, spacing, animations)
 * Applied to: BMD Music School (light theme, Indonesian, music school content)
 * Design spec: design/DESIGN.md
 */

import Navbar from "@/components/landing/Navbar";
import HeroSection from "@/components/landing/HeroSection";
import ProgramKursusSection from "@/components/landing/ProgramKursusSection";
import KenapaBMDSection from "@/components/landing/KenapaBMDSection";
import StudioTourSection from "@/components/landing/StudioTourSection";
import TestimonialSection from "@/components/landing/TestimonialSection";
import PaketHargaSection from "@/components/landing/PaketHargaSection";
import FaqSection from "@/components/landing/FaqSection";
import CtaSection from "@/components/landing/CtaSection";
import Footer from "@/components/landing/Footer";

export default function HomePage() {
  return (
    <main className="min-h-screen bg-[#F8FAFC]">
      <Navbar />
      <HeroSection />
      <ProgramKursusSection />
      <KenapaBMDSection />
      <StudioTourSection />
      <TestimonialSection />
      <PaketHargaSection />
      <FaqSection />
      <CtaSection />
      <Footer />
    </main>
  );
}
