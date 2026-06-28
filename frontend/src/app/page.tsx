"use client";
import dynamic from "next/dynamic";
import { BG_CONFIG } from "@/components/ui/bg-config";

const FloatingLines = dynamic(() => import("@/components/ui/FloatingLines"), { ssr: false });
const Navbar = dynamic(() => import("@/components/landing/Navbar"), { ssr: false });
const HeroSection = dynamic(() => import("@/components/landing/HeroSection"), { ssr: false });
const LayananSection = dynamic(() => import("@/components/landing/LayananSection"), { ssr: false });
const CaraKerjaSection = dynamic(() => import("@/components/landing/CaraKerjaSection"), { ssr: false });
const StudioSection = dynamic(() => import("@/components/landing/StudioSection"), { ssr: false });
const TestimonialSection = dynamic(() => import("@/components/landing/TestimonialSection"), { ssr: false });
const CtaSection = dynamic(() => import("@/components/landing/CtaSection"), { ssr: false });
const Footer = dynamic(() => import("@/components/landing/Footer"), { ssr: false });

function SectionBg({ children }: { children: React.ReactNode }) {
  return (
    <div className="snap-section relative overflow-hidden" style={{ minHeight: "100vh" }}>
      <div className="absolute inset-0 z-0" aria-hidden="true">
        <FloatingLines
          linesGradient={[...BG_CONFIG.colors]}
          enabledWaves={["top", "middle", "bottom"]}
          lineCount={[...BG_CONFIG.lineCount]}
          lineDistance={[...BG_CONFIG.lineDistance]}
          animationSpeed={BG_CONFIG.animationSpeed}
          interactive={BG_CONFIG.interactive}
          parallax={BG_CONFIG.parallax}
          bendRadius={5.0}
          bendStrength={-0.5}
          mixBlendMode="screen"
        />
      </div>
      <div className="relative z-10">{children}</div>
    </div>
  );
}

export default function LandingPage() {
  return (
    <main id="main-content" className="relative min-h-screen bg-[#000000]">
      <div className="relative z-10 snap-container">
        <Navbar />
        <SectionBg><HeroSection /></SectionBg>
        <SectionBg><LayananSection /></SectionBg>
        <SectionBg><CaraKerjaSection /></SectionBg>
        <SectionBg><StudioSection /></SectionBg>
        <SectionBg><TestimonialSection /></SectionBg>
        <SectionBg><CtaSection /></SectionBg>
        <Footer />
      </div>
    </main>
  );
}
