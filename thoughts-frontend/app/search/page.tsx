import type { Metadata } from "next";
import { cookies } from "next/headers";
import { getMe, search, lookupRemoteActor } from "@/lib/api";

export async function generateMetadata({
  searchParams,
}: {
  searchParams: Promise<{ q?: string }>;
}): Promise<Metadata> {
  const { q } = await searchParams;
  const title = q ? `Search: "${q}"` : "Search";
  return {
    title,
    description: q
      ? `Search results for "${q}" on Thoughts`
      : "Search for people and thoughts on Thoughts",
  };
}
import { EmptyState } from "@/components/empty-state";
import { UserListCard } from "@/components/user-list-card";
import { RemoteUserCard } from "@/components/remote-user-card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ThoughtList } from "@/components/thought-list";

const HANDLE_RE = /^@[\w.-]+@[\w.-]+\.\w+$/;

interface SearchPageProps {
  searchParams: Promise<{ q?: string }>;
}

export default async function SearchPage({ searchParams }: SearchPageProps) {
  const { q } = await searchParams;
  const query = q || "";
  const token = (await cookies()).get("auth_token")?.value ?? null;

  if (!query) {
    return (
      <div className="container mx-auto max-w-2xl p-4 sm:p-6 text-center">
        <h1 className="text-2xl font-bold mt-8">Search Thoughts</h1>
        <p className="text-muted-foreground">
          Find users and thoughts across the platform.
        </p>
        <p className="text-xs text-muted-foreground mt-1">
          To find someone on Mastodon, type their full handle: @alice@mastodon.social
        </p>
      </div>
    );
  }

  const isHandle = HANDLE_RE.test(query);
  const isPartialHandle = !isHandle && query.includes("@");

  const [results, remoteActor, me] = await Promise.all([
    isHandle ? null : search(query, token).catch(() => null),
    isHandle ? lookupRemoteActor(query, token).catch(() => null) : null,
    token ? getMe(token).catch(() => null) : null,
  ]);

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6">
        <h1 className="text-3xl font-bold">Search Results</h1>
        <p className="text-muted-foreground">
          Showing results for: &quot;{query}&quot;
        </p>
      </header>
      <main>
        {isPartialHandle && (
          <p className="text-xs text-muted-foreground mb-4">
            Looks like a fediverse handle. Use the full format: @alice@mastodon.social
          </p>
        )}
        {isHandle ? (
          remoteActor ? (
            <div className="space-y-4">
              <h2 className="text-lg font-semibold">Remote user</h2>
              <RemoteUserCard actor={remoteActor} />
            </div>
          ) : (
            <EmptyState emoji="🔍" title="No results" message={`No user found at ${query}`} />
          )
        ) : results ? (
          <Tabs defaultValue="thoughts" className="w-full">
            <TabsList>
              <TabsTrigger value="thoughts">
                Thoughts ({results.thoughts.length})
              </TabsTrigger>
              <TabsTrigger value="users">
                Users ({results.users.length})
              </TabsTrigger>
            </TabsList>
            <TabsContent value="thoughts">
              <ThoughtList
                thoughts={results.thoughts}
                currentUser={me}
              />
            </TabsContent>
            <TabsContent value="users">
              <UserListCard users={results.users} />
            </TabsContent>
          </Tabs>
        ) : (
          <EmptyState emoji="🔍" title="No results" message="No results found or an error occurred." />
        )}
      </main>
    </div>
  );
}
