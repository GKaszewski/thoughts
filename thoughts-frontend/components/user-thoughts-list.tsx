"use client";

import { useState } from "react";
import { getUserThoughts, Me, Thought } from "@/lib/api";
import { ThoughtThread } from "@/components/thought-thread";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/empty-state";
import { buildThoughtThreads } from "@/lib/utils";
import { toast } from "sonner";
import { useAuth } from "@/hooks/use-auth";

interface UserThoughtsListProps {
  username: string;
  initialThoughts: Thought[];
  totalPages: number;
  me: Me | null;
}

export function UserThoughtsList({
  username,
  initialThoughts,
  totalPages,
  me,
}: UserThoughtsListProps) {
  const [thoughts, setThoughts] = useState<Thought[]>(initialThoughts);
  const [page, setPage] = useState(1);
  const [loadingMore, setLoadingMore] = useState(false);
  const { token } = useAuth();

  const thoughtThreads = buildThoughtThreads(thoughts);

  const loadMore = async () => {
    setLoadingMore(true);
    try {
      const result = await getUserThoughts(username, token, page + 1);
      setThoughts((prev) => [...prev, ...result.items]);
      setPage((p) => p + 1);
    } catch {
      toast.error("Failed to load more thoughts.");
    } finally {
      setLoadingMore(false);
    }
  };

  if (thoughtThreads.length === 0) {
    return (
      <EmptyState
        emoji="💭"
        title="Nothing here yet"
        message="This user hasn't posted any public thoughts yet."
      />
    );
  }

  return (
    <div className="space-y-4">
      {thoughtThreads.map((thought) => (
        <ThoughtThread key={thought.id} thought={thought} currentUser={me} />
      ))}
      {page < totalPages && (
        <Button
          onClick={loadMore}
          disabled={loadingMore}
          variant="outline"
          className="w-full rounded-full"
        >
          {loadingMore ? "Loading…" : "Load more"}
        </Button>
      )}
    </div>
  );
}
