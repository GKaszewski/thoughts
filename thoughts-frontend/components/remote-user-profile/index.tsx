"use client";

import { useState } from "react";
import { UserMinus, UserPlus } from "lucide-react";
import { followUser, unfollowUser, RemoteActor, Thought, Me } from "@/lib/api";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ThoughtList } from "@/components/thought-list";
import { toast } from "sonner";
import { useAuth } from "@/hooks/use-auth";
import { ProfileCard } from "./profile-card";
import { Connections } from "./connections";

interface RemoteUserProfileProps {
  actor: RemoteActor;
  initialPosts: Thought[];
  me: Me | null;
  initialFollowed?: boolean;
}

export function RemoteUserProfile({
  actor,
  initialPosts,
  me,
  initialFollowed = false,
}: RemoteUserProfileProps) {
  const [followed, setFollowed] = useState(initialFollowed);
  const [followLoading, setFollowLoading] = useState(false);
  const { token } = useAuth();

  const [followersActive, setFollowersActive] = useState(false);
  const [followingActive, setFollowingActive] = useState(false);

  const handleFollow = async () => {
    if (!token) {
      toast.error("You must be logged in to follow users.");
      return;
    }
    setFollowLoading(true);
    try {
      if (followed) {
        await unfollowUser(actor.handle, token);
        setFollowed(false);
      } else {
        await followUser(actor.handle, token);
        setFollowed(true);
        toast.success(`Follow request sent to ${actor.handle}`);
      }
    } catch {
      toast.error(followed ? "Failed to unfollow." : "Failed to send follow request.");
    } finally {
      setFollowLoading(false);
    }
  };

  const handleTabChange = (tab: string) => {
    if (tab === "followers") setFollowersActive(true);
    if (tab === "following") setFollowingActive(true);
  };

  const isOwnProfile = me?.username === actor.handle;

  const followButton =
    !isOwnProfile && token ? (
      <Button
        onClick={handleFollow}
        disabled={followLoading}
        variant={followed ? "secondary" : "default"}
        size="sm"
      >
        {followed ? (
          <>
            <UserMinus className="mr-2 h-4 w-4" /> Unfollow
          </>
        ) : (
          <>
            <UserPlus className="mr-2 h-4 w-4" /> Follow
          </>
        )}
      </Button>
    ) : undefined;

  return (
    <div>
      <div
        className="h-48 bg-muted bg-cover bg-center"
        style={{
          backgroundImage: actor.bannerUrl ? `url(${actor.bannerUrl})` : "none",
        }}
      />

      <main className="container mx-auto max-w-6xl p-4 -mt-16 grid grid-cols-1 lg:grid-cols-4 gap-8">
        <aside className="col-span-1 space-y-6">
          <div className="sticky top-20 space-y-6">
            <Card className="p-6 bg-card/80 backdrop-blur-lg">
              <ProfileCard actor={actor} action={followButton} />
            </Card>
          </div>
        </aside>

        <div className="col-span-1 lg:col-span-3">
          <Tabs defaultValue="posts" onValueChange={handleTabChange}>
            <TabsList>
              <TabsTrigger value="posts">Posts</TabsTrigger>
              <TabsTrigger value="followers">Followers</TabsTrigger>
              <TabsTrigger value="following">Following</TabsTrigger>
            </TabsList>

            <TabsContent value="posts" className="space-y-4 mt-4">
              {initialPosts.length > 0 ? (
                <ThoughtList thoughts={initialPosts} currentUser={me} />
              ) : (
                <Card className="flex items-center justify-center h-48">
                  <p className="text-center text-muted-foreground">
                    Posts are being fetched — check back soon.
                  </p>
                </Card>
              )}
            </TabsContent>

            <TabsContent value="followers" className="mt-4">
              <Connections
                handle={actor.handle}
                token={token}
                type="followers"
                active={followersActive}
              />
            </TabsContent>

            <TabsContent value="following" className="mt-4">
              <Connections
                handle={actor.handle}
                token={token}
                type="following"
                active={followingActive}
              />
            </TabsContent>
          </Tabs>
        </div>
      </main>
    </div>
  );
}
