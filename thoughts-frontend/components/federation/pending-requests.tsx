"use client";

import { useEffect, useState } from "react";
import {
  getPendingFollowRequests,
  acceptFollowRequest,
  rejectFollowRequest,
  type RemoteActor,
} from "@/lib/api";
import { useAuth } from "@/hooks/use-auth";
import { UserAvatar } from "@/components/user-avatar";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import Link from "next/link";
import { fullFediverseHandle } from "@/lib/utils";

interface Props {
  compact?: boolean;
}

export function PendingRequests({ compact = false }: Props) {
  const { token } = useAuth();
  const [requests, setRequests] = useState<RemoteActor[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!token) return;
    getPendingFollowRequests(token)
      .then(setRequests)
      .catch(() => toast.error("Failed to load follow requests"))
      .finally(() => setLoading(false));
  }, [token]);

  const accept = async (actorUrl: string) => {
    if (!token) return;
    setRequests((prev) => prev.filter((r) => r.url !== actorUrl));
    await acceptFollowRequest(actorUrl, token).catch(() => {
      toast.error("Failed to accept follow request");
    });
  };

  const reject = async (actorUrl: string) => {
    if (!token) return;
    setRequests((prev) => prev.filter((r) => r.url !== actorUrl));
    await rejectFollowRequest(actorUrl, token).catch(() => {
      toast.error("Failed to reject follow request");
    });
  };

  if (loading) return <p className="text-sm text-muted-foreground">Loading…</p>;
  if (requests.length === 0)
    return <p className="text-sm text-muted-foreground">No pending requests.</p>;

  return (
    <ul className={compact ? "space-y-2" : "space-y-3"}>
      {requests.map((actor) => (
        <li
          key={actor.url}
          className="flex items-center justify-between gap-3"
        >
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
          <div className="flex gap-2 shrink-0">
            <Button size="sm" onClick={() => accept(actor.url)}>
              Accept
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => reject(actor.url)}
            >
              Reject
            </Button>
          </div>
        </li>
      ))}
    </ul>
  );
}
