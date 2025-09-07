import { Me, ThoughtThread as ThoughtThreadType } from "@/lib/api";
import { ThoughtCard } from "./thought-card";

interface ThoughtThreadProps {
  thought: ThoughtThreadType;
  authorDetails: Map<string, { avatarUrl?: string | null }>;
  currentUser: Me | null;
  isReply?: boolean;
}

export function ThoughtThread({
  thought,
  authorDetails,
  currentUser,
  isReply = false,
}: ThoughtThreadProps) {
  const author = {
    username: thought.authorUsername,
    displayName: thought.authorDisplayName,
    ...authorDetails.get(thought.authorUsername),
  };

  return (
    <div id={`thought-thread-${thought.id}`} className="flex flex-col gap-0">
      <ThoughtCard
        thought={thought}
        author={author}
        currentUser={currentUser}
        isReply={isReply}
      />

      {thought.replies.length > 0 && (
        <div
          id={`thought-thread-${thought.id}__replies`}
          className="pl-6 border-l-2 border-primary border-dashed ml-6 flex flex-col gap-4 pt-4"
        >
          {thought.replies.map((reply) => (
            <ThoughtThread
              key={reply.id}
              thought={reply}
              authorDetails={authorDetails}
              currentUser={currentUser}
              isReply={true}
            />
          ))}
        </div>
      )}
    </div>
  );
}
