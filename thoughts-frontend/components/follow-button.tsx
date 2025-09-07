"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/use-auth";
import { followUser, unfollowUser } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import { UserPlus, UserMinus } from "lucide-react";

interface FollowButtonProps {
  username: string;
  isInitiallyFollowing: boolean;
}

export function FollowButton({
  username,
  isInitiallyFollowing,
}: FollowButtonProps) {
  const [isFollowing, setIsFollowing] = useState(isInitiallyFollowing);
  const [isLoading, setIsLoading] = useState(false);
  const { token } = useAuth();
  const router = useRouter();

  const handleClick = async () => {
    if (!token) {
      toast.error("You must be logged in to follow users.");
      return;
    }

    setIsLoading(true);
    const action = isFollowing ? unfollowUser : followUser;

    try {
      // Optimistic update
      setIsFollowing(!isFollowing);
      await action(username, token);
      router.refresh(); // Re-fetch server component data to get the latest follower count etc.
    } catch {
      // Revert on error
      setIsFollowing(isFollowing);
      toast.error(`Failed to ${isFollowing ? "unfollow" : "follow"} user.`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Button
      onClick={handleClick}
      disabled={isLoading}
      variant={isFollowing ? "secondary" : "default"}
      data-following={isFollowing}
    >
      {isFollowing ? (
        <>
          <UserMinus className="mr-2 h-4 w-4" /> Unfollow
        </>
      ) : (
        <>
          <UserPlus className="mr-2 h-4 w-4" /> Follow
        </>
      )}
    </Button>
  );
}
