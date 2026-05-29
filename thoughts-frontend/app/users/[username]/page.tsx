import type { Metadata } from "next";
import {
  getFollowersList,
  getFollowingList,
  getMe,
  getRemoteFollowers,
  getRemoteFollowing,
  getUserProfile,
  getUserThoughts,
  Me,
} from "@/lib/api";
import { CssPreviewListener } from "@/components/css-preview-listener";

interface ProfilePageProps {
  params: Promise<{ username: string }>;
}

export async function generateMetadata({
  params,
}: ProfilePageProps): Promise<Metadata> {
  const { username } = await params;
  const user = await getUserProfile(username, null).catch(() => null);
  if (!user) return { title: username };

  const name = user.displayName || user.username;
  const description =
    user.bio ||
    `Follow ${name} on Thoughts and across the Fediverse.`;

  return {
    title: `${name} (@${user.username})`,
    description,
    openGraph: {
      type: "profile",
      title: `${name} (@${user.username})`,
      description,
      images: user.avatarUrl ? [{ url: user.avatarUrl }] : [],
    },
    twitter: {
      card: "summary",
      title: `${name} (@${user.username})`,
      description,
      images: user.avatarUrl ? [user.avatarUrl] : [],
    },
  };
}
import { EmptyState } from "@/components/empty-state";
import { UserAvatar } from "@/components/user-avatar";
import { Calendar, Settings } from "lucide-react";
import { Card } from "@/components/ui/card";
import { notFound } from "next/navigation";
import { cookies } from "next/headers";
import { FollowButton } from "@/components/follow-button";
import { ProfileFriendsWidget } from "@/components/profile-friends-widget";
import { Suspense } from "react";
import { ProfileSkeleton } from "@/components/loading-skeleton";
import { UserThoughtsList } from "@/components/user-thoughts-list";
import { Button } from "@/components/ui/button";
import Link from "next/link";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { PendingRequests } from "@/components/federation/pending-requests";
import { CopyButton } from "@/components/copy-button";
import { HelpCircle } from "lucide-react";

interface ProfilePageProps {
  params: Promise<{ username: string }>;
}

