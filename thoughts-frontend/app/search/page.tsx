import { cookies } from "next/headers";
import { getMe, search, User } from "@/lib/api";
import { UserListCard } from "@/components/user-list-card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ThoughtList } from "@/components/thought-list";

interface SearchPageProps {
  searchParams: { q?: string };
}

export default async function SearchPage({ searchParams }: SearchPageProps) {
  const query = searchParams.q || "";
  const token = (await cookies()).get("auth_token")?.value ?? null;

  if (!query) {
    return (
      <div className="container mx-auto max-w-2xl p-4 sm:p-6 text-center">
        <h1 className="text-2xl font-bold mt-8">Search Thoughts</h1>
        <p className="text-muted-foreground">
          Find users and thoughts across the platform.
        </p>
      </div>
    );
  }

  const [results, me] = await Promise.all([
    search(query, token).catch(() => null),
    token ? getMe(token).catch(() => null) : null,
  ]);

  const authorDetails = new Map<string, { avatarUrl?: string | null }>();
  if (results) {
    results.users.users.forEach((user: User) => {
      authorDetails.set(user.username, { avatarUrl: user.avatarUrl });
    });
  }

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6">
        <h1 className="text-3xl font-bold">Search Results</h1>
        <p className="text-muted-foreground">
          Showing results for: &quot;{query}&quot;
        </p>
      </header>
      <main>
        {results ? (
          <Tabs defaultValue="thoughts" className="w-full">
            <TabsList>
              <TabsTrigger value="thoughts">
                Thoughts ({results.thoughts.thoughts.length})
              </TabsTrigger>
              <TabsTrigger value="users">
                Users ({results.users.users.length})
              </TabsTrigger>
            </TabsList>
            <TabsContent value="thoughts">
              <ThoughtList
                thoughts={results.thoughts.thoughts}
                authorDetails={authorDetails}
                currentUser={me}
              />
            </TabsContent>
            <TabsContent value="users">
              <UserListCard users={results.users.users} />
            </TabsContent>
          </Tabs>
        ) : (
          <p className="text-center text-muted-foreground pt-8">
            No results found or an error occurred.
          </p>
        )}
      </main>
    </div>
  );
}
