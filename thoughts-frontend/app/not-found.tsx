import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function NotFound() {
  return (
    <div className="min-h-[80vh] flex items-center justify-center px-4">
      <div className="glass-effect glossy-effect bottom rounded-2xl shadow-fa-lg p-10 text-center max-w-sm w-full">
        <p className="text-6xl font-black text-muted-foreground/30 mb-4">404</p>
        <h1 className="text-xl font-bold mb-2">Page not found</h1>
        <p className="text-sm text-muted-foreground mb-8">
          This page doesn&apos;t exist or was moved.
        </p>
        <Button asChild className="rounded-full">
          <Link href="/">Go home</Link>
        </Button>
      </div>
    </div>
  );
}
