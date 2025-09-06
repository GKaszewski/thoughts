import { Me, Thought } from "@/lib/api";
import { ThoughtCard } from "./thought-card";

interface ThoughtThreadProps {
  thought: Thought;
  repliesByParentId: Map<string, Thought[]>;
  authorDetails: Map<string, { avatarUrl?: string | null }>;
  currentUser: Me | null;
  isReply?: boolean;
}

export function ThoughtThread({
  thought,
  repliesByParentId,
  authorDetails,
  currentUser,
  isReply = false,
}: ThoughtThreadProps) {
  const author = {
    username: thought.authorUsername,
    avatarUrl: null,
    ...authorDetails.get(thought.authorUsername),
  };

  const directReplies = repliesByParentId.get(thought.id) || [];

  return (
    <div id={`thought-thread-${thought.id}`} className="flex flex-col gap-0">
      <ThoughtCard
        thought={thought}
        author={author}
        currentUser={currentUser}
        isReply={isReply}
      />

      {directReplies.length > 0 && (
        <div
          id={`thought-thread-${thought.id}__replies`}
          className="pl-6 border-l-2 border-primary border-dashed ml-6 flex flex-col gap-4 pt-4"
        >
          {directReplies.map((reply) => (
            <ThoughtThread // RECURSIVE CALL
              key={reply.id}
              thought={reply}
              repliesByParentId={repliesByParentId} // Pass the full map down
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
