"use client";

import { useState } from "react";
import { useAuth } from "@/hooks/use-auth";
import Link from "next/link";
import { followUser } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { UserAvatar } from "@/components/user-avatar";
import { toast } from "sonner";
import { UserPlus } from "lucide-react";
import { profileHref } from "@/lib/utils";

interface RemoteUserCardProps {
  actor: {
    handle: string;
    displayName: string | null;
    avatarUrl: string | null;
    url: string;
  };
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
        href={profileHref(actor.handle, false)}
        className="flex items-center gap-3 hover:opacity-80"
      >
        <UserAvatar src={actor.avatarUrl} alt={actor.displayName ?? actor.handle} />
        <div className="min-w-0">
          <p className="font-medium truncate">{actor.displayName ?? actor.handle}</p>
          <p className="text-sm text-muted-foreground truncate">{actor.handle}</p>
        </div>
      </Link>
      <div className="flex flex-col items-end gap-1">
        <Button
          onClick={handleFollow}
          disabled={loading || followed}
          variant={followed ? "secondary" : "default"}
          size="sm"
        >
          <UserPlus className="mr-2 h-4 w-4" />
          {followed ? "Requested" : "Follow"}
        </Button>
        {followed && (
          <p className="text-xs text-muted-foreground text-right">
            They&apos;ll be notified and can accept from their app.
          </p>
        )}
      </div>
    </div>
  );
}
