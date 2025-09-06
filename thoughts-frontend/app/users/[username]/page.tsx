import { getMe, getUserProfile, getUserThoughts, Me } from "@/lib/api";
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

  const [userResult, thoughtsResult, meResult] = await Promise.allSettled([
    userProfilePromise,
    thoughtsPromise,
    mePromise,
  ]);

  if (userResult.status === "rejected") {
    notFound();
  }

  const user = userResult.value;
  const me = meResult.status === "fulfilled" ? (meResult.value as Me) : null;

  const thoughts =
    thoughtsResult.status === "fulfilled" ? thoughtsResult.value.thoughts : [];
  const { topLevelThoughts, repliesByParentId } = buildThoughtThreads(thoughts);

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

      <main className="container mx-auto max-w-3xl p-4 -mt-16 grid grid-cols-1 md:grid-cols-3 gap-8">
        <aside className="md:col-span-1 space-y-6 pt-24">
          <TopFriends usernames={user.topFriends} />
        </aside>
        <div className="md:col-span-2 mt-8 md:mt-0 space-y-4">
          <Card className="p-6 bg-card/80 backdrop-blur-lg">
            <div className="flex justify-between items-start">
              <div className="flex items-end gap-4">
                <div className="w-24 h-24 rounded-full border-4 border-background shrink-0">
                  <UserAvatar src={user.avatarUrl} alt={user.displayName} />
                </div>
                <div>
                  <h1 className="text-2xl font-bold">
                    {user.displayName || user.username}
                  </h1>
                  <p className="text-sm text-muted-foreground">
                    @{user.username}
                  </p>
                </div>
              </div>

              <div>
                {isOwnProfile ? (
                  <Button asChild variant="outline">
                    <Link href="/settings/profile">
                      <Settings className="mr-2 h-4 w-4" />
                      Settings
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

            <p className="mt-4 whitespace-pre-wrap">{user.bio}</p>
            <div className="flex items-center gap-2 mt-4 text-sm text-muted-foreground">
              <Calendar className="h-4 w-4" />
              <span>Joined {new Date(user.joinedAt).toLocaleDateString()}</span>
            </div>
          </Card>

          {/* Thoughts Feed */}
          <div className="mt-8 space-y-4">
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
              <p className="text-center text-muted-foreground pt-8">
                Your feed is empty. Follow some users to see their thoughts
                here!
              </p>
            )}
          </div>
        </div>
      </main>
    </div>
  );
}
