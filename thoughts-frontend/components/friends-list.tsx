import Link from "next/link";
import { User, RemoteActor } from "@/lib/api";
import { UserAvatar } from "./user-avatar";
import { RemoteFriendCard } from "./remote-friend-card";
import { Card, CardContent } from "./ui/card";

interface FriendsListProps {
  localFriends: User[];
  remoteFriends: RemoteActor[];
}

export function FriendsList({ localFriends, remoteFriends }: FriendsListProps) {
  if (localFriends.length === 0 && remoteFriends.length === 0) {
    return (
      <p className="text-center text-muted-foreground pt-8">
        No friends yet. Follow someone and wait for them to follow back!
      </p>
    );
  }

  return (
    <Card>
      <CardContent className="divide-y px-6">
        {localFriends.map((user) => (
          <Link
            key={user.id}
            href={`/users/${user.username}`}
            className="flex items-center gap-4 p-4 -mx-6 hover:bg-accent transition-colors"
          >
            <UserAvatar src={user.avatarUrl} alt={user.displayName} />
            <div className="flex-1 min-w-0">
              <p className="font-bold truncate">
                {user.displayName || user.username}
              </p>
              <p className="text-sm text-muted-foreground">
                @{user.username}
              </p>
            </div>
          </Link>
        ))}
        {remoteFriends.map((actor) => (
          <RemoteFriendCard key={actor.url} actor={actor} />
        ))}
      </CardContent>
    </Card>
  );
}
