"use client";

import { useState, useEffect } from "react";
import { ActorConnection, getActorFollowers, getActorFollowing } from "@/lib/api";
import { Card } from "@/components/ui/card";
import { RemoteUserCard } from "@/components/remote-user-card";

interface ConnectionsProps {
  handle: string;
  token: string | null;
  type: "followers" | "following";
  /** Parent sets this to true when the tab becomes active for the first time. */
  active: boolean;
}

export function Connections({ handle, token, type, active }: ConnectionsProps) {
  const [items, setItems] = useState<ActorConnection[]>([]);
  const [page, setPage] = useState(1);
  const [hasMore, setHasMore] = useState(false);
  const [loaded, setLoaded] = useState(false);

  const load = async (p: number) => {
    const fetchFn = type === "followers" ? getActorFollowers : getActorFollowing;
    const result = await fetchFn(handle, p, token).catch(() => null);
    if (!result) return;
    setItems((prev) => (p === 1 ? result.items : [...prev, ...result.items]));
    setHasMore(result.hasMore);
    setLoaded(true);
    setPage(p);
  };

  useEffect(() => {
    if (active && !loaded) {
      load(1);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [active]);

  const emptyMessage =
    type === "followers"
      ? "No followers cached yet — check back soon."
      : "No following cached yet — check back soon.";

  if (!loaded) {
    return (
      <Card className="flex items-center justify-center h-48">
        <p className="text-center text-muted-foreground">Loading {type}…</p>
      </Card>
    );
  }

  if (items.length === 0) {
    return (
      <Card className="flex items-center justify-center h-48">
        <p className="text-center text-muted-foreground">{emptyMessage}</p>
      </Card>
    );
  }

  return (
    <div className="space-y-2">
      {items.map((f) => (
        <RemoteUserCard key={f.url} actor={f} />
      ))}
      {hasMore && (
        <button
          onClick={() => load(page + 1)}
          className="w-full text-sm text-muted-foreground hover:text-foreground py-2"
        >
          Load more
        </button>
      )}
    </div>
  );
}
