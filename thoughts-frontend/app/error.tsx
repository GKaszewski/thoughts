"use client";

import { useEffect } from "react";
import { Button } from "@/components/ui/button";

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error(error);
  }, [error]);

  return (
    <div className="min-h-[80vh] flex items-center justify-center px-4">
      <div className="glass-effect glossy-effect bottom rounded-2xl shadow-fa-lg p-10 text-center max-w-sm w-full">
        <p className="text-6xl font-black text-muted-foreground/30 mb-4">Oops</p>
        <h1 className="text-xl font-bold mb-2">Something went wrong</h1>
        <p className="text-sm text-muted-foreground mb-8">
          An unexpected error occurred. Try again or come back later.
        </p>
        <Button onClick={reset} className="rounded-full">
          Try again
        </Button>
      </div>
    </div>
  );
}
