import { Me, Thought } from "@/lib/api";
import { ThoughtCard } from "./thought-card";
import { Card, CardContent } from "./ui/card";

interface ThoughtListProps {
  thoughts: Thought[];
  authorDetails: Map<string, { avatarUrl?: string | null }>;
  currentUser: Me | null;
}

export function ThoughtList({
  thoughts,
  authorDetails,
  currentUser,
}: ThoughtListProps) {
  if (thoughts.length === 0) {
    return (
      <p className="text-center text-muted-foreground pt-8">
        No thoughts to display.
      </p>
    );
  }

  return (
    <Card>
      <CardContent className="divide-y p-0">
        <div className="space-y-6 p-4">
          {thoughts.map((thought) => {
            const author = {
              username: thought.authorUsername,
              displayName: thought.authorDisplayName,
              ...authorDetails.get(thought.authorUsername),
            };
            return (
              <ThoughtCard
                key={thought.id}
                thought={thought}
                author={author}
                currentUser={currentUser}
              />
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}