export default async function ProfilePage({ params }: ProfilePageProps) {
  const { username } = await params;
  const token = (await cookies()).get("auth_token")?.value ?? null;

  const userProfilePromise = getUserProfile(username, token);
  const thoughtsPromise = getUserThoughts(username, token);
  const mePromise = token ? getMe(token) : Promise.resolve(null);
  const followersPromise = getFollowersList(username, token);
  const followingPromise = getFollowingList(username, token);

  const [
    userResult,
    thoughtsResult,
    meResult,
    followersResult,
    followingResult,
  ] = await Promise.allSettled([
    userProfilePromise,
    thoughtsPromise,
    mePromise,
    followersPromise,
    followingPromise,
  ]);

  if (userResult.status === "rejected") {
    notFound();
  }

  const user = userResult.value;
  const me = meResult.status === "fulfilled" ? (meResult.value as Me) : null;

  const thoughtsData = thoughtsResult.status === "fulfilled" ? thoughtsResult.value : null;
  const thoughts = thoughtsData?.items ?? [];
  const totalPages = thoughtsData
    ? Math.ceil(thoughtsData.total / thoughtsData.perPage)
    : 1;

  const localFollowersCount =
    followersResult.status === "fulfilled"
      ? followersResult.value.total
      : 0;
  const localFollowingCount =
    followingResult.status === "fulfilled"
      ? followingResult.value.total
      : 0;

  const isOwnProfile = me?.username === user.username;

  const [remoteFollowersCount, remoteFollowingCount] =
    isOwnProfile && token
      ? await Promise.all([
          getRemoteFollowers(token).then((r) => r.length).catch(() => 0),
          getRemoteFollowing(token).then((r) => r.length).catch(() => 0),
        ])
      : [0, 0];

  const followersCount = localFollowersCount + remoteFollowersCount;
  const followingCount = localFollowingCount + remoteFollowingCount;
  const isFollowing = user.isFollowedByViewer;

  const fediverseDomain =
    process.env.NEXT_PUBLIC_FEDIVERSE_DOMAIN ??
    (process.env.NEXT_PUBLIC_API_URL
      ? new URL(process.env.NEXT_PUBLIC_API_URL).hostname
      : null);
  const fediverseHandle =
    user.local && fediverseDomain ? `@${user.username}@${fediverseDomain}` : null;

  return (
    <div id={`profile-page-${user.username}`}>
      {user.customCss && (
        <style dangerouslySetInnerHTML={{ __html: user.customCss }} />
      )}
      <CssPreviewListener />

      <div
        id="profile-header"
        className="h-48 bg-gray-200 bg-cover bg-center profile-header"
        style={{
          backgroundImage: user.headerUrl ? `url(${user.headerUrl})` : "none",
        }}
      />

      <main
        id="main-container"
        className="container mx-auto max-w-6xl p-4 -mt-16 grid grid-cols-1 lg:grid-cols-4 gap-8"
      >
        {/* Left Sidebar (Profile Card & Top Friends) */}
        <aside id="left-sidebar" className="col-span-1 lg:col-span-1 space-y-6">
          <div id="left-sidebar__inner" className="sticky top-20 space-y-6">
            <Card id="profile-card" className="p-6 bg-card/80 backdrop-blur-lg">
              <div
                id="profile-card__inner"
                className="flex justify-between items-start"
              >
                <div id="profile-card__avatar" className="flex items-end gap-4">
                  <div
                    id="profile-card__avatar-image"
                    className="w-24 h-24 rounded-full border-4 border-background shrink-0"
                  >
                    <UserAvatar
                      src={user.avatarUrl}
                      alt={user.displayName}
                      className="w-full h-full"
                    />
                  </div>
                </div>
                {/* Action Button */}
                <div id="profile-card__action">
                  {isOwnProfile ? (
                    <Button asChild variant="outline" size="sm">
                      <Link href="/settings/profile">
                        <Settings className="mr-2 h-4 w-4" /> Edit
                      </Link>
                    </Button>
                  ) : token ? (
                    <FollowButton
                      username={user.username}
                      isInitiallyFollowing={isFollowing}
                    />
                  ) : null}
                </div>
              </div>

              <div id="profile-card__info" className="mt-4">
                <h1 id="profile-card__name" className="text-2xl font-bold">
                  {user.displayName || user.username}
                </h1>
                <p
                  id="profile-card__username"
                  className="text-sm text-muted-foreground"
                >
                  @{user.username}
                </p>
                {fediverseHandle && (
                  <div className="flex items-center gap-1 mt-0.5">
                    <p className="text-xs text-muted-foreground/70 font-mono break-all">
                      {fediverseHandle}
                    </p>
                    <CopyButton text={fediverseHandle} />
                    <Link
                      href="/about/fediverse"
                      title="What is the Fediverse?"
                      className="inline-flex items-center text-muted-foreground/50 hover:text-muted-foreground transition-colors"
                    >
                      <HelpCircle className="h-3 w-3" />
                    </Link>
                  </div>
                )}
              </div>

              <p
                id="profile-card__bio"
                className="mt-4 text-sm whitespace-pre-wrap"
              >
                {user.bio}
              </p>

              {isOwnProfile && (
                <div
                  id="profile-card__stats"
                  className="flex items-center gap-4 mt-4 text-sm"
                >
                  <Link
                    href={`/users/${user.username}/following`}
                    className="hover:underline"
                  >
                    <span className="font-bold">{followingCount}</span>
                    <span className="text-muted-foreground ml-1">
                      Following
                    </span>
                  </Link>
                  <Link
                    href={`/users/${user.username}/followers`}
                    className="hover:underline"
                  >
                    <span className="font-bold">{followersCount}</span>
                    <span className="text-muted-foreground ml-1">
                      Followers
                    </span>
                  </Link>
                </div>
              )}

              <div
                id="profile-card__joined"
                className="flex items-center gap-2 mt-4 text-sm text-muted-foreground"
              >
                <Calendar className="h-4 w-4" />
                <span>
                  Joined {user.joinedAt ? new Date(user.joinedAt).toLocaleDateString() : "Unknown"}
                </span>
              </div>
            </Card>

            <Suspense fallback={<ProfileSkeleton />}>
              <ProfileFriendsWidget
                username={user.username}
                isOwnProfile={isOwnProfile}
                token={token}
              />
            </Suspense>
          </div>
        </aside>

        <div
          id="profile-card__thoughts"
          className="col-span-1 lg:col-span-3 space-y-4"
        >
          <Tabs key={user.id.toString()} defaultValue="thoughts">
            <TabsList className="mb-4">
              <TabsTrigger value="thoughts">Thoughts</TabsTrigger>
              {isOwnProfile && (
                <TabsTrigger value="federation">Requests</TabsTrigger>
              )}
            </TabsList>
            <TabsContent value="thoughts" className="space-y-4">
              <UserThoughtsList
                username={username}
                initialThoughts={thoughts}
                totalPages={totalPages}
                me={me}
              />
            </TabsContent>
            {isOwnProfile && (
              <TabsContent value="federation">
                <PendingRequests />
              </TabsContent>
            )}
          </Tabs>
        </div>
      </main>
    </div>
  );
}
