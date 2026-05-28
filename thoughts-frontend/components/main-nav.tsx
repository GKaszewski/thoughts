"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Menu } from "lucide-react";
import { cn } from "@/lib/utils";
import { SearchInput } from "./search-input";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";

interface MainNavProps {
  isLoggedIn?: boolean;
}

const NAV_LINKS = (isLoggedIn: boolean) => [
  { href: "/users/all", label: "Discover" },
  { href: "/about/fediverse", label: "Fediverse" },
  ...(isLoggedIn ? [{ href: "/friends", label: "Friends" }] : []),
];

export function MainNav({ isLoggedIn }: MainNavProps) {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);
  const links = NAV_LINKS(!!isLoggedIn);

  return (
    <>
      {/* Desktop nav */}
      <nav className="hidden md:flex items-center space-x-6 text-sm font-medium">
        {links.map(({ href, label }) => (
          <Link
            key={href}
            href={href}
            className={cn(
              "transition-colors hover:text-foreground/80",
              pathname === href ? "text-foreground" : "text-foreground/60"
            )}
          >
            {label}
          </Link>
        ))}
        <SearchInput />
      </nav>

      {/* Mobile hamburger */}
      <Sheet open={open} onOpenChange={setOpen}>
        <SheetTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="md:hidden"
            aria-label="Open menu"
          >
            <Menu className="h-5 w-5" />
          </Button>
        </SheetTrigger>
        <SheetContent side="left" className="w-72 glass-effect">
          <SheetHeader>
            <SheetTitle className="text-left">Menu</SheetTitle>
          </SheetHeader>
          <nav className="flex flex-col gap-1 mt-6">
            {links.map(({ href, label }) => (
              <Link
                key={href}
                href={href}
                onClick={() => setOpen(false)}
                className={cn(
                  "px-3 py-2 rounded-lg text-sm font-medium transition-colors hover:bg-accent",
                  pathname === href
                    ? "bg-accent text-foreground"
                    : "text-foreground/70"
                )}
              >
                {label}
              </Link>
            ))}
          </nav>
          <div className="mt-6">
            <SearchInput />
          </div>
        </SheetContent>
      </Sheet>
    </>
  );
}
