import { cookies } from "next/headers";
import {
  getFeed,
  getFriends,
  getMe,
  getUserProfile,
  Me,
  User,
} from "@/lib/api";
import { PostThoughtForm } from "@/components/post-thought-form";
import { Button } from "@/components/ui/button";
import Link from "next/link";
import { PopularTags } from "@/components/popular-tags";
import { ThoughtThread } from "@/components/thought-thread";
import { buildThoughtThreads } from "@/lib/utils";
import { TopFriends } from "@/components/top-friends";
import { UsersCount } from "@/components/users-count";

import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationNext,
  PaginationPrevious,
} from "@/components/ui/pagination";
import { redirect } from "next/navigation";

export default async function Home({
  searchParams,
}: {
  searchParams: { page?: string };
}) {
  const token = (await cookies()).get("auth_token")?.value ?? null;

  if (token) {
    return <FeedPage token={token} searchParams={searchParams} />;
  } else {
    return <LandingPage />;
  }
}

async function FeedPage({
  token,
  searchParams,
}: {
  token: string;
  searchParams: { page?: string };
}) {
  const page = parseInt(searchParams.page ?? "1", 10);

  const [feedData, me] = await Promise.all([
    getFeed(token, page).catch(() => null),
    getMe(token).catch(() => null) as Promise<Me | null>,
  ]);

  if (!feedData || !me) {
    redirect("/login");
  }

  const { items: allThoughts, totalPages } = feedData!;
  const thoughtThreads = buildThoughtThreads(allThoughts);

  const authors = [...new Set(allThoughts.map((t) => t.authorUsername))];
  const userProfiles = await Promise.all(
    authors.map((username) => getUserProfile(username, token).catch(() => null))
  );

  const authorDetails = new Map<string, { avatarUrl?: string | null }>(
    userProfiles
      .filter((u): u is User => !!u)
      .map((user) => [user.username, { avatarUrl: user.avatarUrl }])
  );

  const friends = (await getFriends(token)).users.map((user) => user.username);
  const shouldDisplayTopFriends =
    token && me?.topFriends && me.topFriends.length > 8;

  console.log("Should display top friends:", shouldDisplayTopFriends);

  return (
    <div className="container mx-auto max-w-6xl p-4 sm:p-6">
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
        <aside className="hidden lg:block lg:col-span-1">
          <div className="sticky top-20 space-y-6 glass-effect glossy-effect bottom rounded-md p-4">
            <h2 className="text-lg font-semibold">Filters & Sorting</h2>
            <p className="text-sm text-muted-foreground">Coming soon...</p>
          </div>
        </aside>

        <main className="col-span-1 lg:col-span-2 space-y-6">
          <header className="mb-6">
            <h1 className="text-3xl font-bold text-shadow-sm">Your Feed</h1>
          </header>
          <PostThoughtForm />

          <div className="block lg:hidden space-y-6">
            <PopularTags />
            {shouldDisplayTopFriends && (
              <TopFriends mode="top-friends" usernames={me.topFriends} />
            )}
            {!shouldDisplayTopFriends && token && friends.length > 0 && (
              <TopFriends mode="friends" usernames={friends || []} />
            )}
            <UsersCount />
          </div>

          <div className="space-y-6">
            {thoughtThreads.map((thought) => (
              <ThoughtThread
                key={thought.id}
                thought={thought}
                authorDetails={authorDetails}
                currentUser={me}
              />
            ))}
            {thoughtThreads.length === 0 && (
              <p className="text-center text-muted-foreground pt-8">
                Your feed is empty. Follow some users to see their thoughts!
              </p>
            )}
          </div>
          <Pagination className="mt-8">
            <PaginationContent>
              <PaginationItem>
                <PaginationPrevious
                  href={page > 1 ? `/?page=${page - 1}` : "#"}
                  aria-disabled={page <= 1}
                />
              </PaginationItem>
              <PaginationItem>
                <PaginationNext
                  href={page < totalPages ? `/?page=${page + 1}` : "#"}
                  aria-disabled={page >= totalPages}
                />
              </PaginationItem>
            </PaginationContent>
          </Pagination>
        </main>

        <aside className="hidden lg:block lg:col-span-1">
          <div className="sticky top-20 space-y-6">
            <PopularTags />
            {shouldDisplayTopFriends && (
              <TopFriends mode="top-friends" usernames={me.topFriends} />
            )}
            {!shouldDisplayTopFriends && token && friends.length > 0 && (
              <TopFriends mode="friends" usernames={friends || []} />
            )}
            <UsersCount />
          </div>
        </aside>
      </div>
    </div>
  );
}

function LandingPage() {
  return (
    <>
      <div className="font-sans min-h-screen text-gray-800 flex items-center justify-center">
        <div className="container mx-auto max-w-2xl p-4 sm:p-6 text-center glass-effect glossy-effect bottom rounded-md shadow-fa-lg">
          <h1
            className="text-5xl font-bold"
            style={{ textShadow: "2px 2px 4px rgba(0,0,0,0.1)" }}
          >
            Welcome to Thoughts
          </h1>
          <p className="text-muted-foreground mt-2">
            Throwback to the golden age of microblogging.
          </p>
          <div className="mt-8 flex justify-center gap-4">
            <Button asChild>
              <Link href="/login">Login</Link>
            </Button>
            <Button variant="secondary" asChild>
              <Link href="/register">Register</Link>
            </Button>
          </div>
        </div>
      </div>
    </>
  );
}
