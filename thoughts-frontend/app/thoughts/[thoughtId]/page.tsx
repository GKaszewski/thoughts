import type { Metadata } from "next";
import { cookies } from "next/headers";
import {
  getThoughtById,
  getThoughtThread,
  getMe,
  Me,
  ThoughtThread as ThoughtThreadType,
} from "@/lib/api";
import { ThoughtThread } from "@/components/thought-thread";
import { notFound } from "next/navigation";

interface ThoughtPageProps {
  params: Promise<{ thoughtId: string }>;
}

function stripHtml(html: string) {
  return html.replace(/<[^>]*>/g, "").trim();
}

export async function generateMetadata({
  params,
}: ThoughtPageProps): Promise<Metadata> {
  const { thoughtId } = await params;
  const thought = await getThoughtById(thoughtId, null).catch(() => null);
  if (!thought) return { title: "Thought" };

  const author = thought.author.displayName || thought.author.username;
  const preview = stripHtml(thought.content).slice(0, 120);
  const description = preview || `A thought by ${author}`;

  return {
    title: `${author}: "${preview.slice(0, 60)}${preview.length > 60 ? "…" : ""}"`,
    description,
    openGraph: {
      type: "article",
      title: `${author} on Thoughts`,
      description,
      images: thought.author.avatarUrl
        ? [{ url: thought.author.avatarUrl }]
        : [],
      publishedTime: thought.createdAt.toISOString(),
    },
    twitter: {
      card: "summary",
      title: `${author} on Thoughts`,
      description,
      images: thought.author.avatarUrl ? [thought.author.avatarUrl] : [],
    },
  };
}

export default async function ThoughtPage({ params }: ThoughtPageProps) {
  const { thoughtId } = await params;
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

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6">
        <h1 className="text-3xl font-bold">Thoughts</h1>
      </header>
      <main>
        <ThoughtThread
          thought={thread}
          currentUser={me}
        />
      </main>
    </div>
  );
}
