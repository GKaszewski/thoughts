import { getMe, getUserProfile, getUserThoughts } from "@/lib/api";
import { UserAvatar } from "@/components/user-avatar";
import { ThoughtCard } from "@/components/thought-card";
import { Calendar } from "lucide-react";
import { Card } from "@/components/ui/card";
import { notFound } from "next/navigation";
import { cookies } from "next/headers";
import { FollowButton } from "@/components/follow-button";
import { TopFriends } from "@/components/top-friends";

interface ProfilePageProps {
  params: { username: string };
}

export default async function ProfilePage({ params }: ProfilePageProps) {
  const { username } = params;
  const token = (await cookies()).get("auth_token")?.value ?? null;

  // Fetch data in parallel
  const userProfilePromise = getUserProfile(username, token);
  const thoughtsPromise = getUserThoughts(username, token);
  // Fetch the logged-in user's data (if they exist)
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
  const thoughts =
    thoughtsResult.status === "fulfilled" ? thoughtsResult.value.thoughts : [];
  const me = meResult.status === "fulfilled" ? meResult.value : null;

  // *** SIMPLIFIED LOGIC ***
  // The follow status is now directly available from the `me` object.
  const isOwnProfile = me?.username === user.username;
  const isFollowing =
    me?.following?.some(
      (followedUser) => followedUser.username === user.username
    ) || false;

  return (
    <div>
      {/* Custom CSS Injection */}
      {user.customCss && (
        <style dangerouslySetInnerHTML={{ __html: user.customCss }} />
      )}

      {/* Header Image */}
      <div
        className="h-48 bg-gray-200 bg-cover bg-center"
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

              {/* Render the FollowButton if it's not the user's own profile */}
              {!isOwnProfile && token && (
                <FollowButton
                  username={user.username}
                  isInitiallyFollowing={isFollowing}
                />
              )}
            </div>

            <p className="mt-4 whitespace-pre-wrap">{user.bio}</p>
            <div className="flex items-center gap-2 mt-4 text-sm text-muted-foreground">
              <Calendar className="h-4 w-4" />
              <span>Joined {new Date(user.joinedAt).toLocaleDateString()}</span>
            </div>
          </Card>

          {/* Thoughts Feed */}
          <div className="mt-8 space-y-4">
            {thoughts.map((thought) => (
              <ThoughtCard
                key={thought.id}
                thought={thought}
                author={{ username: user.username, avatarUrl: user.avatarUrl }}
              />
            ))}
            {thoughts.length === 0 && (
              <p className="text-center text-muted-foreground">
                This user hasn&apos;t posted any thoughts yet.
              </p>
            )}
          </div>
        </div>
      </main>
    </div>
  );
}
