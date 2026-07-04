"use client"

import { useOptimistic, useRef } from "react"
import { useRouter } from "next/navigation"
import { followUser, unfollowUser } from "@/lib/api"
import { useAuth } from "@/hooks/use-auth"
import { Button } from "@/components/ui/button"
import { toast } from "sonner"
import { UserPlus, UserMinus } from "lucide-react"

interface FollowButtonProps {
  username: string
  isInitiallyFollowing: boolean
}

const BURST_COLORS = ["#2563eb", "#06b6d4", "#10b981", "#f59e0b", "#a855f7", "#ef4444"]

function burstParticles(canvas: HTMLCanvasElement) {
  const ctx = canvas.getContext("2d")
  if (!ctx) return

  const cx = canvas.width / 2
  const cy = canvas.height / 2
  const particles = Array.from({ length: 14 }, (_, i) => {
    const angle = (i / 14) * Math.PI * 2
    const speed = 2.5 + Math.random() * 2
    return {
      x: cx,
      y: cy,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed,
      r: 3 + Math.random() * 3,
      color: BURST_COLORS[i % BURST_COLORS.length],
      life: 1,
    }
  })

  let rafId: number

  function frame() {
    if (!canvas.isConnected) {
      cancelAnimationFrame(rafId)
      return
    }
    ctx!.clearRect(0, 0, canvas.width, canvas.height)
    let alive = false
    for (const p of particles) {
      p.x += p.vx
      p.y += p.vy
      p.vy += 0.08
      p.life -= 0.03
      if (p.life > 0) {
        alive = true
        ctx!.globalAlpha = p.life
        ctx!.fillStyle = p.color
        ctx!.beginPath()
        ctx!.arc(p.x, p.y, p.r, 0, Math.PI * 2)
        ctx!.fill()
      }
    }
    ctx!.globalAlpha = 1
    if (alive) {
      rafId = requestAnimationFrame(frame)
    }
  }

  rafId = requestAnimationFrame(frame)
}

export function FollowButton({ username, isInitiallyFollowing }: FollowButtonProps) {
  const [optimisticFollowing, setOptimisticFollowing] = useOptimistic(isInitiallyFollowing)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const { token } = useAuth()
  const router = useRouter()

  async function handleClick() {
    if (!token) return
    const next = !optimisticFollowing
    setOptimisticFollowing(next)

    if (next && canvasRef.current) {
      burstParticles(canvasRef.current)
    }

    try {
      await (next ? followUser(username, token) : unfollowUser(username, token))
      router.refresh()
    } catch {
      setOptimisticFollowing(!next)
      toast.error(`Failed to ${next ? "follow" : "unfollow"} user.`)
    }
  }

  return (
    <div className="relative inline-block">
      <canvas
        ref={canvasRef}
        width={160}
        height={80}
        className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 pointer-events-none"
        aria-hidden
      />
      <Button
        onClick={handleClick}
        variant={optimisticFollowing ? "secondary" : "default"}
        className="relative rounded-full"
        data-following={optimisticFollowing}
      >
        {optimisticFollowing ? (
          <><UserMinus className="mr-2 h-4 w-4" /> Unfollow</>
        ) : (
          <><UserPlus className="mr-2 h-4 w-4" /> Follow</>
        )}
      </Button>
    </div>
  )
}
