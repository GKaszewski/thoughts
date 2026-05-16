import { cookies } from "next/headers";
import { notFound } from "next/navigation";
import { getFollowingList, getMe } from "@/lib/api";
import { UserListCard } from "@/components/user-list-card";
import { RemoteFollowing } from "@/components/federation/remote-following";

interface FollowingPageProps {
  params: Promise<{ username: string }>;
}

export default async function FollowingPage({ params }: FollowingPageProps) {
  const { username } = await params;
  const token = (await cookies()).get("auth_token")?.value ?? null;

  const [followingData, me] = await Promise.all([
    getFollowingList(username, token).catch(() => null),
    token ? getMe(token).catch(() => null) : null,
  ]);

  if (!followingData) {
    notFound();
  }

  const isOwnProfile = me?.username === username;

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6">
        <h1 className="text-3xl font-bold">Following</h1>
        <p className="text-muted-foreground">Users that @{username} follows.</p>
      </header>
      <main className="space-y-8">
        <UserListCard users={followingData.items} />
        {isOwnProfile && (
          <section>
            <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3">
              Remote following
            </h2>
            <RemoteFollowing />
          </section>
        )}
      </main>
    </div>
  );
}
