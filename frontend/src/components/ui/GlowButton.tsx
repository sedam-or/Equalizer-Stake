"use client";

import { useState } from "react";

interface GlowButtonProps {
  children: React.ReactNode;
  href?: string;
  variant?: "primary" | "secondary";
  className?: string;
}

export default function GlowButton({
  children,
  href = "#",
  variant = "primary",
  className = "",
}: GlowButtonProps) {
  const [clicked, setClicked] = useState(false);

  const handleClick = () => {
    setClicked(true);
    setTimeout(() => setClicked(false), 600);
  };

  const base = "relative inline-block rounded-full px-8 py-3.5 text-base font-semibold transition-all duration-300 overflow-hidden";

  const variants = {
    primary: "bg-[#08CB00] text-black hover:bg-[#0aad00] hover:shadow-[0_0_30px_rgba(8,203,0,0.4)]",
    secondary: "border-2 border-[#08CB00] text-[#08CB00] hover:bg-[#08CB00] hover:text-black hover:shadow-[0_0_30px_rgba(8,203,0,0.3)]",
  };

  return (
    <a
      href={href}
      className={`${base} ${variants[variant]} ${className}`}
      onClick={handleClick}
    >
      {/* Glow ripple on click */}
      {clicked && (
        <span
          className="absolute inset-0 animate-ping rounded-full bg-[#08CB00]/30"
          style={{ animationDuration: "0.6s" }}
        />
      )}
      {/* Hover glow border */}
      <span className="absolute inset-0 rounded-full opacity-0 transition-opacity duration-300 hover:opacity-100" style={{ boxShadow: "0 0 20px rgba(8,203,0,0.3), inset 0 0 20px rgba(8,203,0,0.1)" }} />
      <span className="relative z-10">{children}</span>
    </a>
  );
}
