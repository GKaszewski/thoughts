import type { Metadata } from "next";
import "./globals.css";
import { AuthProvider } from "@/hooks/use-auth";
import { Toaster } from "@/components/ui/sonner";
import { Header } from "@/components/header";
import localFont from "next/font/local";
import InstallPrompt from "@/components/install-prompt";

export const metadata: Metadata = {
  title: "Thoughts",
  description: "A social network for sharing thoughts",
};

const frutiger = localFont({
  src: [
    {
      path: "./frutiger.woff",
      weight: "normal",
      style: "normal",
    },
    {
      path: "./frutiger-bold.woff",
      weight: "bold",
      style: "normal",
    },
  ],
  variable: "--font-frutiger",
});

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${frutiger.className} antialiased`}>
        <AuthProvider>
          <Header />
          <main className="flex-1">{children}</main>
          <InstallPrompt />
          <Toaster />
        </AuthProvider>
      </body>
    </html>
  );
}
