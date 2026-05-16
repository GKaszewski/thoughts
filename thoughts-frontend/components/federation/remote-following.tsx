"use client";

import { useEffect, useState } from "react";
import { getRemoteFollowing, unfollowRemoteActor, type RemoteActor } from "@/lib/api";
import { useAuth } from "@/hooks/use-auth";
import { UserAvatar } from "@/components/user-avatar";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import Link from "next/link";
import { fullFediverseHandle } from "@/lib/utils";

export function RemoteFollowing() {
  const { token } = useAuth();
  const [following, setFollowing] = useState<RemoteActor[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!token) return;
    getRemoteFollowing(token)
      .then(setFollowing)
      .catch(() => toast.error("Failed to load following"))
      .finally(() => setLoading(false));
  }, [token]);

  const unfollow = async (actor: RemoteActor) => {
    if (!token) return;
    const handle = fullFediverseHandle(actor.handle, actor.url);
    setFollowing((prev) => prev.filter((f) => f.url !== actor.url));
    await unfollowRemoteActor(handle, token).catch(() => {
      toast.error("Failed to unfollow");
    });
  };

  if (loading) return <p className="text-sm text-muted-foreground">Loading…</p>;
  if (following.length === 0)
    return <p className="text-sm text-muted-foreground">Not following anyone remotely yet.</p>;

  return (
    <ul className="space-y-3">
      {following.map((actor) => (
        <li key={actor.url} className="flex items-center justify-between gap-3">
          <Link
            href={`/users/@${fullFediverseHandle(actor.handle, actor.url)}`}
            className="flex items-center gap-2 min-w-0 hover:opacity-80"
          >
            <UserAvatar
              src={actor.avatarUrl}
              alt={actor.displayName}
              className="h-8 w-8 shrink-0"
            />
            <div className="min-w-0">
              <p className="text-sm font-medium truncate">
                {actor.displayName || actor.handle}
              </p>
              <p className="text-xs text-muted-foreground truncate font-mono">
                @{fullFediverseHandle(actor.handle, actor.url)}
              </p>
            </div>
          </Link>
          <Button
            size="sm"
            variant="outline"
            onClick={() => unfollow(actor)}
          >
            Unfollow
          </Button>
        </li>
      ))}
    </ul>
  );
}
