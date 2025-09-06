import { cookies } from "next/headers";
import { notFound } from "next/navigation";
import { getFollowingList } from "@/lib/api";
import { UserListCard } from "@/components/user-list-card";

interface FollowingPageProps {
  params: { username: string };
}

export default async function FollowingPage({ params }: FollowingPageProps) {
  const { username } = params;
  const token = (await cookies()).get("auth_token")?.value ?? null;

  const followingData = await getFollowingList(username, token).catch(
    () => null
  );

  if (!followingData) {
    notFound();
  }

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6">
        <h1 className="text-3xl font-bold">Following</h1>
        <p className="text-muted-foreground">Users that @{username} follows.</p>
      </header>
      <main>
        <UserListCard users={followingData.users} />
      </main>
    </div>
  );
}
