"use client";

import { useAuth } from "@/hooks/use-auth";
import Link from "next/link";
import { Button } from "./ui/button";
import { UserNav } from "./user-nav";
import { MainNav } from "./main-nav";
import { ThemeToggle } from "./theme-toggle";
import { Wind } from "lucide-react";

export function Header() {
  const { token } = useAuth();

  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="w-full flex h-14 items-center px-2">
        <div className="flex gap-2">
          <Link href="/" className="flex items-center gap-1">
            <span className="hidden font-bold sm:inline-block">Thoughts</span>
          </Link>
          <MainNav />
        </div>
        <div className="flex flex-1 items-center justify-end space-x-2">
          <ThemeToggle />
          {token ? (
            <UserNav />
          ) : (
            <>
              <Button asChild variant="ghost" size="sm">
                <Link href="/login">Login</Link>
              </Button>
              <Button asChild size="sm">
                <Link href="/register">Register</Link>
              </Button>
            </>
          )}
        </div>
      </div>
    </header>
  );
}
