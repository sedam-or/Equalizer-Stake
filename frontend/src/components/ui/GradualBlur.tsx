"use client";

import React, { CSSProperties, useEffect, useRef, useState, useMemo, PropsWithChildren } from "react";

type GradualBlurProps = PropsWithChildren<{
  position?: "top" | "bottom" | "left" | "right";
  strength?: number;
  height?: string;
  divCount?: number;
  exponential?: boolean;
  zIndex?: number;
  opacity?: number;
  curve?: "linear" | "bezier" | "ease-in" | "ease-out";
  target?: "parent" | "page";
  className?: string;
  style?: CSSProperties;
}>;

const CURVE_FUNCTIONS: Record<string, (p: number) => number> = {
  linear: (p) => p,
  bezier: (p) => p * p * (3 - 2 * p),
  "ease-in": (p) => p * p,
  "ease-out": (p) => 1 - Math.pow(1 - p, 2),
};

export default function GradualBlur({
  position = "bottom",
  strength = 2,
  height = "6rem",
  divCount = 5,
  exponential = false,
  zIndex = 1000,
  opacity = 1,
  curve = "bezier",
  target = "parent",
  className = "",
  style,
  children,
}: GradualBlurProps) {
  const curveFunc = CURVE_FUNCTIONS[curve] || CURVE_FUNCTIONS.linear;
  const increment = 100 / divCount;

  const blurDivs = useMemo(() => {
    const divs: React.ReactNode[] = [];
    for (let i = 1; i <= divCount; i++) {
      let progress = i / divCount;
      progress = curveFunc(progress);

      let blurValue: number;
      if (exponential) {
        blurValue = Math.pow(2, progress * 4) * 0.0625 * strength;
      } else {
        blurValue = 0.0625 * (progress * divCount + 1) * strength;
      }

      const p1 = Math.round((increment * i - increment) * 10) / 10;
      const p2 = Math.round(increment * i * 10) / 10;
      const p3 = Math.round((increment * i + increment) * 10) / 10;
      const p4 = Math.round((increment * i + increment * 2) * 10) / 10;

      let gradient = `transparent ${p1}%, black ${p2}%`;
      if (p3 <= 100) gradient += `, black ${p3}%`;
      if (p4 <= 100) gradient += `, transparent ${p4}%`;

      const directions: Record<string, string> = { top: "to top", bottom: "to bottom", left: "to left", right: "to right" };
      const direction = directions[position] || "to bottom";

      divs.push(
        <div
          key={i}
          className="absolute inset-0"
          style={{
            maskImage: `linear-gradient(${direction}, ${gradient})`,
            WebkitMaskImage: `linear-gradient(${direction}, ${gradient})`,
            backdropFilter: `blur(${blurValue.toFixed(3)}rem)`,
            opacity,
          }}
        />
      );
    }
    return divs;
  }, [divCount, curveFunc, exponential, strength, increment, position, opacity]);

  const isPageTarget = target === "page";
  const isVertical = ["top", "bottom"].includes(position);

  const containerStyle: CSSProperties = {
    position: isPageTarget ? "fixed" : "absolute",
    pointerEvents: "none",
    zIndex: isPageTarget ? zIndex + 100 : zIndex,
    ...(isVertical ? { height, width: "100%", [position]: 0, left: 0, right: 0 } : { width: height, height: "100%", [position]: 0, top: 0, bottom: 0 }),
    ...style,
  };

  return (
    <div ref={undefined} className={`relative isolate ${className}`} style={containerStyle}>
      <div className="relative h-full w-full">{blurDivs}</div>
      {children && <div className="relative">{children}</div>}
    </div>
  );
}
