"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import { buttonVariants } from "@/components/ui/button";

interface SettingsNavProps extends React.HTMLAttributes<HTMLElement> {
  items: {
    href: string;
    title: string;
  }[];
}

export function SettingsNav({ className, items, ...props }: SettingsNavProps) {
  const pathname = usePathname();

  return (
    <nav
      className={cn(
        "flex space-x-2 lg:flex-col lg:space-x-0 lg:space-y-1",
        className
      )}
      {...props}
    >
      {items.map((item) => (
        <Link
          key={item.href}
          href={item.href}
          className={cn(
            buttonVariants({ variant: "ghost" }),
            pathname === item.href ? "bg-muted" : "hover:underline",
            "justify-start glass-effect glossy-effect bottom shadow-fa-md"
          )}
        >
          {item.title}
        </Link>
      ))}
    </nav>
  );
}
