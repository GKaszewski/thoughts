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
      <Card>
        <CardHeader>
          <CardTitle>Top Friends</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-center text-muted-foreground">
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
    <Card>
      <CardHeader>
        <CardTitle>Top Friends</CardTitle>
      </CardHeader>
      <CardContent className="grid grid-cols-4 gap-4">
        {friends.map((friend) => (
          <Link
            href={`/users/${friend.username}`}
            key={friend.id}
            className="flex flex-col items-center gap-2 text-center group"
          >
            <UserAvatar src={friend.avatarUrl} alt={friend.username} />
            <span className="text-xs font-medium truncate w-full group-hover:underline">
              {friend.displayName || friend.username}
            </span>
          </Link>
        ))}
      </CardContent>
    </Card>
  );
}
