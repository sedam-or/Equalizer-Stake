import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Equalizer-Stake — BMD Music School",
  description: "Sistem Penjadwalan Kursus & Reservasi Studio BMD Musik Yogyakarta",
  manifest: "/manifest.json",
  themeColor: "#1E40AF",
  viewport: "width=device-width, initial-scale=1, maximum-scale=1",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="id">
      <body>{children}</body>
    </html>
  );
}
