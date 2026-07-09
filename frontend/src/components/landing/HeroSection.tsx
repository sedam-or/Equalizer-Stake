"use client";

import { useEffect, useRef, useState } from "react";
import BlurText from "@/components/BlurText";

function CountUp({ target, suffix = "" }: { target: number; suffix?: string }) {
  const [count, setCount] = useState(0);
  const ref = useRef<HTMLSpanElement>(null);
  const started = useRef(false);

  useEffect(() => {
    const obs = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !started.current) {
          started.current = true;
          const duration = 2000;
          const start = performance.now();
          const animate = (now: number) => {
            const progress = Math.min((now - start) / duration, 1);
            const eased = 1 - Math.pow(1 - progress, 4);
            setCount(Math.floor(eased * target));
            if (progress < 1) requestAnimationFrame(animate);
          };
          requestAnimationFrame(animate);
        }
      },
      { threshold: 0.5 }
    );
    if (ref.current) obs.observe(ref.current);
    return () => obs.disconnect();
  }, [target]);

  return (
    <span ref={ref} className="tabular-nums">
      {count}{suffix}
    </span>
  );
}

export default function HeroSection() {
  return (
    <section data-theme="dark" className="snap-section relative flex min-h-[100dvh] items-center overflow-hidden bg-[#0F172A] px-6 pt-20 md:px-8">
      {/* Subtle gradient overlay */}
      <div className="absolute inset-0 bg-gradient-to-br from-[#0F172A] via-[#1E293B] to-[#0F172A]" />

      {/* Gold accent line */}
      <div className="absolute left-0 top-0 h-full w-px bg-gradient-to-b from-transparent via-[#D4AF37]/30 to-transparent" />

      <div className="relative z-10 mx-auto grid max-w-7xl items-center gap-8 sm:gap-12 lg:gap-16 lg:grid-cols-2">
        {/* Left: Text */}
        <div>
          {/* Eyebrow */}
          <p className="mb-6 text-sm font-semibold uppercase tracking-[0.3em] text-[#D4AF37]">
            Kelas Privat Musik Premium — Yogyakarta
          </p>

          {/* Headline */}
          <BlurText
            text="Musik Bukan Sekadar Hobi."
            animateBy="words"
            direction="top"
            delay={100}
            className="text-balance font-heading text-3xl font-bold leading-[1.15] tracking-tight text-white sm:text-4xl lg:text-5xl"
          />
          <BlurText
            text="Warisan untuk Masa Depan Anak."
            animateBy="words"
            direction="top"
            delay={150}
            className="mt-2 text-balance font-heading text-3xl font-bold leading-[1.15] tracking-tight text-[#D4AF37] sm:text-4xl lg:text-5xl"
          />

          {/* Sub */}
          <p className="mt-6 max-w-lg text-lg leading-relaxed text-gray-400">
            BMD Music School — kelas privat musik premium untuk keluarga yang menghargai kualitas. Pengalaman 10+ tahun, guru berstandar internasional.
          </p>

          {/* CTA */}
          <div className="mt-10 flex flex-col items-start gap-4 sm:flex-row">
            <a
              href="/register-course"
              className="group inline-flex items-center gap-2 rounded-lg border border-white/20 bg-white/5 px-8 py-4 text-sm font-semibold text-white backdrop-blur-sm transition-all duration-300 hover:border-[#D4AF37]/50 hover:bg-[#D4AF37]/10 hover:text-[#D4AF37]"
            >
              Mulai Perjalanan Musik
              <span className="transition-transform duration-300 group-hover:translate-x-1">→</span>
            </a>
            <a
              href="#instrumen"
              className="inline-flex items-center gap-2 px-4 py-4 text-sm font-medium text-gray-500 transition-colors duration-300 hover:text-white"
            >
              Lihat Program
            </a>
          </div>

          {/* Stats */}
          <div className="mt-8 flex flex-wrap items-center gap-4 sm:gap-8 border-t border-white/10 pt-8">
            <div>
              <div className="font-heading text-2xl font-bold text-white">
                <CountUp target={150} suffix="+" />
              </div>
              <div className="mt-1 text-xs uppercase tracking-wider text-gray-500">Keluarga</div>
            </div>
            <div>
              <div className="font-heading text-2xl font-bold text-white">
                <CountUp target={10} suffix="+" />
              </div>
              <div className="mt-1 text-xs uppercase tracking-wider text-gray-500">Tahun</div>
            </div>
            <div>
              <div className="font-heading text-2xl font-bold text-white">
                <CountUp target={4.9} suffix="★" />
              </div>
              <div className="mt-1 text-xs uppercase tracking-wider text-gray-500">Google</div>
            </div>
          </div>
        </div>

        {/* Right: Visual */}
        <div className="relative hidden lg:block">
          <div className="relative overflow-hidden rounded-sm">
            <img
              src="/images/screen.png"
              alt="Studio BMD Music School"
              className="h-[600px] w-full object-cover"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-[#0F172A]/80 via-transparent to-[#0F172A]/20" />
          </div>
          {/* Gold corner accent */}
          <div className="absolute -right-4 -top-4 h-24 w-24 border-r-2 border-t-2 border-[#D4AF37]/30" />
          <div className="absolute -bottom-4 -left-4 h-24 w-24 border-b-2 border-l-2 border-[#D4AF37]/30" />
        </div>
      </div>

      {/* Scroll indicator */}
      <div className="absolute bottom-8 left-1/2 z-10 -translate-x-1/2">
        <div className="flex flex-col items-center gap-2 text-gray-500">
          <span className="text-xs uppercase tracking-widest">Scroll</span>
          <div className="h-10 w-px animate-pulse bg-gradient-to-b from-gray-500 to-transparent" />
        </div>
      </div>
    </section>
  );
}
