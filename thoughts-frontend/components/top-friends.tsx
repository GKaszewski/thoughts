import Link from "next/link";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { UserAvatar } from "./user-avatar";
import { getTopFriends } from "@/lib/api";
import { cookies } from "next/headers";

interface TopFriendsProps {
  username: string;
}

export async function TopFriends({ username }: TopFriendsProps) {
  const token = (await cookies()).get("auth_token")?.value ?? null;
  const data = await getTopFriends(username, token).catch(() => ({ topFriends: [] }));
  const friends = data.topFriends;

  if (friends.length === 0) return null;

  return (
    <Card id="top-friends" className="p-4">
      <CardHeader id="top-friends__header" className="p-0 pb-2">
        <CardTitle id="top-friends__title" className="text-lg text-shadow-md">
          Top Friends
        </CardTitle>
      </CardHeader>
      <CardContent id="top-friends__content" className="p-0">
        {friends.map((friend) => (
          <Link
            id={`top-friends__link-${friend.id}`}
            href={`/users/${friend.username}`}
            key={friend.id}
            className="flex items-center gap-3 py-2 px-2 -mx-2 rounded-lg hover:bg-accent/50 transition-colors"
          >
            <UserAvatar src={friend.avatarUrl} alt={friend.username} />
            <span
              id={`top-friends__name-${friend.id}`}
              className="text-xs truncate w-full font-medium text-shadow-sm"
            >
              {friend.displayName || friend.username}
            </span>
          </Link>
        ))}
      </CardContent>
    </Card>
  );
}
