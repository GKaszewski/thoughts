import Link from "next/link";
import { CopyButton } from "./copy-button";

interface FediverseHandleWidgetProps {
  username: string;
}

export function FediverseHandleWidget({ username }: FediverseHandleWidgetProps) {
  const domain = process.env.NEXT_PUBLIC_FEDIVERSE_DOMAIN;
  if (!domain) return null;

  const handle = `@${username}@${domain}`;

  return (
    <div className="glass-effect rounded-md shadow-fa-lg p-4">
      <h2 className="text-sm font-semibold mb-2">Your fediverse handle</h2>
      <div className="flex items-center gap-1 mb-2">
        <p className="text-xs font-mono text-muted-foreground break-all">{handle}</p>
        <CopyButton text={handle} />
      </div>
      <p className="text-xs text-muted-foreground mb-2">
        Anyone on Mastodon or Pixelfed can follow you with this.
      </p>
      <Link
        href="/about/fediverse"
        className="text-xs text-primary hover:underline"
      >
        Learn more →
      </Link>
    </div>
  );
}
