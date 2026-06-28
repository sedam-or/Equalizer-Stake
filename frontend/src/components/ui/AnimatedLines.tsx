"use client";

import { useEffect, useRef } from "react";

/**
 * AnimatedLines — Pure canvas animated background.
 * No three.js dependency. Visible animation as proof.
 */
export default function AnimatedLines({
  color = "#08CB00",
  opacity = 0.15,
  lineCount = 12,
  speed = 0.5,
}: {
  color?: string;
  opacity?: number;
  lineCount?: number;
  speed?: number;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let animId: number;
    let time = 0;

    // Parse hex to rgb
    const hexToRgb = (hex: string) => {
      const h = hex.replace("#", "");
      return {
        r: parseInt(h.substring(0, 2), 16),
        g: parseInt(h.substring(2, 4), 16),
        b: parseInt(h.substring(4, 6), 16),
      };
    };
    const rgb = hexToRgb(color);

    const resize = () => {
      const dpr = window.devicePixelRatio || 1;
      canvas.width = window.innerWidth * dpr;
      canvas.height = window.innerHeight * dpr;
      canvas.style.width = window.innerWidth + "px";
      canvas.style.height = window.innerHeight + "px";
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };
    resize();
    window.addEventListener("resize", resize);

    const draw = () => {
      const w = window.innerWidth;
      const h = window.innerHeight;
      ctx.clearRect(0, 0, w, h);

      for (let i = 0; i < lineCount; i++) {
        const t = i / lineCount;
        const yBase = h * (0.1 + t * 0.8);
        const amplitude = 30 + i * 4;
        const frequency = 0.004 + i * 0.0006;
        const phase = time * speed * (0.8 + i * 0.12);

        ctx.beginPath();
        ctx.moveTo(0, yBase);

        for (let x = 0; x <= w; x += 3) {
          const y =
            yBase +
            Math.sin(x * frequency + phase) * amplitude +
            Math.sin(x * frequency * 1.5 + phase * 0.7) * (amplitude * 0.35) +
            Math.cos(x * frequency * 0.5 + phase * 0.3) * (amplitude * 0.2);
          ctx.lineTo(x, y);
        }

        const alpha = opacity * (0.4 + t * 0.6);
        ctx.strokeStyle = `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha})`;
        ctx.lineWidth = 1.5;
        ctx.stroke();

        // Second glow line (offset)
        ctx.beginPath();
        ctx.moveTo(0, yBase + 2);
        for (let x = 0; x <= w; x += 3) {
          const y =
            yBase +
            2 +
            Math.sin(x * frequency + phase + 0.3) * amplitude * 0.5 +
            Math.sin(x * frequency * 1.5 + phase * 0.7 + 0.3) * (amplitude * 0.2);
          ctx.lineTo(x, y);
        }
        ctx.strokeStyle = `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha * 0.3})`;
        ctx.lineWidth = 3;
        ctx.stroke();
      }

      time += 0.016;
      animId = requestAnimationFrame(draw);
    };
    draw();

    return () => {
      cancelAnimationFrame(animId);
      window.removeEventListener("resize", resize);
    };
  }, [color, opacity, lineCount, speed]);

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        width: "100vw",
        height: "100vh",
        pointerEvents: "none",
        zIndex: 0,
      }}
    />
  );
}
