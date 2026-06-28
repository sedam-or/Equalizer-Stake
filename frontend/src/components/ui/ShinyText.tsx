"use client";

import { useState, useCallback, useEffect, useRef } from "react";

interface ShinyTextProps {
  text: string;
  color?: string;
  shineColor?: string;
  speed?: number;
  className?: string;
}

export default function ShinyText({
  text,
  color = "#EEEEEE",
  shineColor = "#08CB00",
  speed = 3,
  className = "",
}: ShinyTextProps) {
  const gradientStyle: React.CSSProperties = {
    backgroundImage: `linear-gradient(120deg, ${color} 0%, ${color} 35%, ${shineColor} 50%, ${color} 65%, ${color} 100%)`,
    backgroundSize: "200% auto",
    WebkitBackgroundClip: "text",
    backgroundClip: "text",
    WebkitTextFillColor: "transparent",
    animation: `shiny ${speed}s linear infinite`,
  };

  return (
    <>
      <style>{`
        @keyframes shiny {
          0% { background-position: 200% center; }
          100% { background-position: -200% center; }
        }
      `}</style>
      <span className={`inline-block ${className}`} style={gradientStyle}>
        {text}
      </span>
    </>
  );
}
