import {
  getFollowersList,
  getFollowingList,
  getMe,
  getUserProfile,
  getUserThoughts,
  Me,
} from "@/lib/api";
import { UserAvatar } from "@/components/user-avatar";
import { Calendar, Settings } from "lucide-react";
import { Card } from "@/components/ui/card";
import { notFound } from "next/navigation";
import { cookies } from "next/headers";
import { FollowButton } from "@/components/follow-button";
import { TopFriends } from "@/components/top-friends";
import { buildThoughtThreads } from "@/lib/utils";
import { ThoughtThread } from "@/components/thought-thread";
import { Button } from "@/components/ui/button";
import Link from "next/link";

interface ProfilePageProps {
  params: { username: string };
}

export default async function ProfilePage({ params }: ProfilePageProps) {
  const { username } = params;
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

  const thoughts =
    thoughtsResult.status === "fulfilled" ? thoughtsResult.value.thoughts : [];
  const { topLevelThoughts, repliesByParentId } = buildThoughtThreads(thoughts);

  const followersCount =
    followersResult.status === "fulfilled"
      ? followersResult.value.users.length
      : 0;
  const followingCount =
    followingResult.status === "fulfilled"
      ? followingResult.value.users.length
      : 0;

  const isOwnProfile = me?.username === user.username;
  const isFollowing =
    me?.following?.some(
      (followedUser) => followedUser.username === user.username
    ) || false;

  const authorDetails = new Map<string, { avatarUrl?: string | null }>();
  authorDetails.set(user.username, { avatarUrl: user.avatarUrl });

  return (
    <div>
      {user.customCss && (
        <style dangerouslySetInnerHTML={{ __html: user.customCss }} />
      )}

      <div
        className="h-48 bg-gray-200 bg-cover bg-center profile-header"
        style={{
          backgroundImage: user.headerUrl ? `url(${user.headerUrl})` : "none",
        }}
      />

      <main className="container mx-auto max-w-6xl p-4 -mt-16 grid grid-cols-1 lg:grid-cols-4 gap-8">
        {/* Left Sidebar (Profile Card & Top Friends) */}
        <aside className="col-span-1 lg:col-span-1 space-y-6">
          <div className="sticky top-20 space-y-6">
            <Card className="p-6 bg-card/80 backdrop-blur-lg">
              <div className="flex justify-between items-start">
                <div className="flex items-end gap-4">
                  <div className="w-24 h-24 rounded-full border-4 border-background shrink-0">
                    <UserAvatar src={user.avatarUrl} alt={user.displayName} />
                  </div>
                </div>
                {/* Action Button */}
                <div>
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

              <div className="mt-4">
                <h1 className="text-2xl font-bold">
                  {user.displayName || user.username}
                </h1>
                <p className="text-sm text-muted-foreground">
                  @{user.username}
                </p>
              </div>

              <p className="mt-4 text-sm whitespace-pre-wrap">{user.bio}</p>

              {isOwnProfile && (
                <div className="flex items-center gap-4 mt-4 text-sm">
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

              <div className="flex items-center gap-2 mt-4 text-sm text-muted-foreground">
                <Calendar className="h-4 w-4" />
                <span>
                  Joined {new Date(user.joinedAt).toLocaleDateString()}
                </span>
              </div>
            </Card>

            <TopFriends usernames={user.topFriends} />
          </div>
        </aside>

        <div className="col-span-1 lg:col-span-3 space-y-4">
          {topLevelThoughts.map((thought) => (
            <ThoughtThread
              key={thought.id}
              thought={thought}
              repliesByParentId={repliesByParentId}
              authorDetails={authorDetails}
              currentUser={me}
            />
          ))}
          {topLevelThoughts.length === 0 && (
            <Card className="flex items-center justify-center h-48">
              <p className="text-center text-muted-foreground">
                This user hasn&apos;t posted any public thoughts yet.
              </p>
            </Card>
          )}
        </div>
      </main>
    </div>
  );
}
