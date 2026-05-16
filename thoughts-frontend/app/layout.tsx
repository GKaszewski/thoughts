import type { Metadata } from "next";
import "./globals.css";
import { AuthProvider } from "@/hooks/use-auth";
import { Toaster } from "@/components/ui/sonner";
import { Header } from "@/components/header";
import localFont from "next/font/local";
import Image from "next/image";
import InstallPrompt from "@/components/install-prompt";

export const metadata: Metadata = {
  title: {
    default: "Thoughts",
    template: "%s · Thoughts",
  },
  description:
    "A federated social network for short-form thoughts. Follow people across Mastodon, Pixelfed, and the wider Fediverse.",
  openGraph: {
    type: "website",
    siteName: "Thoughts",
    title: "Thoughts",
    description:
      "A federated social network for short-form thoughts. Follow people across the Fediverse.",
  },
  twitter: {
    card: "summary",
    title: "Thoughts",
    description:
      "A federated social network for short-form thoughts. Follow people across the Fediverse.",
  },
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
        <Image
          src="/bg1.avif"
          alt=""
          fill
          priority
          quality={85}
          className="fixed inset-0 -z-10 object-cover object-center"
        />
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
