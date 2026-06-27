"use client";

import { useEffect, useRef } from "react";

/**
 * AnimatedLines — Pure CSS/canvas animated background.
 * Mimics FloatingLines without three.js dependency.
 * Visible animation as proof of success.
 */
export default function AnimatedLines({
  color = "#08CB00",
  opacity = 0.15,
  lineCount = 12,
  speed = 0.5,
  className = "",
}: {
  color?: string;
  opacity?: number;
  lineCount?: number;
  speed?: number;
  className?: string;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let animId: number;
    let time = 0;

    const resize = () => {
      canvas.width = canvas.offsetWidth * window.devicePixelRatio;
      canvas.height = canvas.offsetHeight * window.devicePixelRatio;
      ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    };
    resize();
    window.addEventListener("resize", resize);

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

    const draw = () => {
      const w = canvas.offsetWidth;
      const h = canvas.offsetHeight;
      ctx.clearRect(0, 0, w, h);

      for (let i = 0; i < lineCount; i++) {
        const t = i / lineCount;
        const yBase = h * (0.1 + t * 0.8);
        const amplitude = 20 + i * 3;
        const frequency = 0.003 + i * 0.0005;
        const phase = time * speed * (0.8 + i * 0.1);

        ctx.beginPath();
        ctx.moveTo(0, yBase);

        for (let x = 0; x <= w; x += 4) {
          const y =
            yBase +
            Math.sin(x * frequency + phase) * amplitude +
            Math.sin(x * frequency * 1.5 + phase * 0.7) * (amplitude * 0.4);
          ctx.lineTo(x, y);
        }

        const alpha = opacity * (0.3 + t * 0.7);
        ctx.strokeStyle = `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha})`;
        ctx.lineWidth = 1.5;
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
      className={`pointer-events-none absolute inset-0 h-full w-full ${className}`}
      style={{ opacity }}
    />
  );
}
