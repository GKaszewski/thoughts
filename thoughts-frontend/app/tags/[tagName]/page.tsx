// app/tags/[tagName]/page.tsx
import type { Metadata } from "next";
import { cookies } from "next/headers";
import { getThoughtsByTag, getMe, Me } from "@/lib/api";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ tagName: string }>;
}): Promise<Metadata> {
  const { tagName } = await params;
  return {
    title: `#${tagName}`,
    description: `Thoughts tagged with #${tagName}`,
    openGraph: {
      title: `#${tagName} · Thoughts`,
      description: `Thoughts tagged with #${tagName}`,
    },
    twitter: {
      card: "summary",
      title: `#${tagName} · Thoughts`,
      description: `Thoughts tagged with #${tagName}`,
    },
  };
}
import { EmptyState } from "@/components/empty-state";
import { buildThoughtThreads } from "@/lib/utils";
import { ThoughtThread } from "@/components/thought-thread";
import { notFound } from "next/navigation";
import { Hash } from "lucide-react";

interface TagPageProps {
  params: Promise<{ tagName: string }>;
}

export default async function TagPage({ params }: TagPageProps) {
  const { tagName } = await params;
  const token = (await cookies()).get("auth_token")?.value ?? null;

  const [thoughtsResult, meResult] = await Promise.allSettled([
    getThoughtsByTag(tagName, token),
    token ? getMe(token) : Promise.resolve(null),
  ]);

  if (thoughtsResult.status === "rejected") {
    notFound();
  }

  const allThoughts = thoughtsResult.value.items;
  const thoughtThreads = buildThoughtThreads(allThoughts);
  const me = meResult.status === "fulfilled" ? (meResult.value as Me) : null;

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6">
        <h1 className="flex items-center gap-2 text-3xl font-bold">
          <Hash className="h-7 w-7" />
          {tagName}
        </h1>
      </header>
      <main className="space-y-6">
        {thoughtThreads.map((thought) => (
          <ThoughtThread
            key={thought.id}
            thought={thought}
            currentUser={me}
          />
        ))}
        {thoughtThreads.length === 0 && (
          <EmptyState emoji="🏷" title="No thoughts here yet" message="No thoughts found for this tag." />
        )}
      </main>
    </div>
  );
}
