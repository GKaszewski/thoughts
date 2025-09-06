import { cookies } from "next/headers";
import {
  getThoughtById,
  getUserThoughts,
  getUserProfile,
  getMe,
  Me,
  Thought,
} from "@/lib/api";
import { buildThoughtThreads } from "@/lib/utils";
import { ThoughtThread } from "@/components/thought-thread";
import { notFound } from "next/navigation";

interface ThoughtPageProps {
  params: { thoughtId: string };
}

async function findConversationRoot(
  startThought: Thought,
  token: string | null
): Promise<Thought> {
  let currentThought = startThought;
  while (currentThought.replyToId) {
    const parentThought = await getThoughtById(
      currentThought.replyToId,
      token
    ).catch(() => null);
    if (!parentThought) break;
    currentThought = parentThought;
  }
  return currentThought;
}

export default async function ThoughtPage({ params }: ThoughtPageProps) {
  const { thoughtId } = params;
  const token = (await cookies()).get("auth_token")?.value ?? null;

  const initialThought = await getThoughtById(thoughtId, token).catch(
    () => null
  );

  if (!initialThought) {
    notFound();
  }

  const rootThought = await findConversationRoot(initialThought, token);

  const [thoughtsResult, meResult] = await Promise.allSettled([
    getUserThoughts(rootThought.authorUsername, token),
    token ? getMe(token) : Promise.resolve(null),
  ]);

  if (thoughtsResult.status === "rejected") {
    notFound();
  }

  const allThoughts = thoughtsResult.value.thoughts;
  const me = meResult.status === "fulfilled" ? (meResult.value as Me) : null;

  const author = await getUserProfile(rootThought.authorUsername, token).catch(
    () => null
  );
  const authorDetails = new Map<string, { avatarUrl?: string | null }>();
  if (author) {
    authorDetails.set(author.username, { avatarUrl: author.avatarUrl });
  }

  const { repliesByParentId } = buildThoughtThreads(allThoughts);

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6">
        <h1 className="text-3xl font-bold">Thoughts</h1>
      </header>
      <main>
        <ThoughtThread
          thought={rootThought}
          repliesByParentId={repliesByParentId}
          authorDetails={authorDetails}
          currentUser={me}
        />
      </main>
    </div>
  );
}
