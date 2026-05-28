import Link from "next/link";
import { User } from "@/lib/api";
import { UserAvatar } from "./user-avatar";

interface TopFriendsStripProps {
  topFriends: User[];
}

export function TopFriendsStrip({ topFriends }: TopFriendsStripProps) {
  return (
    <div className="flex items-center gap-3 p-4 rounded-lg bg-accent/30">
      <div className="flex-1 flex items-center gap-2 flex-wrap">
        {topFriends.length === 0 ? (
          <p className="text-sm text-muted-foreground">No top friends yet.</p>
        ) : (
          topFriends.map((f) => (
            <Link
              key={f.id}
              href={`/users/${f.username}`}
              title={f.displayName ?? f.username}
            >
              <UserAvatar
                src={f.avatarUrl}
                alt={f.displayName ?? f.username}
                className="h-8 w-8"
              />
            </Link>
          ))
        )}
      </div>
      <Link
        href="/settings/friends"
        className="text-sm text-muted-foreground hover:underline shrink-0"
      >
        Edit →
      </Link>
    </div>
  );
}
