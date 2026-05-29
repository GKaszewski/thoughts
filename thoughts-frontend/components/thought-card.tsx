"use client";

import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
} from "@/components/ui/card";
import { UserAvatar } from "./user-avatar";
import { Me, Thought } from "@/lib/api";
import { deleteThought } from "@/app/actions/thoughts";
import { format, formatDistanceToNow } from "date-fns";
import { useAuth } from "@/hooks/use-auth";
import { useState } from "react";
import { toast } from "sonner";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import {
  CornerUpLeft,
  MessageSquare,
  MoreHorizontal,
  Trash2,
} from "lucide-react";
import { ThoughtForm } from "@/components/thought-form";
import { MovieCard } from "@/components/movie-card";
import Link from "next/link";
import { cn, profileHref } from "@/lib/utils";

interface ThoughtCardProps {
  thought: Thought;
  currentUser: Me | null;
  isReply?: boolean;
}

function renderWithHashtags(content: string) {
  return content.split(/(#\w+)/g).map((part, i) =>
    /^#\w+$/.test(part) ? (
      <span key={i} className="text-primary font-medium">
        {part}
      </span>
    ) : (
      part
    )
  );
}

export function ThoughtCard({
  thought,
  currentUser,
  isReply = false,
}: ThoughtCardProps) {
  const { author } = thought;
  const [isAlertOpen, setIsAlertOpen] = useState(false);
  const [isReplyOpen, setIsReplyOpen] = useState(false);
  const [deletingState, setDeletingState] = useState<"idle" | "shaking" | "fading">("idle");
  const { token } = useAuth();
  const timeAgo = formatDistanceToNow(new Date(thought.createdAt), {
    addSuffix: true,
  });

  const isAuthor = currentUser?.username === thought.author.username;

  const meta = thought.noteExtensions as Record<string, unknown> | null | undefined;
  if (meta?.movieTitle) {
    return (
      <MovieCard
        meta={meta as unknown as Parameters<typeof MovieCard>[0]["meta"]}
        author={thought.author}
        createdAt={new Date(thought.createdAt)}
      />
    );
  }

  const handleDelete = async () => {
    setIsAlertOpen(false);
    setDeletingState("shaking");
    await new Promise((r) => setTimeout(r, 450));
    setDeletingState("fading");
    await new Promise((r) => setTimeout(r, 300));
    try {
      await deleteThought(thought.id);
      toast.success("Thought deleted.");
    } catch (error) {
      console.error("Failed to delete thought:", error);
      setDeletingState("idle");
      toast.error("Failed to delete thought.");
    }
  };

  return (
    <>
      <div
        id={thought.id}
        className={cn(
          "bg-transparent backdrop-blur-lg shadow-fa-md rounded-xl overflow-hidden glossy-effect bottom",
          isReply
            ? "bg-white/80 glass-effect glossy-effect bottom shadow-fa-sm p-2"
            : ""
        )}
      >
        {thought.replyToId && isReply && (
          <div className="text-sm text-muted-foreground flex items-center gap-2">
            <CornerUpLeft className="h-4 w-4 text-primary/70" />
            <span>
              Replying to{" "}
              <Link
                href={`#${thought.replyToId}`}
                className="hover:underline text-primary text-shadow-sm"
              >
                parent thought
              </Link>
            </span>
          </div>
        )}
        {!thought.replyToId && thought.replyToUrl && (
          <div className="text-sm text-muted-foreground flex items-center gap-2">
            <CornerUpLeft className="h-4 w-4 text-primary/70" />
            <span>
              Replying to{" "}
              <a
                href={thought.replyToUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="hover:underline text-primary text-shadow-sm"
              >
                original post ↗
              </a>
            </span>
          </div>
        )}
      </div>
      <Card
        className={cn(
          "mt-2 transition-transform duration-200 hover:-translate-y-0.5 hover:shadow-fa-lg",
          deletingState === "shaking" && "animate-shake",
          deletingState === "fading" && "animate-fade-out pointer-events-none"
        )}
      >
        <CardHeader className="flex flex-row items-center justify-between space-y-0">
          <Link
            href={profileHref(author.username, author.local)}
            className="flex items-center gap-4 text-shadow-md"
          >
            <UserAvatar
              src={author.avatarUrl}
              alt={author.displayName || author.username}
            />
            <div className="flex flex-col min-w-0">
              <span className="font-bold">
                {author.displayName || author.username}
              </span>
              {!author.local && (
                <div className="flex items-center gap-1.5 min-w-0">
                  <span className="text-xs text-muted-foreground/70 truncate flex-1">
                    {author.username.startsWith("@") ? author.username : `@${author.username}`}
                  </span>
                  <span className="text-[10px] font-medium px-2 py-0.5 rounded-full bg-blue-500/10 border border-blue-500/20 text-blue-500 shrink-0 max-w-[8rem] truncate">
                    {author.username.split("@").filter(Boolean).at(-1)}
                  </span>
                </div>
              )}
              <time
                dateTime={new Date(thought.createdAt).toISOString()}
                title={format(new Date(thought.createdAt), "PPP p")}
                className="text-sm text-muted-foreground text-shadow-sm"
              >
                {timeAgo}
              </time>
            </div>
          </Link>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <button className="p-2 rounded-full hover:bg-accent">
                <MoreHorizontal className="h-4 w-4" />
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              {isAuthor && (
                <DropdownMenuItem
                  className="text-destructive"
                  onSelect={() => setIsAlertOpen(true)}
                >
                  <Trash2 className="mr-2 h-4 w-4" />
                  Delete
                </DropdownMenuItem>
              )}
              <DropdownMenuItem>
                <Link href={`/thoughts/${thought.id}`} className="flex gap-2">
                  <MessageSquare className="mr-2 h-4 w-4" />
                  View
                </Link>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </CardHeader>
        <CardContent>
          {thought.author.local ? (
            <p className="whitespace-pre-wrap break-words text-shadow-sm">
              {renderWithHashtags(thought.content)}
            </p>
          ) : (
            <div
              className="text-sm break-words [&_a]:underline [&_a]:text-primary [&_p]:mb-2 [&_.media-notice]:text-muted-foreground [&_.media-notice]:italic"
              dangerouslySetInnerHTML={{
                __html:
                  thought.content.trim() ||
                  '<p class="media-notice">📎 Media attachment — not supported</p>',
              }}
            />
          )}
          {(thought.mood || (meta?.mood as string | undefined)) && (
            <p className="text-xs text-muted-foreground italic mt-2">
              feeling {thought.mood || (meta?.mood as string)}
            </p>
          )}
        </CardContent>

        {token && (
          <CardFooter className="border-t px-4 pt-2 pb-2 border-border/50">
            <Button
              variant="ghost"
              size="sm"
              className="rounded-full bg-primary/8 border border-primary/15 text-primary hover:bg-primary/15"
              onClick={() => setIsReplyOpen(!isReplyOpen)}
            >
              <MessageSquare className="mr-2 h-4 w-4" />
              Reply
            </Button>
          </CardFooter>
        )}

        {isReplyOpen && (
          <div className="animate-slide-down border-t m-4 rounded-2xl border-border/50 bg-secondary/20">
            <ThoughtForm
              replyToId={thought.id}
              onSuccess={() => setIsReplyOpen(false)}
              currentUser={currentUser}
            />
          </div>
        )}
      </Card>
      <AlertDialog open={isAlertOpen} onOpenChange={setIsAlertOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Are you sure?</AlertDialogTitle>
            <AlertDialogDescription>
              This action cannot be undone. This will permanently delete your
              thought.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDelete}>Delete</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
