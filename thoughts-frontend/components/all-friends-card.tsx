import Link from "next/link";
import { User, RemoteActor } from "@/lib/api";
import { UserAvatar } from "./user-avatar";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface AllFriendsCardProps {
  localFriends: User[];
  remoteFriends: RemoteActor[];
}

export function AllFriendsCard({ localFriends, remoteFriends }: AllFriendsCardProps) {
  const total = localFriends.length + remoteFriends.length;
  if (total === 0) return null;

  const localSlice = localFriends.slice(0, 6);
  const remoteSlice = remoteFriends.slice(0, Math.max(0, 6 - localSlice.length));

  return (
    <Card id="all-friends" className="p-4">
      <CardHeader className="p-0 pb-3">
        <CardTitle className="text-lg">
          Friends
        </CardTitle>
      </CardHeader>
      <CardContent className="p-0 space-y-1">
        {localSlice.map((friend) => (
          <Link
            key={friend.id}
            href={`/users/${friend.username}`}
            className="flex items-center gap-3 py-2 px-2 -mx-2 rounded-lg hover:bg-accent/50 transition-colors"
          >
            <UserAvatar
              src={friend.avatarUrl}
              alt={friend.displayName || friend.username}
            />
            <div className="flex flex-col min-w-0">
              <span className="text-xs font-semibold truncate text-shadow-sm">
                {friend.displayName || friend.username}
              </span>
              <span className="text-[10px] text-muted-foreground truncate">
                @{friend.username}
              </span>
            </div>
          </Link>
        ))}
        {remoteSlice.map((actor) => (
          <a
            key={actor.url}
            href={actor.url}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-3 py-2 px-2 -mx-2 rounded-lg hover:bg-accent/50 transition-colors"
          >
            <UserAvatar
              src={actor.avatarUrl}
              alt={actor.displayName ?? actor.handle}
            />
            <div className="flex flex-col min-w-0 flex-1">
              <span className="text-xs font-semibold truncate">
                {actor.displayName || actor.handle}
              </span>
              <span className="text-[10px] text-muted-foreground truncate">
                @{actor.handle}
              </span>
            </div>
            <span className="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-blue-500/10 border border-blue-500/20 text-blue-500 shrink-0">
              federated
            </span>
          </a>
        ))}
        {total > 6 && (
          <Link
            href="/friends"
            className="block text-xs text-muted-foreground hover:text-foreground pt-2 transition-colors"
          >
            See all {total} friends →
          </Link>
        )}
      </CardContent>
    </Card>
  );
}
