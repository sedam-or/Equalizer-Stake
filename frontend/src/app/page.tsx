"use client";
import dynamic from "next/dynamic";
import { BG_CONFIG } from "@/components/ui/bg-config";

const AnimatedLines = dynamic(() => import("@/components/ui/AnimatedLines"), { ssr: false });
const Navbar = dynamic(() => import("@/components/landing/Navbar"), { ssr: false });
const HeroSection = dynamic(() => import("@/components/landing/HeroSection"), { ssr: false });
const LayananSection = dynamic(() => import("@/components/landing/LayananSection"), { ssr: false });
const CaraKerjaSection = dynamic(() => import("@/components/landing/CaraKerjaSection"), { ssr: false });
const StudioSection = dynamic(() => import("@/components/landing/StudioSection"), { ssr: false });
const TestimonialSection = dynamic(() => import("@/components/landing/TestimonialSection"), { ssr: false });
const CtaSection = dynamic(() => import("@/components/landing/CtaSection"), { ssr: false });
const Footer = dynamic(() => import("@/components/landing/Footer"), { ssr: false });

function SectionBg({ children, lines, speed }: { children: React.ReactNode; lines: number; speed: number }) {
  return (
    <div className="snap-section relative overflow-hidden">
      <div className="absolute inset-0 z-0" aria-hidden="true">
        <AnimatedLines color={BG_CONFIG.color} opacity={BG_CONFIG.opacity} lineCount={lines} speed={speed} />
      </div>
      <div className="relative z-10">{children}</div>
    </div>
  );
}

export default function LandingPage() {
  const c = BG_CONFIG;
  return (
    <main id="main-content" className="relative min-h-screen bg-[#000000]">
      <div className="relative z-10 snap-container">
        <Navbar />
        <SectionBg lines={c.lines.hero} speed={c.speed.hero}><HeroSection /></SectionBg>
        <SectionBg lines={c.lines.layanan} speed={c.speed.layanan}><LayananSection /></SectionBg>
        <SectionBg lines={c.lines.caraKerja} speed={c.speed.caraKerja}><CaraKerjaSection /></SectionBg>
        <SectionBg lines={c.lines.studio} speed={c.speed.studio}><StudioSection /></SectionBg>
        <SectionBg lines={c.lines.testimonial} speed={c.speed.testimonial}><TestimonialSection /></SectionBg>
        <SectionBg lines={c.lines.cta} speed={c.speed.cta}><CtaSection /></SectionBg>
        <Footer />
      </div>
    </main>
  );
}
