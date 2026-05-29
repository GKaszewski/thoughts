import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { cookies } from "next/headers";
import { getMe, getRemoteFollowing, lookupRemoteActor, getRemoteActorPosts, Me } from "@/lib/api";
import { RemoteUserProfile } from "@/components/remote-user-profile";

interface RemoteActorPageProps {
  searchParams: Promise<{ handle?: string }>;
}

function stripHtml(html: string) {
  return html.replace(/<[^>]*>/g, "").trim();
}

export async function generateMetadata({
  searchParams,
}: RemoteActorPageProps): Promise<Metadata> {
  const { handle } = await searchParams;
  if (!handle) return { title: "Profile" };

  const token = (await cookies()).get("auth_token")?.value ?? null;
  const actor = await lookupRemoteActor(handle, token).catch(() => null);
  if (!actor) return { title: handle };

  const name = actor.displayName || actor.handle;
  const description = actor.bio
    ? stripHtml(actor.bio).slice(0, 160)
    : `${name} on the Fediverse. Follow from Thoughts.`;

  return {
    title: `${name} (${actor.handle})`,
    description,
    openGraph: {
      type: "profile",
      title: `${name} (${actor.handle})`,
      description,
      images: actor.avatarUrl ? [{ url: actor.avatarUrl }] : [],
    },
    twitter: {
      card: "summary",
      title: `${name} · Thoughts`,
      description,
      images: actor.avatarUrl ? [actor.avatarUrl] : [],
    },
  };
}

export default async function RemoteActorPage({
  searchParams,
}: RemoteActorPageProps) {
  const { handle } = await searchParams;
  if (!handle) notFound();

  const token = (await cookies()).get("auth_token")?.value ?? null;

  const [actorResult, postsResult, meResult, followingResult] = await Promise.allSettled([
    lookupRemoteActor(handle, token),
    getRemoteActorPosts(handle, 1, token),
    token ? getMe(token) : Promise.resolve(null),
    token ? getRemoteFollowing(token) : Promise.resolve([]),
  ]);

  if (actorResult.status === "rejected") {
    notFound();
  }

  const actor = actorResult.value;
  const postsData = postsResult.status === "fulfilled" ? postsResult.value : null;
  const posts = postsData?.items ?? [];
  const totalPages = postsData
    ? Math.ceil(postsData.total / postsData.perPage)
    : 1;
  const me =
    meResult.status === "fulfilled" ? (meResult.value as Me | null) : null;
  const following =
    followingResult.status === "fulfilled" ? followingResult.value : [];
  const initialFollowed = following.some((f) => f.url === actor.url);

  return (
    <RemoteUserProfile
      key={actor.url}
      actor={actor}
      handle={handle}
      initialPosts={posts}
      initialTotalPages={totalPages}
      me={me}
      initialFollowed={initialFollowed}
    />
  );
}
