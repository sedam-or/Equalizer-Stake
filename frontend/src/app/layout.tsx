import type { Metadata, Viewport } from "next";
import { Inter, Geist } from "next/font/google";
import "./globals.css";
import { cn } from "@/lib/utils";

const geist = Geist({subsets:['latin'],variable:'--font-sans'});

const inter = Inter({ subsets: ["latin"] });

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  maximumScale: 1,
  themeColor: "#000000",
};

export const metadata: Metadata = {
  title: "BMD Music School — Sistem Penjadwalan Kursus Musik",
  description: "Sistem Penjadwalan Kursus & Reservasi Studio BMD Musik Yogyakarta",
  manifest: "/manifest.json",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="id" className={cn("font-sans", geist.variable)}>
      <body className={inter.className}>
        <a href="#main-content" className="sr-only focus:not-sr-only focus:absolute focus:z-[100] focus:p-4 focus:bg-white focus:text-gray-900 focus:font-semibold focus:outline-none focus:ring-2 focus:ring-[#08CB00]">
          Lewati ke konten utama
        </a>
        {children}
      </body>
    </html>
  );
}
