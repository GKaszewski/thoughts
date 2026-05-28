import { getTopFriends, getMyFriends, getMyRemoteFriends } from "@/lib/api";
import { TopFriends } from "./top-friends";
import { AllFriendsCard } from "./all-friends-card";

interface ProfileFriendsWidgetProps {
  username: string;
  isOwnProfile: boolean;
  token: string | null;
}

export async function ProfileFriendsWidget({
  username,
  isOwnProfile,
  token,
}: ProfileFriendsWidgetProps) {
  const topFriendsData = await getTopFriends(username, token).catch(() => ({
    topFriends: [],
  }));

  if (topFriendsData.topFriends.length > 0) {
    return <TopFriends username={username} />;
  }

  if (!isOwnProfile || !token) return null;

  const [localData, remoteData] = await Promise.all([
    getMyFriends(token).catch(() => ({ items: [], total: 0, page: 1, perPage: 50 })),
    getMyRemoteFriends(token).catch(() => []),
  ]);

  return (
    <AllFriendsCard
      localFriends={localData.items}
      remoteFriends={remoteData}
    />
  );
}
