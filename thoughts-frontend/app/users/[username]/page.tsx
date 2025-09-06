import { getUserProfile, getUserThoughts } from "@/lib/api";
import { UserAvatar } from "@/components/user-avatar";
import { ThoughtCard } from "@/components/thought-card";
import { Calendar } from "lucide-react";
import { Card } from "@/components/ui/card";
import { notFound } from "next/navigation";
import { cookies } from "next/headers";

interface ProfilePageProps {
  params: { username: string };
}

export default async function ProfilePage({ params }: ProfilePageProps) {
  const { username } = params;
  const token = (await cookies()).get("auth_token")?.value ?? null;

  // Fetch data directly on the server.
  // The `loading.tsx` file will be shown to the user during this fetch.
  const [userResult, thoughtsResult] = await Promise.allSettled([
    getUserProfile(username, token),
    getUserThoughts(username, token),
  ]);

  // Handle errors from the server-side fetch
  if (userResult.status === "rejected") {
    // If the user isn't found, render the Next.js 404 page
    notFound();
  }

  const user = userResult.value;
  const thoughts =
    thoughtsResult.status === "fulfilled" ? thoughtsResult.value.thoughts : [];

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

      <main className="container mx-auto max-w-3xl p-4 -mt-16">
        {/* Profile Info */}
        <Card className="p-6 bg-card/80 backdrop-blur-lg">
          <div className="flex items-end gap-4">
            <div className="w-24 h-24 rounded-full border-4 border-background">
              <UserAvatar src={user.avatarUrl} alt={user.displayName} />
            </div>
            <div>
              <h1 className="text-2xl font-bold">
                {user.displayName || user.username}
              </h1>
              <p className="text-sm text-muted-foreground">@{user.username}</p>
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
      </main>
    </div>
  );
}
