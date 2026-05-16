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
      <CardHeader id="top-friends__header" className="p-0 pb-3">
        <CardTitle id="top-friends__title" className="text-lg flex items-center gap-2">
          <span className="widget-icon widget-icon-green">👥</span>
          Top Friends
        </CardTitle>
      </CardHeader>
      <CardContent id="top-friends__content" className="p-0 space-y-1">
        {friends.map((friend) => (
          <Link
            id={`top-friends__link-${friend.id}`}
            href={`/users/${friend.username}`}
            key={friend.id}
            className="flex items-center gap-3 py-2 px-2 -mx-2 rounded-lg hover:bg-accent/50 transition-colors"
          >
            <UserAvatar src={friend.avatarUrl} alt={friend.displayName || friend.username} />
            <div className="flex flex-col min-w-0">
              <span className="text-xs font-semibold truncate text-shadow-sm">
                {friend.displayName || friend.username}
              </span>
              <span className="text-[10px] text-muted-foreground truncate">
                @{friend.username}
              </span>
            </div>
            <span className="ml-auto shrink-0 text-[10px] font-semibold px-2 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-600">
              following
            </span>
          </Link>
        ))}
      </CardContent>
    </Card>
  );
}
