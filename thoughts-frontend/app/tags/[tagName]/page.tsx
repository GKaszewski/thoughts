// app/tags/[tagName]/page.tsx
import { cookies } from "next/headers";
import { getThoughtsByTag, getUserProfile, getMe, Me, User } from "@/lib/api";
import { buildThoughtThreads } from "@/lib/utils";
import { ThoughtThread } from "@/components/thought-thread";
import { notFound } from "next/navigation";
import { Hash } from "lucide-react";

interface TagPageProps {
  params: { tagName: string };
}

export default async function TagPage({ params }: TagPageProps) {
  const { tagName } = params;
  const token = (await cookies()).get("auth_token")?.value ?? null;

  const [thoughtsResult, meResult] = await Promise.allSettled([
    getThoughtsByTag(tagName, token),
    token ? getMe(token) : Promise.resolve(null),
  ]);

  if (thoughtsResult.status === "rejected") {
    notFound();
  }

  const allThoughts = thoughtsResult.value.thoughts;
  const me = meResult.status === "fulfilled" ? (meResult.value as Me) : null;

  const authors = [...new Set(allThoughts.map((t) => t.authorUsername))];
  const userProfiles = await Promise.all(
    authors.map((username) => getUserProfile(username, token).catch(() => null))
  );
  const authorDetails = new Map<string, { avatarUrl?: string | null }>(
    userProfiles
      .filter((u): u is User => !!u)
      .map((user) => [user.username, { avatarUrl: user.avatarUrl }])
  );

  const { topLevelThoughts, repliesByParentId } =
    buildThoughtThreads(allThoughts);

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6">
        <h1 className="flex items-center gap-2 text-3xl font-bold">
          <Hash className="h-7 w-7" />
          {tagName}
        </h1>
      </header>
      <main className="space-y-6">
        {topLevelThoughts.map((thought) => (
          <ThoughtThread
            key={thought.id}
            thought={thought}
            repliesByParentId={repliesByParentId}
            authorDetails={authorDetails}
            currentUser={me}
          />
        ))}
        {topLevelThoughts.length === 0 && (
          <p className="text-center text-muted-foreground pt-8">
            No thoughts found for this tag.
          </p>
        )}
      </main>
    </div>
  );
}
