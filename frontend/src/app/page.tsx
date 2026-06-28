"use client";
import dynamic from "next/dynamic";

const Navbar = dynamic(() => import("@/components/landing/Navbar"), { ssr: false });
const HeroSection = dynamic(() => import("@/components/landing/HeroSection"), { ssr: false });
const ProgramKursusSection = dynamic(() => import("@/components/landing/ProgramKursusSection"), { ssr: false });
const KenapaBMDSection = dynamic(() => import("@/components/landing/KenapaBMDSection"), { ssr: false });
const StudioTourSection = dynamic(() => import("@/components/landing/StudioTourSection"), { ssr: false });
const TestimonialSection = dynamic(() => import("@/components/landing/TestimonialSection"), { ssr: false });
const PaketHargaSection = dynamic(() => import("@/components/landing/PaketHargaSection"), { ssr: false });
const CtaSection = dynamic(() => import("@/components/landing/CtaSection"), { ssr: false });
const Footer = dynamic(() => import("@/components/landing/Footer"), { ssr: false });
const GradualBlur = dynamic(() => import("@/components/ui/GradualBlur"), { ssr: false });

export default function LandingPage() {
  return (
    <main id="main-content" className="relative min-h-screen bg-[#000000]">
      <div className="snap-container">
        <Navbar />

        {/* Hero — FloatingLines bg + SplashCursor inside */}
        <div className="snap-section relative">
          <HeroSection />
          <GradualBlur position="bottom" height="8rem" strength={2} divCount={5} curve="bezier" exponential={true} opacity={1} />
        </div>

        {/* Program Kursus */}
        <div className="snap-section relative">
          <ProgramKursusSection />
          <GradualBlur position="bottom" height="6rem" strength={1.5} divCount={5} curve="bezier" exponential={true} opacity={1} />
        </div>

        {/* Kenapa BMD */}
        <div className="snap-section relative">
          <KenapaBMDSection />
          <GradualBlur position="bottom" height="6rem" strength={1.5} divCount={5} curve="bezier" exponential={true} opacity={1} />
        </div>

        {/* Studio Tour */}
        <div className="snap-section relative">
          <StudioTourSection />
          <GradualBlur position="bottom" height="6rem" strength={1.5} divCount={5} curve="bezier" exponential={true} opacity={1} />
        </div>

        {/* Testimonial */}
        <div className="snap-section relative">
          <TestimonialSection />
          <GradualBlur position="bottom" height="6rem" strength={1.5} divCount={5} curve="bezier" exponential={true} opacity={1} />
        </div>

        {/* Paket & Harga */}
        <div className="snap-section relative">
          <PaketHargaSection />
          <GradualBlur position="bottom" height="6rem" strength={1.5} divCount={5} curve="bezier" exponential={true} opacity={1} />
        </div>

        {/* CTA */}
        <div className="snap-section relative">
          <CtaSection />
        </div>

        <Footer />
      </div>
    </main>
  );
}
