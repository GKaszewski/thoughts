import Link from "next/link";

interface EmptyStateProps {
  emoji?: string;
  title?: string;
  message: string;
  ctaLabel?: string;
  ctaHref?: string;
  className?: string;
}

export function EmptyState({
  emoji = "💭",
  title,
  message,
  ctaLabel,
  ctaHref,
  className = "",
}: EmptyStateProps) {
  return (
    <div className={`flex flex-col items-center text-center py-10 gap-2 ${className}`}>
      <span className="text-4xl animate-float-bob select-none" aria-hidden="true">
        {emoji}
      </span>
      {title && (
        <p className="font-bold text-base text-foreground text-shadow-sm">{title}</p>
      )}
      <p className="text-sm text-muted-foreground max-w-xs leading-relaxed">{message}</p>
      {ctaLabel && ctaHref && (
        <Link
          href={ctaHref}
          className="mt-2 inline-flex items-center gap-1.5 px-5 py-2 rounded-full text-sm font-bold text-white fa-gradient-blue shadow-fa-md glossy-effect relative overflow-hidden"
        >
          {ctaLabel}
        </Link>
      )}
    </div>
  );
}
