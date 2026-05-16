"use client";

import { useState } from "react";
import { useAuth } from "@/hooks/use-auth";
import Link from "next/link";
import { followUser } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { UserAvatar } from "@/components/user-avatar";
import { toast } from "sonner";
import { UserPlus } from "lucide-react";

interface RemoteUserCardProps {
  actor: {
    handle: string;
    displayName: string | null;
    avatarUrl: string | null;
    url: string;
  };
}

function resolveProfileHref(handle: string): string {
  const apiDomain = process.env.NEXT_PUBLIC_API_URL
    ? new URL(process.env.NEXT_PUBLIC_API_URL).hostname
    : null;
  const clean = handle.startsWith("@") ? handle.slice(1) : handle;
  const atIdx = clean.indexOf("@");
  const domain = atIdx !== -1 ? clean.slice(atIdx + 1) : null;
  const username = atIdx !== -1 ? clean.slice(0, atIdx) : clean;
  return apiDomain && domain === apiDomain
    ? `/users/${username}`
    : `/remote-actor?handle=@${clean}`;
}

export function RemoteUserCard({ actor }: RemoteUserCardProps) {
  const [followed, setFollowed] = useState(false);
  const [loading, setLoading] = useState(false);
  const { token } = useAuth();

  const handleFollow = async () => {
    if (!token) {
      toast.error("You must be logged in to follow users.");
      return;
    }
    setLoading(true);
    try {
      await followUser(actor.handle, token);
      setFollowed(true);
      toast.success(`Follow request sent to ${actor.handle}`);
    } catch {
      toast.error("Failed to send follow request.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex items-center justify-between p-4 border rounded-lg">
      <Link
        href={resolveProfileHref(actor.handle)}
        className="flex items-center gap-3 hover:opacity-80"
      >
        <UserAvatar src={actor.avatarUrl} alt={actor.displayName ?? actor.handle} />
        <div className="min-w-0">
          <p className="font-medium truncate">{actor.displayName ?? actor.handle}</p>
          <p className="text-sm text-muted-foreground truncate">{actor.handle}</p>
        </div>
      </Link>
      <Button
        onClick={handleFollow}
        disabled={loading || followed}
        variant={followed ? "secondary" : "default"}
        size="sm"
      >
        <UserPlus className="mr-2 h-4 w-4" />
        {followed ? "Requested" : "Follow"}
      </Button>
    </div>
  );
}
