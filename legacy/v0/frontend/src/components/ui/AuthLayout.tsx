"use client";

import dynamic from "next/dynamic";

const ColorBends = dynamic(() => import("@/components/ui/ColorBends"), { ssr: false });

const AUTH_COLORS = ["#08cb00", "#CBD5E1", "#00ffd1", "#0a0a0a"];

export default function AuthLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="relative flex min-h-screen items-center justify-center overflow-hidden bg-[#F8FAFC] px-4 py-12">
      {/* ColorBends background */}
      <div className="pointer-events-none absolute inset-0 z-0" aria-hidden="true">
        <ColorBends
          colors={AUTH_COLORS}
          rotation={90}
          speed={0.15}
          scale={1}
          frequency={1}
          warpStrength={1}
          mouseInfluence={0.8}
          noise={0.12}
          parallax={0.4}
          iterations={2}
          intensity={1.2}
          bandWidth={5}
          transparent
        />
      </div>

      {/* Dark overlay for readability */}
      <div className="pointer-events-none absolute inset-0 z-[1] bg-[#F8FAFC]/50" aria-hidden="true" />

      {/* Content card */}
      <div className="relative z-10 w-full max-w-[720px]">
        {children}
      </div>
    </div>
  );
}
