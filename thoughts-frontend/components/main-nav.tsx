"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import { SearchInput } from "./search-input";

export function MainNav() {
  const pathname = usePathname();
  return (
    <nav className="inline-flex md:flex items-center space-x-6 text-sm font-medium">
      <Link
        href="/users/all"
        className={cn(
          "transition-colors hover:text-foreground/80",
          pathname === "/users/all" ? "text-foreground" : "text-foreground/60"
        )}
      >
        Discover
      </Link>
      <SearchInput />
    </nav>
  );
}
