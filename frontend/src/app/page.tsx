"use client";
import dynamic from "next/dynamic";

const Navbar = dynamic(() => import("@/components/landing/Navbar"), { ssr: false });
const HeroSection = dynamic(() => import("@/components/landing/HeroSection"), { ssr: false });
const LayananSection = dynamic(() => import("@/components/landing/LayananSection"), { ssr: false });
const CaraKerjaSection = dynamic(() => import("@/components/landing/CaraKerjaSection"), { ssr: false });
const StudioSection = dynamic(() => import("@/components/landing/StudioSection"), { ssr: false });
const TestimonialSection = dynamic(() => import("@/components/landing/TestimonialSection"), { ssr: false });
const CtaSection = dynamic(() => import("@/components/landing/CtaSection"), { ssr: false });
const Footer = dynamic(() => import("@/components/landing/Footer"), { ssr: false });

export default function LandingPage() {
  return (
    <main id="main-content" className="relative min-h-screen bg-[#000000]">
      <div className="relative z-10 snap-container">
        <Navbar />
        <div className="snap-section"><HeroSection /></div>
        <div className="snap-section relative"><LayananSection /></div>
        <div className="snap-section relative"><CaraKerjaSection /></div>
        <div className="snap-section relative"><StudioSection /></div>
        <div className="snap-section relative"><TestimonialSection /></div>
        <div className="snap-section relative"><CtaSection /></div>
        <Footer />
      </div>
    </main>
  );
}
