import Link from "next/link";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { UserAvatar } from "./user-avatar";
import { getUserProfile, User } from "@/lib/api";
import { cookies } from "next/headers";

interface TopFriendsProps {
  usernames: string[];
}

// This is an async Server Component
export async function TopFriends({ usernames }: TopFriendsProps) {
  const token = (await cookies()).get("auth_token")?.value ?? null;

  if (usernames.length === 0) {
    return (
      <Card className="p-4">
        <CardHeader className="p-0 pb-2">
          <CardTitle className="text-lg text-shadow-md">Top Friends</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <p className="text-sm text-muted-foreground">
            No top friends to display.
          </p>
        </CardContent>
      </Card>
    );
  }

  // Fetch all top friend profiles in parallel
  const friendsResults = await Promise.allSettled(
    usernames.map((username) => getUserProfile(username, token))
  );

  const friends = friendsResults
    .filter(
      (result): result is PromiseFulfilledResult<User> =>
        result.status === "fulfilled"
    )
    .map((result) => result.value);

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
              className="text-xs truncate w-full group-hover:underline font-medium text-shadow-sm"
            >
              {friend.displayName || friend.username}
            </span>
          </Link>
        ))}
      </CardContent>
    </Card>
  );
}
