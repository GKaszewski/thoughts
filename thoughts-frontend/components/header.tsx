"use client";

import { useAuth } from "@/hooks/use-auth";
import Link from "next/link";
import { Button } from "./ui/button";
import { UserNav } from "./user-nav";
import { MainNav } from "./main-nav";

export function Header() {
  const { token } = useAuth();

  return (
    <header className="sticky top-0 z-50 flex justify-center w-full border-b border-primary/20 bg-background/80 glass-effect glossy-effect bottom rounded-none">
      <div className="container flex h-14 items-center px-2">
        <div className="flex gap-2">
          <Link href="/" className="flex items-center gap-1">
            <span className="hidden font-bold text-primary sm:inline-block">
              Thoughts
            </span>
          </Link>
          <MainNav />
        </div>
        <div className="flex flex-1 items-center justify-end space-x-2">
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
