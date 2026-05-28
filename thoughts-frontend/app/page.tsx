import type { Metadata } from "next";
import { cookies } from "next/headers";
import { getFeed, getMe, Me, FeedOptions, FeedSortOption } from "@/lib/api";
import { FiltersSortingPanel } from "@/components/filters-sorting-panel";
import { ThoughtForm } from "@/components/thought-form";
import { EmptyState } from "@/components/empty-state";
import { Button } from "@/components/ui/button";
import Link from "next/link";
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

function LandingPage() {
  return (
    <div className="font-sans min-h-screen flex items-center justify-center relative overflow-hidden">
      {/* Ambient orbs */}
      <div
        className="orb"
        style={{
          width: 280,
          height: 280,
          background:
            "radial-gradient(circle, #ffffff 0%, #87ceeb 60%, transparent 100%)",
          top: "-80px",
          left: "-60px",
        }}
      />
      <div
        className="orb"
        style={{
          width: 220,
          height: 220,
          background:
            "radial-gradient(circle, #b2f5ea 0%, #48bb78 60%, transparent 100%)",
          bottom: "-40px",
          right: "5%",
        }}
      />
      <div
        className="orb"
        style={{
          width: 160,
          height: 160,
          background:
            "radial-gradient(circle, #e0f2fe 0%, #38bdf8 60%, transparent 100%)",
          top: "35%",
          left: "65%",
        }}
      />

      {/* Hero card */}
      <div
        className="container mx-auto max-w-lg p-4 sm:p-6 text-center relative z-10"
        style={{
          background: "rgba(255,255,255,0.28)",
          backdropFilter: "blur(20px)",
          WebkitBackdropFilter: "blur(20px)",
          border: "1px solid rgba(255,255,255,0.55)",
          borderRadius: "20px",
          boxShadow:
            "0 8px 32px rgba(0,0,0,0.10), inset 0 1px 0 rgba(255,255,255,0.6)",
        }}
      >
        {/* Gloss sweep */}
        <div
          aria-hidden
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            right: 0,
            height: "55%",
            background:
              "linear-gradient(180deg, rgba(255,255,255,0.38) 0%, transparent 100%)",
            borderRadius: "20px 20px 0 0",
            pointerEvents: "none",
          }}
        />

        <h1
          className="text-5xl font-bold relative"
          style={{
            textShadow:
              "0 2px 4px rgba(255,255,255,0.6), 0 1px 2px rgba(0,0,0,0.1)",
          }}
        >
          Welcome to Thoughts
        </h1>
        <p className="text-muted-foreground mt-3 relative">
          A federated social network for short-form thoughts.
          <br />
          Connect with the Fediverse.
        </p>

        <div className="mt-8 flex justify-center gap-4 relative">
          <Button asChild className="px-7">
            <Link href="/login">Login</Link>
          </Button>
          <Button asChild variant="secondary" className="px-7">
            <Link href="/register">Register</Link>
          </Button>
        </div>

        {/* Fediverse badge */}
        <div className="mt-5 relative flex justify-center">
          <span
            className="inline-flex items-center gap-2 px-4 py-1.5 rounded-full text-xs text-muted-foreground"
            style={{
              background: "rgba(255,255,255,0.3)",
              border: "1px solid rgba(255,255,255,0.5)",
            }}
          >
            <span
              className="w-2 h-2 rounded-full bg-emerald-400 inline-block"
              style={{ boxShadow: "0 0 4px #34d399" }}
            />
            Works with Mastodon, Pixelfed &amp; more
          </span>
        </div>
      </div>
    </div>
  );
}
