import { RemoteActor } from "@/lib/api";
import { UserAvatar } from "./user-avatar";

interface RemoteFriendCardProps {
  actor: RemoteActor;
}

export function RemoteFriendCard({ actor }: RemoteFriendCardProps) {
  return (
    <a
      href={actor.url}
      target="_blank"
      rel="noopener noreferrer"
      className="flex items-center gap-4 p-4 -mx-6 hover:bg-accent transition-colors"
    >
      <UserAvatar
        src={actor.avatarUrl}
        alt={actor.displayName ?? actor.handle}
      />
      <div className="flex-1 min-w-0">
        <p className="font-bold truncate">
          {actor.displayName || actor.handle}
        </p>
        <p className="text-sm text-muted-foreground truncate">
          @{actor.handle}
        </p>
      </div>
      <span className="text-xs px-2 py-0.5 rounded-full bg-blue-500/10 border border-blue-500/20 text-blue-500 shrink-0">
        federated
      </span>
    </a>
  );
}
