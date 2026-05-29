import Link from "next/link";
import {
  BookOpen,
  Code2,
  Fingerprint,
  Globe,
  Info,
} from "lucide-react";

const API_URL = process.env.NEXT_PUBLIC_API_URL ?? "";

const LINKS = [
  {
    href: `${API_URL}/docs`,
    label: "API Reference",
    icon: BookOpen,
    external: true,
  },
  {
    href: `${API_URL}/.well-known/nodeinfo`,
    label: "NodeInfo",
    icon: Info,
    external: true,
  },
  {
    href: `${API_URL}/.well-known/webfinger`,
    label: "WebFinger",
    icon: Fingerprint,
    external: true,
  },
  {
    href: "/about/fediverse",
    label: "About the Fediverse",
    icon: Globe,
    external: false,
  },
  {
    href: "https://git.gabrielkaszewski.dev/GKaszewski/thoughts",
    label: "Source Code",
    icon: Code2,
    external: true,
  },
] as const;

export function Footer() {
  return (
    <footer className="relative flex justify-center w-full border-t border-white/20 bg-background/80 backdrop-blur-lg shadow-fa-sm mt-auto">
      {/* Gloss highlight strip */}
      <div
        className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-white/40 to-transparent"
        aria-hidden
      />

      <div className="container flex flex-wrap items-center justify-center gap-x-1 gap-y-2 px-4 py-3">
        {LINKS.map(({ href, label, icon: Icon }, i) => (
          <span key={href} className="flex items-center gap-1">
            {i > 0 && (
              <span className="text-muted-foreground/40 select-none text-xs mx-1">
                ·
              </span>
            )}
            <Link
              href={href}
              {...(href.startsWith("http") || href.startsWith(API_URL)
                ? { target: "_blank", rel: "noopener noreferrer" }
                : {})}
              className="flex items-center gap-1 text-xs text-muted-foreground hover:text-primary transition-colors duration-150 group"
            >
              <Icon
                size={11}
                className="shrink-0 opacity-60 group-hover:opacity-100 transition-opacity"
              />
              <span>{label}</span>
            </Link>
          </span>
        ))}
      </div>
    </footer>
  );
}
