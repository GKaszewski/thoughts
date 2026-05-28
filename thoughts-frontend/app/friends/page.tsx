// app/friends/page.tsx
import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import { getMe, getMyFriends, getMyRemoteFriends, getTopFriends } from "@/lib/api";
import { FriendsList } from "@/components/friends-list";
import { TopFriendsStrip } from "@/components/top-friends-strip";

export default async function FriendsPage() {
  const token = (await cookies()).get("auth_token")?.value;
  if (!token) redirect("/login");

  const me = await getMe(token).catch(() => null);
  if (!me) redirect("/login");

  const [localFriendsData, remoteFriends, topFriendsData] = await Promise.all([
    getMyFriends(token).catch(() => ({
      items: [],
      total: 0,
      page: 1,
      perPage: 50,
    })),
    getMyRemoteFriends(token).catch(() => []),
    getTopFriends(me.username, token).catch(() => ({ topFriends: [] })),
  ]);

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6">
        <h1 className="text-3xl font-bold">Friends</h1>
        <p className="text-muted-foreground">
          People who follow you and who you follow back.
        </p>
      </header>
      <main className="space-y-6">
        <TopFriendsStrip topFriends={topFriendsData.topFriends} />
        <FriendsList
          localFriends={localFriendsData.items}
          remoteFriends={remoteFriends}
        />
      </main>
    </div>
  );
}
