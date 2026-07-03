"use client";

import { useEffect, useRef } from "react";

interface SplashCursorProps {
  COLOR?: string;
  RAINBOW_MODE?: boolean;
  SPLAT_RADIUS?: number;
  SPLAT_FORCE?: number;
}

export default function SplashCursor({
  COLOR = "#2563EB",
  RAINBOW_MODE = false,
  SPLAT_RADIUS = 0.15,
  SPLAT_FORCE = 4000,
}: SplashCursorProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    let isActive = true;
    const gl = canvas.getContext("webgl2", { alpha: true, premultipliedAlpha: true });
    if (!gl) return;

    // Simple fluid effect using WebGL
    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
      gl.viewport(0, 0, canvas.width, canvas.height);
    };
    resize();
    window.addEventListener("resize", resize);

    // Mouse tracking
    const mouse = { x: 0, y: 0, px: 0, py: 0, down: false };
    const handleMouseMove = (e: MouseEvent) => {
      mouse.px = mouse.x;
      mouse.py = mouse.y;
      mouse.x = e.clientX;
      mouse.y = e.clientY;
    };
    const handleMouseDown = () => { mouse.down = true; };
    const handleMouseUp = () => { mouse.down = false; };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mousedown", handleMouseDown);
    window.addEventListener("mouseup", handleMouseUp);

    // Simple particle system
    interface Particle {
      x: number; y: number;
      vx: number; vy: number;
      life: number; maxLife: number;
      r: number; g: number; b: number;
      size: number;
    }

    const particles: Particle[] = [];
    const maxParticles = 100;

    const hexToRgb = (hex: string) => {
      const h = hex.replace("#", "");
      return {
        r: parseInt(h.substring(0, 2), 16) / 255,
        g: parseInt(h.substring(2, 4), 16) / 255,
        b: parseInt(h.substring(4, 6), 16) / 255,
      };
    };
    const baseColor = hexToRgb(COLOR);

    const spawnParticle = (x: number, y: number) => {
      if (particles.length >= maxParticles) return;
      const angle = Math.random() * Math.PI * 2;
      const speed = 1 + Math.random() * 3;
      particles.push({
        x, y,
        vx: Math.cos(angle) * speed,
        vy: Math.sin(angle) * speed,
        life: 1,
        maxLife: 30 + Math.random() * 30,
        r: baseColor.r + (Math.random() - 0.5) * 0.2,
        g: baseColor.g + (Math.random() - 0.5) * 0.2,
        b: baseColor.b + (Math.random() - 0.5) * 0.2,
        size: 2 + Math.random() * 4,
      });
    };

    // Simple 2D renderer (no complex WebGL shaders)
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const draw = () => {
      if (!isActive) return;

      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Spawn particles on mouse move
      const dx = mouse.x - mouse.px;
      const dy = mouse.y - mouse.py;
      const speed = Math.sqrt(dx * dx + dy * dy);

      if (speed > 2) {
        for (let i = 0; i < Math.min(3, Math.floor(speed / 5)); i++) {
          spawnParticle(mouse.x, mouse.y);
        }
      }

      // Update and draw particles
      for (let i = particles.length - 1; i >= 0; i--) {
        const p = particles[i];
        p.x += p.vx;
        p.y += p.vy;
        p.vx *= 0.98;
        p.vy *= 0.98;
        p.life -= 1;

        if (p.life <= 0) {
          particles.splice(i, 1);
          continue;
        }

        const alpha = p.life / p.maxLife;
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.size * alpha, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(${Math.floor(p.r * 255)}, ${Math.floor(p.g * 255)}, ${Math.floor(p.b * 255)}, ${alpha * 0.6})`;
        ctx.fill();

        // Glow
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.size * alpha * 2, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(${Math.floor(p.r * 255)}, ${Math.floor(p.g * 255)}, ${Math.floor(p.b * 255)}, ${alpha * 0.15})`;
        ctx.fill();
      }

      requestAnimationFrame(draw);
    };
    draw();

    return () => {
      isActive = false;
      window.removeEventListener("resize", resize);
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mousedown", handleMouseDown);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [COLOR, RAINBOW_MODE, SPLAT_RADIUS, SPLAT_FORCE]);

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
        zIndex: 50,
      }}
    />
  );
}
