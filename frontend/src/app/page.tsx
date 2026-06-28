"use client";
import dynamic from "next/dynamic";

const AnimatedLines = dynamic(() => import("@/components/ui/AnimatedLines"), { ssr: false });
const Navbar = dynamic(() => import("@/components/landing/Navbar"), { ssr: false });
const HeroSection = dynamic(() => import("@/components/landing/HeroSection"), { ssr: false });
const LayananSection = dynamic(() => import("@/components/landing/LayananSection"), { ssr: false });
const CaraKerjaSection = dynamic(() => import("@/components/landing/CaraKerjaSection"), { ssr: false });
const StudioSection = dynamic(() => import("@/components/landing/StudioSection"), { ssr: false });
const TestimonialSection = dynamic(() => import("@/components/landing/TestimonialSection"), { ssr: false });
const CtaSection = dynamic(() => import("@/components/landing/CtaSection"), { ssr: false });
const Footer = dynamic(() => import("@/components/landing/Footer"), { ssr: false });

function SectionBg({ children, lineCount = 10, speed = 0.4 }: { children: React.ReactNode; lineCount?: number; speed?: number }) {
  return (
    <div className="snap-section relative overflow-hidden">
      {/* AnimatedLines background per section */}
      <div className="absolute inset-0 z-0" aria-hidden="true">
        <AnimatedLines color="#08CB00" opacity={0.1} lineCount={lineCount} speed={speed} />
      </div>
      {/* Transparent overlay content */}
      <div className="relative z-10">
        {children}
      </div>
    </div>
  );
}

export default function LandingPage() {
  return (
    <main id="main-content" className="relative min-h-screen bg-[#000000]">
      <div className="relative z-10 snap-container">
        <Navbar />

        <SectionBg lineCount={14} speed={0.5}>
          <HeroSection />
        </SectionBg>

        <SectionBg lineCount={10} speed={0.3}>
          <LayananSection />
        </SectionBg>

        <SectionBg lineCount={8} speed={0.4}>
          <CaraKerjaSection />
        </SectionBg>

        <SectionBg lineCount={12} speed={0.35}>
          <StudioSection />
        </SectionBg>

        <SectionBg lineCount={8} speed={0.3}>
          <TestimonialSection />
        </SectionBg>

        <SectionBg lineCount={14} speed={0.5}>
          <CtaSection />
        </SectionBg>

        <Footer />
      </div>
    </main>
  );
}
