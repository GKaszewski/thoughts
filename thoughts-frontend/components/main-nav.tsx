"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import { SearchInput } from "./search-input";

export function MainNav() {
  const pathname = usePathname();
  return (
    <nav className="hidden md:flex items-center space-x-6 text-sm font-medium">
      <Link
        href="/"
        className={cn(
          "transition-colors hover:text-foreground/80",
          pathname === "/" ? "text-foreground" : "text-foreground/60"
        )}
      >
        Feed
      </Link>
      <SearchInput />
    </nav>
  );
}
