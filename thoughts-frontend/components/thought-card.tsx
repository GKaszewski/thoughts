import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { UserAvatar } from "./user-avatar";
import { Thought } from "@/lib/api";
import { formatDistanceToNow } from "date-fns";

interface ThoughtCardProps {
  thought: Thought;
  author: {
    username: string;
    avatarUrl?: string | null;
  };
}

export function ThoughtCard({ thought, author }: ThoughtCardProps) {
  const timeAgo = formatDistanceToNow(new Date(thought.createdAt), {
    addSuffix: true,
  });

  return (
    <Card>
      <CardHeader className="flex flex-row items-center gap-4 space-y-0">
        <UserAvatar src={author.avatarUrl} alt={author.username} />
        <div className="flex flex-col">
          <span className="font-bold">{author.username}</span>
          <span className="text-sm text-muted-foreground">{timeAgo}</span>
        </div>
      </CardHeader>
      <CardContent>
        <p className="whitespace-pre-wrap break-words">{thought.content}</p>
      </CardContent>
    </Card>
  );
}
