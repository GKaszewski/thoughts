"use client";

import { useAuth } from "@/hooks/use-auth";
import Image from "next/image";
import Link from "next/link";
import { Button } from "./ui/button";
import { UserNav } from "./user-nav";
import { MainNav } from "./main-nav";

export function Header() {
  const { token } = useAuth();

  return (
    <header className="sticky top-0 z-50 flex justify-center w-full border-b border-white/20 bg-background/80 glass-effect glossy-effect bottom rounded-none shadow-fa-md">
      <div className="container flex h-14 items-center px-2">
        {/* Logo */}
        <Link href="/" className="flex items-center gap-2 mr-4 shrink-0">
          <Image
            src="/icon.avif"
            alt="Thoughts"
            width={32}
            height={32}
            className="rounded-lg shadow-fa-sm"
          />
          <span className="hidden sm:inline-block font-bold text-primary text-shadow-sm">
            Thoughts
          </span>
        </Link>

        <MainNav />

        <div className="flex flex-1 items-center justify-end space-x-2">
          {token ? (
            <UserNav />
          ) : (
            <>
              <Button asChild size="sm" variant="outline" className="rounded-full">
                <Link href="/login">Login</Link>
              </Button>
              <Button asChild size="sm" className="rounded-full">
                <Link href="/register">Register</Link>
              </Button>
            </>
          )}
        </div>
      </div>
    </header>
  );
}
