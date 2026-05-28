import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import { getMe, getMyFriends, getTopFriends } from "@/lib/api";
import { TopFriendsEditor } from "@/components/top-friends-editor";

export default async function FriendsSettingsPage() {
  const token = (await cookies()).get("auth_token")?.value;
  if (!token) redirect("/login");

  const me = await getMe(token).catch(() => null);
  if (!me) redirect("/login");

  const [localFriendsData, topFriendsData] = await Promise.all([
    getMyFriends(token).catch(() => ({ items: [], total: 0, page: 1, perPage: 50 })),
    getTopFriends(me.username, token).catch(() => ({ topFriends: [] })),
  ]);

  return (
    <div className="space-y-6">
      <div className="glass-effect glossy-effect bottom rounded-md shadow-fa-lg p-4">
        <h3 className="text-lg font-medium">Top Friends</h3>
        <p className="text-sm text-muted-foreground">
          Choose up to 8 friends to feature on your profile. Only local friends
          can be top friends.
        </p>
      </div>
      <TopFriendsEditor
        token={token}
        initialTopFriends={topFriendsData.topFriends}
        localFriends={localFriendsData.items}
      />
    </div>
  );
}
