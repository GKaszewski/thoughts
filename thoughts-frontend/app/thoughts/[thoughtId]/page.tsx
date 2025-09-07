import { cookies } from "next/headers";
import {
  getThoughtThread,
  getUserProfile,
  getMe,
  Me,
  User,
  ThoughtThread as ThoughtThreadType,
} from "@/lib/api";
import { ThoughtThread } from "@/components/thought-thread";
import { notFound } from "next/navigation";

interface ThoughtPageProps {
  params: { thoughtId: string };
}

function collectAuthors(thread: ThoughtThreadType): string[] {
  const authors = new Set<string>([thread.authorUsername]);
  for (const reply of thread.replies) {
    collectAuthors(reply).forEach((author) => authors.add(author));
  }
  return Array.from(authors);
}

export default async function ThoughtPage({ params }: ThoughtPageProps) {
  const { thoughtId } = params;
  const token = (await cookies()).get("auth_token")?.value ?? null;

  const [threadResult, meResult] = await Promise.allSettled([
    getThoughtThread(thoughtId, token),
    token ? getMe(token) : Promise.resolve(null),
  ]);

  if (threadResult.status === "rejected") {
    notFound();
  }

  const thread = threadResult.value;
  const me = meResult.status === "fulfilled" ? (meResult.value as Me) : null;

  // Fetch details for all authors in the thread efficiently
  const authorUsernames = collectAuthors(thread);
  const userProfiles = await Promise.all(
    authorUsernames.map((username) =>
      getUserProfile(username, token).catch(() => null)
    )
  );

  const authorDetails = new Map<string, { avatarUrl?: string | null }>(
    userProfiles
      .filter((u): u is User => !!u)
      .map((user) => [user.username, { avatarUrl: user.avatarUrl }])
  );

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6">
        <h1 className="text-3xl font-bold">Thoughts</h1>
      </header>
      <main>
        <ThoughtThread
          thought={thread}
          authorDetails={authorDetails}
          currentUser={me}
        />
      </main>
    </div>
  );
}
