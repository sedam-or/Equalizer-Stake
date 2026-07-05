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
    <main id="main-content" className="min-h-screen scroll-smooth bg-[#F8FAFC]">
      <a href="#main-content" className="sr-only focus:not-sr-only focus:absolute focus:z-[100] focus:rounded-lg focus:bg-white focus:px-4 focus:py-2 focus:text-sm focus:font-semibold focus:text-[#2563EB] focus:shadow-lg">
        Lewati ke konten utama
      </a>
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