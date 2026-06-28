import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "CogniPhish — Platform Penelitian Akademik",
  description:
    "Platform penelitian tentang bagaimana manusia memproses informasi dalam situasi ambigu",
  keywords: ["penelitian", "akademik", "pengambilan keputusan", "informasi digital"],
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="id">
      <body className={inter.className}>
        <a href="#main-content" className="sr-only focus:not-sr-only focus:absolute focus:z-[100] focus:p-4 focus:bg-white focus:text-gray-900 focus:font-semibold focus:outline-none focus:ring-2 focus:ring-blue-600">
          Lewati ke konten utama
        </a>
        {children}
      </body>
    </html>
  );
}
