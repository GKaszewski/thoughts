"use client"

import { useOptimistic } from "react"
import { followUser, unfollowUser } from "@/app/actions/social"
import { Button } from "@/components/ui/button"
import { toast } from "sonner"
import { UserPlus, UserMinus } from "lucide-react"

interface FollowButtonProps {
  username: string
  isInitiallyFollowing: boolean
}

export function FollowButton({ username, isInitiallyFollowing }: FollowButtonProps) {
  const [optimisticFollowing, setOptimisticFollowing] = useOptimistic(isInitiallyFollowing)

  async function handleClick() {
    const next = !optimisticFollowing
    setOptimisticFollowing(next)
    try {
      await (next ? followUser(username) : unfollowUser(username))
    } catch {
      setOptimisticFollowing(!next) // revert
      toast.error(`Failed to ${next ? "follow" : "unfollow"} user.`)
    }
  }

  return (
    <Button
      onClick={handleClick}
      variant={optimisticFollowing ? "secondary" : "default"}
      data-following={optimisticFollowing}
    >
      {optimisticFollowing ? (
        <><UserMinus className="mr-2 h-4 w-4" /> Unfollow</>
      ) : (
        <><UserPlus className="mr-2 h-4 w-4" /> Follow</>
      )}
    </Button>
  )
}
