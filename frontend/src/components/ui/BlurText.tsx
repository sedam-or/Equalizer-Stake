"use client";

import { useEffect, useRef, useState, useMemo } from "react";

interface BlurTextProps {
  text: string;
  delay?: number;
  className?: string;
  animateBy?: "words" | "letters";
  direction?: "top" | "bottom";
  threshold?: number;
  rootMargin?: string;
  stepDuration?: number;
}

export default function BlurText({
  text = "",
  delay = 150,
  className = "",
  animateBy = "words",
  direction = "top",
  threshold = 0.1,
  rootMargin = "0px",
  stepDuration = 0.35,
}: BlurTextProps) {
  const elements = animateBy === "words" ? text.split(" ") : text.split("");
  const [inView, setInView] = useState(false);
  const ref = useRef<HTMLParagraphElement>(null);

  useEffect(() => {
    if (!ref.current) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setInView(true);
          observer.disconnect();
        }
      },
      { threshold, rootMargin }
    );
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, [threshold, rootMargin]);

  const fromY = direction === "top" ? -30 : 30;

  return (
    <p ref={ref} className={className} style={{ display: "flex", flexWrap: "wrap", justifyContent: "inherit" }}>
      {elements.map((segment, index) => {
        const delayMs = (index * delay) / 1000;
        return (
          <span
            key={index}
            className="inline-block"
            style={{
              filter: inView ? "blur(0px)" : "blur(8px)",
              opacity: inView ? 1 : 0,
              transform: inView ? "translateY(0)" : `translateY(${fromY}px)`,
              transition: `filter ${stepDuration}s ease ${delayMs}s, opacity ${stepDuration}s ease ${delayMs}s, transform ${stepDuration}s ease ${delayMs}s`,
              willChange: "filter, opacity, transform",
            }}
          >
            {segment === " " ? "\u00A0" : segment}
            {animateBy === "words" && index < elements.length - 1 && "\u00A0"}
          </span>
        );
      })}
    </p>
  );
}
