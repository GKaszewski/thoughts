import { cookies } from "next/headers";
import { notFound } from "next/navigation";
import { getFollowersList } from "@/lib/api";
import { UserListCard } from "@/components/user-list-card";

interface FollowersPageProps {
  params: { username: string };
}

export default async function FollowersPage({ params }: FollowersPageProps) {
  const { username } = params;
  const token = (await cookies()).get("auth_token")?.value ?? null;

  const followersData = await getFollowersList(username, token).catch(
    () => null
  );

  if (!followersData) {
    notFound();
  }

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6">
        <h1 className="text-3xl font-bold">Followers</h1>
        <p className="text-muted-foreground">Users following @{username}.</p>
      </header>
      <main>
        <UserListCard users={followersData.users} />
      </main>
    </div>
  );
}
