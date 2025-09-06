import Link from "next/link";
import { User } from "@/lib/api";
import { UserAvatar } from "./user-avatar";
import { Card, CardContent } from "./ui/card";

interface UserListCardProps {
  users: User[];
}

export function UserListCard({ users }: UserListCardProps) {
  if (users.length === 0) {
    return (
      <p className="text-center text-muted-foreground pt-8">
        No users to display.
      </p>
    );
  }

  return (
    <Card>
      <CardContent className="divide-y">
        {users.map((user) => (
          <Link
            href={`/users/${user.username}`}
            key={user.id}
            className="flex items-center gap-4 p-4 -mx-6 hover:bg-accent"
          >
            <UserAvatar src={user.avatarUrl} alt={user.displayName} />
            <div>
              <p className="font-bold">{user.displayName || user.username}</p>
              <p className="text-sm text-muted-foreground">@{user.username}</p>
            </div>
          </Link>
        ))}
      </CardContent>
    </Card>
  );
}
