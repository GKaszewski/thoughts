// app/(auth)/layout.tsx
import type { Metadata } from "next";

export const metadata: Metadata = {
  openGraph: { type: "website" },
};

export default function AuthLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-center min-h-screen">
      {children}
    </div>
  );
}
