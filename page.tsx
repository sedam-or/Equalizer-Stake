"use client";
import dynamic from "next/dynamic";

const Iridescence = dynamic(() => import("@/components/Iridescence"), { ssr: false });
const Navbar = dynamic(() => import("@/components/Navbar"), { ssr: false });
const HeroSection = dynamic(() => import("@/components/HeroSection"), { ssr: false });
const TransitionSection = dynamic(() => import("@/components/TransitionSection"), { ssr: false });
const FeatureCards = dynamic(() => import("@/components/FeatureCards"), { ssr: false });
const HowItWorks = dynamic(() => import("@/components/HowItWorks"), { ssr: false });
const FAQSection = dynamic(() => import("@/components/FAQSection"), { ssr: false });
const Footer = dynamic(() => import("@/components/Footer"), { ssr: false });
const GradualBlur = dynamic(() => import("@/components/GradualBlur"), { ssr: false });

export default function LandingPage() {
  return (
    <main id="main-content" className="relative min-h-screen">
      {/* ONE Iridescence for entire page */}
      <div className="fixed inset-0 z-0" aria-hidden="true">
        <Iridescence color={[0, 0.2, 0.61]} speed={0.7} amplitude={0.1} mouseReact={true} />
      </div>

      {/* All content on top */}
      <div className="relative z-10 snap-container">
        <Navbar />

        <div className="snap-section relative">
          <HeroSection />
          <GradualBlur target="parent" position="bottom" height="8rem" strength={2} divCount={6} curve="bezier" exponential={true} opacity={1} />
        </div>

        <div className="snap-section relative">
          <TransitionSection />
          <GradualBlur target="parent" position="bottom" height="6rem" strength={1.5} divCount={5} curve="bezier" exponential={true} opacity={1} />
        </div>

        <div className="snap-section relative">
          <FeatureCards />
          <GradualBlur target="parent" position="bottom" height="6rem" strength={1.5} divCount={5} curve="bezier" exponential={true} opacity={1} />
        </div>

        <div className="snap-section relative">
          <HowItWorks />
          <GradualBlur target="parent" position="bottom" height="6rem" strength={1.5} divCount={5} curve="bezier" exponential={true} opacity={1} />
        </div>

        <div className="snap-section relative">
          <FAQSection />
        </div>

        <Footer />
      </div>
    </main>
  );
}
