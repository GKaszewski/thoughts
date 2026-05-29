import Link from "next/link";
import { ExternalLink } from "lucide-react";
import { ReactNode } from "react";
import { RemoteActor } from "@/lib/api";
import { UserAvatar } from "@/components/user-avatar";
import { Button } from "@/components/ui/button";

interface ProfileCardProps {
  actor: RemoteActor;
  /** Slot rendered next to the avatar (e.g. follow/unfollow button). */
  action?: ReactNode;
}

export function ProfileCard({ actor, action }: ProfileCardProps) {
  let hostname: string | null = null;
  try {
    if (actor.url) hostname = new URL(actor.url).hostname;
  } catch {
    hostname = actor.url;
  }

  return (
    <>
      <div className="flex justify-between items-start">
        <div className="w-24 h-24 rounded-full border-4 border-background shrink-0">
          <UserAvatar
            src={actor.avatarUrl}
            alt={actor.displayName}
            className="w-full h-full"
          />
        </div>
        {action}
      </div>

      <div className="mt-4 min-w-0">
        <h1 className="text-2xl font-bold truncate">
          {actor.displayName ?? actor.handle}
        </h1>
        <p className="text-sm text-muted-foreground truncate">{actor.handle}</p>
      </div>

      {actor.bio && (
        <div
          className="mt-4 text-sm [&_a]:underline [&_a]:text-primary [&_p]:mb-2"
          dangerouslySetInnerHTML={{ __html: actor.bio }}
        />
      )}

      <Button asChild variant="outline" size="sm" className="mt-4 w-full">
        <Link
          href={actor.url}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center overflow-hidden"
        >
          <ExternalLink className="mr-2 h-4 w-4 shrink-0" />
          <span className="truncate">{hostname}</span>
        </Link>
      </Button>

      {actor.alsoKnownAs.length > 0 && (
        <p className="mt-2 text-xs text-muted-foreground">
          Also known as:{" "}
          {actor.alsoKnownAs.map((aka, i) => (
            <span key={aka}>
              {i > 0 && ", "}
              <Link
                href={aka}
                target="_blank"
                rel="noopener noreferrer"
                className="underline"
              >
                {aka}
              </Link>
            </span>
          ))}
        </p>
      )}

      {actor.attachment.length > 0 && (
        <div className="mt-4 space-y-0 text-sm">
          {actor.attachment.map((field) => (
            <div
              key={field.name}
              className="grid grid-cols-[minmax(0,5rem)_1fr] gap-2 border-t py-1"
            >
              <span
                className="font-medium text-muted-foreground truncate"
                title={field.name}
              >
                {field.name}
              </span>
              <span
                className="break-all min-w-0 [&_a]:underline [&_a]:text-primary"
                dangerouslySetInnerHTML={{ __html: field.value }}
              />
            </div>
          ))}
        </div>
      )}
    </>
  );
}
