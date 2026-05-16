"use client";

import { useEffect, useState } from "react";
import { getRemoteFollowers, rejectFollowRequest, type RemoteActor } from "@/lib/api";
import { useAuth } from "@/hooks/use-auth";
import { UserAvatar } from "@/components/user-avatar";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import Link from "next/link";
import { fullFediverseHandle } from "@/lib/utils";

export function RemoteFollowers() {
  const { token } = useAuth();
  const [followers, setFollowers] = useState<RemoteActor[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!token) return;
    getRemoteFollowers(token)
      .then(setFollowers)
      .catch(() => toast.error("Failed to load followers"))
      .finally(() => setLoading(false));
  }, [token]);

  const remove = async (actorUrl: string) => {
    if (!token) return;
    setFollowers((prev) => prev.filter((f) => f.url !== actorUrl));
    await rejectFollowRequest(actorUrl, token).catch(() => {
      toast.error("Failed to remove follower");
    });
  };

  if (loading) return <p className="text-sm text-muted-foreground">Loading…</p>;
  if (followers.length === 0)
    return <p className="text-sm text-muted-foreground">No remote followers yet.</p>;

  return (
    <ul className="space-y-3">
      {followers.map((actor) => (
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
          <Button size="sm" variant="outline" onClick={() => remove(actor.url)}>
            Remove
          </Button>
        </li>
      ))}
    </ul>
  );
}
