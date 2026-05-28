import type { Metadata } from "next";
import { cookies } from "next/headers";
import { getFeed, getMe, Me, FeedOptions, FeedSortOption } from "@/lib/api";
import { FiltersSortingPanel } from "@/components/filters-sorting-panel";
import { ThoughtForm } from "@/components/thought-form";
import { EmptyState } from "@/components/empty-state";
import { LandingPage } from "@/components/landing-page";
import { PopularTags } from "@/components/popular-tags";
import { ThoughtThread } from "@/components/thought-thread";
import { buildThoughtThreads } from "@/lib/utils";
import { TopFriends } from "@/components/top-friends";
import { UsersCount } from "@/components/users-count";
import { PaginationNav } from "@/components/pagination-nav";
import { redirect } from "next/navigation";
import { Suspense } from "react";
import {
  ProfileSkeleton,
  TagsSkeleton,
  CountSkeleton,
} from "@/components/loading-skeleton";

export const metadata: Metadata = {
  title: "Home",
  description: "Your home timeline — thoughts from people you follow",
};

export default async function Home({
  searchParams,
}: {
  searchParams: Promise<{
    page?: string;
    sort?: string;
    originals_only?: string;
    replies_only?: string;
    local_only?: string;
    hide_sensitive?: string;
  }>;
}) {
  const token = (await cookies()).get("auth_token")?.value ?? null;
  const resolvedSearchParams = await searchParams;

  if (token) {
    return <FeedPage token={token} searchParams={resolvedSearchParams} />;
  } else {
    return <LandingPage />;
  }
}

async function FeedPage({
  token,
  searchParams,
}: {
  token: string;
  searchParams: {
    page?: string;
    sort?: string;
    originals_only?: string;
    replies_only?: string;
    local_only?: string;
    hide_sensitive?: string;
  };
}) {
  const page = parseInt(searchParams.page ?? "1", 10);

  const feedOpts: FeedOptions = {
    sort: searchParams.sort as FeedSortOption | undefined,
    originals_only: searchParams.originals_only === "true",
    replies_only:   searchParams.replies_only   === "true",
    local_only:     searchParams.local_only     === "true",
    hide_sensitive: searchParams.hide_sensitive === "true",
  };

  const [feedData, me] = await Promise.all([
    getFeed(token, page, 20, feedOpts).catch(() => null),
    getMe(token).catch(() => null) as Promise<Me | null>,
  ]);

  if (!feedData || !me) {
    redirect("/login");
  }

  const { items: allThoughts, totalPages } = feedData!;
  const thoughtThreads = buildThoughtThreads(allThoughts);

  const sidebar = (
    <>
      <Suspense fallback={<ProfileSkeleton />}>
        <TopFriends username={me.username} />
      </Suspense>
      <Suspense fallback={<TagsSkeleton />}>
        <PopularTags />
      </Suspense>
      <Suspense fallback={<CountSkeleton />}>
        <UsersCount />
      </Suspense>
    </>
  );

  return (
    <div className="container mx-auto max-w-6xl p-4 sm:p-6">
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
        <aside className="hidden lg:block lg:col-span-1">
          <div className="sticky top-20 space-y-6 glass-effect glossy-effect bottom rounded-md p-4">
            <h2 className="text-lg font-semibold">Filters &amp; Sorting</h2>
            <Suspense>
              <FiltersSortingPanel />
            </Suspense>
          </div>
        </aside>

        <main className="col-span-1 lg:col-span-2 space-y-6">
          <header className="mb-6">
            <h1 className="text-3xl font-bold text-shadow-sm">Your Feed</h1>
          </header>
          <ThoughtForm />

          <div className="block lg:hidden space-y-6">{sidebar}</div>

          <div className="space-y-6">
            {thoughtThreads.map((thought) => (
              <ThoughtThread
                key={thought.id}
                thought={thought}
                currentUser={me}
              />
            ))}
            {thoughtThreads.length === 0 && (
              <EmptyState
                emoji="💭"
                title="Your feed is quiet"
                message="Your feed is empty. Follow some users to see their thoughts!"
                ctaLabel="Discover people ✨"
                ctaHref="/users/all"
              />
            )}
          </div>
          <PaginationNav
            page={page}
            totalPages={totalPages}
            buildHref={(p) => `/?page=${p}`}
          />
        </main>

        <aside className="hidden lg:block lg:col-span-1">
          <div className="sticky top-20 space-y-6">{sidebar}</div>
        </aside>
      </div>
    </div>
  );
}

