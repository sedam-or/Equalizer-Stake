/**
 * Equalizer-Stake Landing Page — Premium v3
 * Dark navy + gold accent, scroll snap, micro-interactions
 * 5 sections: Hero → WhyEQS → Program → Testimonial → CTA
 */

"use client";

import { useEffect } from "react";
import Navbar from "@/components/landing/Navbar";
import HeroSection from "@/components/landing/HeroSection";
import WhyEQS from "@/components/landing/WhyEQS";
import ProgramKursusSection from "@/components/landing/ProgramKursusSection";
import TeachersSection from "@/components/landing/TeachersSection";
import TestimonialSection from "@/components/landing/TestimonialSection";
import CtaSection from "@/components/landing/CtaSection";
import Footer from "@/components/landing/Footer";

export default function HomePage() {
  // IntersectionObserver for fade-in animations
  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add("revealed");
          }
        });
      },
      { threshold: 0.1 }
    );

    document.querySelectorAll(".fade-in").forEach((el) => observer.observe(el));
    return () => observer.disconnect();
  }, []);

  return (
    <main className="scroll-container min-h-screen scroll-smooth bg-[#0F172A]">
      <a href="#main-content" className="sr-only focus:not-sr-only focus:absolute focus:z-[100] focus:rounded-lg focus:bg-white focus:px-4 focus:py-2 focus:text-sm focus:font-semibold focus:text-[#2563EB] focus:shadow-lg">
        Lewati ke konten utama
      </a>
      <Navbar />
      <div id="main-content">
        <HeroSection />
        <WhyEQS />
        <ProgramKursusSection />
        <TeachersSection />
        <TestimonialSection />
        <CtaSection />
      </div>
      <Footer />
    </main>
  );
}
