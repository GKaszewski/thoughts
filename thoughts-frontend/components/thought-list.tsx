import { Me, Thought } from "@/lib/api";
import { ThoughtCard } from "./thought-card";
import { Card, CardContent } from "./ui/card";

interface ThoughtListProps {
  thoughts: Thought[];
  currentUser: Me | null;
}

export function ThoughtList({ thoughts, currentUser }: ThoughtListProps) {
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
          {thoughts.map((thought) => (
            <ThoughtCard
              key={thought.id}
              thought={thought}
              currentUser={currentUser}
            />
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
