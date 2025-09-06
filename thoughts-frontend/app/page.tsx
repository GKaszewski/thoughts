import { cookies } from "next/headers";
import { getFeed, getMe, getUserProfile, Me, User } from "@/lib/api";
import { PostThoughtForm } from "@/components/post-thought-form";
import { Button } from "@/components/ui/button";
import Link from "next/link";
import { PopularTags } from "@/components/popular-tags";
import { ThoughtThread } from "@/components/thought-thread";
import { buildThoughtThreads } from "@/lib/utils";
import { TopFriends } from "@/components/top-friends";

export default async function Home() {
  const token = (await cookies()).get("auth_token")?.value ?? null;

  if (token) {
    return <FeedPage token={token} />;
  } else {
    return <LandingPage />;
  }
}

async function FeedPage({ token }: { token: string }) {
  const [feedData, me] = await Promise.all([
    getFeed(token),
    getMe(token).catch(() => null) as Promise<Me | null>,
  ]);

  const authors = [...new Set(feedData.thoughts.map((t) => t.authorUsername))];
  const userProfiles = await Promise.all(
    authors.map((username) => getUserProfile(username, token).catch(() => null))
  );

  const authorDetails = new Map<string, { avatarUrl?: string | null }>(
    userProfiles
      .filter((u): u is User => !!u)
      .map((user) => [user.username, { avatarUrl: user.avatarUrl }])
  );

  const { topLevelThoughts, repliesByParentId } = buildThoughtThreads(
    feedData.thoughts
  );

  return (
    <div className="container mx-auto max-w-6xl p-4 sm:p-6">
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
        <aside className="hidden lg:block lg:col-span-1">
          <div className="sticky top-20 space-y-6">
            <h2 className="text-lg font-semibold">Filters & Sorting</h2>
            <p className="text-sm text-muted-foreground">Coming soon...</p>
          </div>
        </aside>

        <main className="col-span-1 lg:col-span-2 space-y-6">
          <header className="mb-6">
            <h1 className="text-3xl font-bold">Your Feed</h1>
          </header>
          <PostThoughtForm />
          <div className="space-y-6">
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
                Your feed is empty. Follow some users to see their thoughts!
              </p>
            )}
          </div>
        </main>

        <aside className="hidden lg:block lg:col-span-1">
          <div className="sticky top-20 space-y-6">
            {me?.topFriends && <TopFriends usernames={me.topFriends} />}
            <PopularTags />
          </div>
        </aside>
      </div>
    </div>
  );
}

function LandingPage() {
  return (
    <div className="font-sans min-h-screen text-gray-800 flex items-center justify-center">
      <div className="container mx-auto max-w-2xl p-4 sm:p-6 text-center">
        <h1
          className="text-5xl font-bold"
          style={{ textShadow: "2px 2px 4px rgba(0,0,0,0.1)" }}
        >
          Welcome to Thoughts
        </h1>
        <p className="text-muted-foreground mt-2">
          Your space on the decentralized web.
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
  );
}
