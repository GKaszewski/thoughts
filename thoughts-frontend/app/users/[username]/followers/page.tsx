import { cookies } from "next/headers";
import { notFound } from "next/navigation";
import { getFollowersList, getMe } from "@/lib/api";
import { UserListCard } from "@/components/user-list-card";
import { RemoteFollowers } from "@/components/federation/remote-followers";

interface FollowersPageProps {
  params: Promise<{ username: string }>;
}

export default async function FollowersPage({ params }: FollowersPageProps) {
  const { username } = await params;
  const token = (await cookies()).get("auth_token")?.value ?? null;

  const [followersData, me] = await Promise.all([
    getFollowersList(username, token).catch(() => null),
    token ? getMe(token).catch(() => null) : null,
  ]);

  if (!followersData) {
    notFound();
  }

  const isOwnProfile = me?.username === username;

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6">
        <h1 className="text-3xl font-bold">Followers</h1>
        <p className="text-muted-foreground">Users following @{username}.</p>
      </header>
      <main className="space-y-8">
        <UserListCard users={followersData.items} />
        {isOwnProfile && (
          <section>
            <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3">
              Remote followers
            </h2>
            <RemoteFollowers />
          </section>
        )}
      </main>
    </div>
  );
}
