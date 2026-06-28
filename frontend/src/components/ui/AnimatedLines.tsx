"use client";

import { useEffect, useRef } from "react";

/**
 * AnimatedLines — Canvas animated background per section.
 * Position: absolute (fills parent section).
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
      const parent = canvas.parentElement;
      if (!parent) return;
      const dpr = window.devicePixelRatio || 1;
      const w = parent.clientWidth;
      const h = parent.clientHeight;
      canvas.width = w * dpr;
      canvas.height = h * dpr;
      canvas.style.width = w + "px";
      canvas.style.height = h + "px";
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };
    resize();

    const ro = new ResizeObserver(resize);
    if (canvas.parentElement) ro.observe(canvas.parentElement);

    const draw = () => {
      const w = canvas.width / (window.devicePixelRatio || 1);
      const h = canvas.height / (window.devicePixelRatio || 1);
      ctx.clearRect(0, 0, w, h);

      for (let i = 0; i < lineCount; i++) {
        const t = i / lineCount;
        const yBase = h * (0.1 + t * 0.8);
        const amplitude = 25 + i * 3;
        const frequency = 0.004 + i * 0.0005;
        const phase = time * speed * (0.8 + i * 0.12);

        ctx.beginPath();
        ctx.moveTo(0, yBase);

        for (let x = 0; x <= w; x += 3) {
          const y =
            yBase +
            Math.sin(x * frequency + phase) * amplitude +
            Math.sin(x * frequency * 1.5 + phase * 0.7) * (amplitude * 0.3) +
            Math.cos(x * frequency * 0.5 + phase * 0.3) * (amplitude * 0.15);
          ctx.lineTo(x, y);
        }

        const alpha = opacity * (0.3 + t * 0.7);
        ctx.strokeStyle = `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha})`;
        ctx.lineWidth = 1.5;
        ctx.stroke();

        // Glow line
        ctx.beginPath();
        ctx.moveTo(0, yBase + 2);
        for (let x = 0; x <= w; x += 3) {
          const y =
            yBase + 2 +
            Math.sin(x * frequency + phase + 0.3) * amplitude * 0.4 +
            Math.sin(x * frequency * 1.5 + phase * 0.7 + 0.3) * (amplitude * 0.15);
          ctx.lineTo(x, y);
        }
        ctx.strokeStyle = `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha * 0.25})`;
        ctx.lineWidth = 3;
        ctx.stroke();
      }

      time += 0.016;
      animId = requestAnimationFrame(draw);
    };
    draw();

    return () => {
      cancelAnimationFrame(animId);
      ro.disconnect();
    };
  }, [color, opacity, lineCount, speed]);

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        width: "100%",
        height: "100%",
        pointerEvents: "none",
      }}
    />
  );
}
