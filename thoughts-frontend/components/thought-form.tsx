"use client"

import { useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import {
  Form,
  FormField,
  FormItem,
  FormControl,
  FormMessage,
} from "@/components/ui/form"
import { Textarea } from "@/components/ui/textarea"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { CreateThoughtSchema, type Me } from "@/lib/api"
import { useAuth } from "@/hooks/use-auth"
import { toast } from "sonner"
import { Globe, Lock, Users } from "lucide-react"
import { useState } from "react"
import { Confetti } from "./confetti"
import { createThought } from "@/app/actions/thoughts"

const DEFAULT_MOODS = [
  "relaxed 😌", "happy 😊", "excited 🤩", "grateful 🙏", "inspired ✨",
  "thoughtful 🤔", "curious 🧐", "amused 😄", "proud 💪", "hopeful 🌟",
  "tired 😴", "stressed 😰", "anxious 😟", "sad 😢", "frustrated 😤",
  "angry 😠", "bored 😑", "confused 😕", "nostalgic 🥹", "silly 🤪",
]

interface ThoughtFormProps {
  /** Set to the parent thought ID when composing a reply. */
  replyToId?: string
  /** Called after successful submit (e.g. close the reply panel). */
  onSuccess?: () => void
  /** Whether to wrap in a Card. Defaults to true when no replyToId. */
  card?: boolean
  currentUser?: Me | null
}

export function ThoughtForm({ replyToId, onSuccess, card = !replyToId, currentUser }: ThoughtFormProps) {
  const { token } = useAuth()
  const [showConfetti, setShowConfetti] = useState(false)

  const allMoods = [
    ...DEFAULT_MOODS,
    ...(currentUser?.customMoods ?? [])
      .filter(m => !DEFAULT_MOODS.some(d => d.toLowerCase().startsWith(m.name.toLowerCase())))
      .map(m => `${m.name} ${m.value}`),
  ]

  const form = useForm<z.infer<typeof CreateThoughtSchema>>({
    resolver: zodResolver(CreateThoughtSchema),
    defaultValues: {
      content: "",
      visibility: "public",
      ...(replyToId ? { inReplyToId: replyToId } : {}),
    },
  })

  async function onSubmit(values: z.infer<typeof CreateThoughtSchema>) {
    if (!token) {
      toast.error("You must be logged in.")
      return
    }
    try {
      await createThought(values)
      toast.success(replyToId ? "Reply posted!" : "Thought posted!")
      setShowConfetti(true)
      form.reset()
      onSuccess?.()
    } catch {
      toast.error(replyToId ? "Failed to post reply." : "Failed to post thought.")
    }
  }

  const inner = (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="content"
          render={({ field }) => (
            <FormItem>
              <FormControl>
                <Textarea
                  placeholder={replyToId ? "Post your reply..." : "What's on your mind?"}
                  className={`resize-none ${replyToId ? "bg-white shadow-fa-sm" : ""}`}
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <div className={`flex ${replyToId ? "justify-end gap-2" : "justify-between items-center"}`}>
          <div className="flex gap-2">
            {!replyToId && (
              <FormField
                control={form.control}
                name="visibility"
                render={({ field }) => (
                  <Select onValueChange={field.onChange} defaultValue={field.value}>
                    <FormControl>
                      <SelectTrigger className="w-[150px]">
                        <SelectValue placeholder="Visibility" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value="public">
                        <div className="flex items-center gap-2"><Globe className="h-4 w-4" /> Public</div>
                      </SelectItem>
                      <SelectItem value="followers">
                        <div className="flex items-center gap-2"><Users className="h-4 w-4" /> Followers</div>
                      </SelectItem>
                      <SelectItem value="unlisted">
                        <div className="flex items-center gap-2"><Lock className="h-4 w-4" /> Unlisted</div>
                      </SelectItem>
                      <SelectItem value="direct">
                        <div className="flex items-center gap-2"><Lock className="h-4 w-4" /> Direct</div>
                      </SelectItem>
                    </SelectContent>
                  </Select>
                )}
              />
            )}
            <FormField
              control={form.control}
              name="mood"
              render={({ field }) => (
                <Select onValueChange={(v) => field.onChange(v === "__none__" ? undefined : v)} value={field.value ?? "__none__"}>
                  <FormControl>
                    <SelectTrigger className="w-[170px]">
                      <SelectValue placeholder="How are you feeling?" />
                    </SelectTrigger>
                  </FormControl>
                  <SelectContent>
                    <SelectItem value="__none__">No mood</SelectItem>
                    {allMoods.map((mood) => (
                      <SelectItem key={mood} value={mood}>{mood}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}
            />
            {replyToId && (
              <Button type="button" variant="ghost" onClick={() => onSuccess?.()}>
                Cancel
              </Button>
            )}
          </div>
          <Button type="submit" disabled={form.formState.isSubmitting}>
            {form.formState.isSubmitting
              ? (replyToId ? "Replying..." : "Posting...")
              : (replyToId ? "Reply" : "Post Thought")}
          </Button>
        </div>
      </form>
    </Form>
  )

  return (
    <>
      <Confetti fire={showConfetti} onComplete={() => setShowConfetti(false)} />
      {card
        ? <Card><CardContent className="p-4">{inner}</CardContent></Card>
        : <div className="space-y-2 p-4">{inner}</div>
      }
    </>
  )
}
