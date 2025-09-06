"use client";

import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
} from "@/components/ui/card";
import { UserAvatar } from "./user-avatar";
import { deleteThought, Me, Thought } from "@/lib/api";
import { formatDistanceToNow } from "date-fns";
import { useAuth } from "@/hooks/use-auth";
import { useState } from "react";
import { useRouter } from "next/navigation";
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
import { ReplyForm } from "@/components/reply-form";
import Link from "next/link";

interface ThoughtCardProps {
  thought: Thought;
  author: {
    username: string;
    avatarUrl?: string | null;
  };
  currentUser: Me | null;
  isReply?: boolean;
}

export function ThoughtCard({
  thought,
  author,
  currentUser,
  isReply = false,
}: ThoughtCardProps) {
  const [isAlertOpen, setIsAlertOpen] = useState(false);
  const [isReplyOpen, setIsReplyOpen] = useState(false);
  const { token } = useAuth();
  const router = useRouter();
  const timeAgo = formatDistanceToNow(new Date(thought.createdAt), {
    addSuffix: true,
  });

  const isAuthor = currentUser?.username === thought.authorUsername;

  const handleDelete = async () => {
    if (!token) return;
    try {
      await deleteThought(thought.id, token);
      toast.success("Thought deleted successfully.");
      router.refresh();
    } catch (error) {
      console.error("Failed to delete thought:", error);
      toast.error("Failed to delete thought.");
    } finally {
      setIsAlertOpen(false);
    }
  };

  return (
    <>
      <div
        id={thought.id}
        className={!isReply ? "bg-card rounded-xl border shadow-sm" : ""}
      >
        {thought.replyToId && isReply && (
          <div className="px-4 pt-2 text-sm text-muted-foreground flex items-center gap-2">
            <CornerUpLeft className="h-4 w-4" />
            <span>
              Replying to{" "}
              <Link
                href={`#${thought.replyToId}`}
                className="hover:underline text-primary"
              >
                parent thought
              </Link>
            </span>
          </div>
        )}
      </div>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0">
          <Link
            href={`/users/${author.username}`}
            className="flex items-center gap-4"
          >
            <UserAvatar src={author.avatarUrl} alt={author.username} />
            <div className="flex flex-col">
              <span className="font-bold">{author.username}</span>
              <span className="text-sm text-muted-foreground">{timeAgo}</span>
            </div>
          </Link>
          {isAuthor && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button className="p-2 rounded-full hover:bg-accent">
                  <MoreHorizontal className="h-4 w-4" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent>
                <DropdownMenuItem
                  className="text-destructive"
                  onSelect={() => setIsAlertOpen(true)}
                >
                  <Trash2 className="mr-2 h-4 w-4" />
                  Delete
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          )}
        </CardHeader>
        <CardContent>
          <p className="whitespace-pre-wrap break-words">{thought.content}</p>
        </CardContent>

        {token && (
          <CardFooter className="border-t px-4 pt-2 pb-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsReplyOpen(!isReplyOpen)}
            >
              <MessageSquare className="mr-2 h-4 w-4" />
              Reply
            </Button>
          </CardFooter>
        )}

        {isReplyOpen && (
          <div className="border-t p-4">
            <ReplyForm
              parentThoughtId={thought.id}
              onReplySuccess={() => setIsReplyOpen(false)}
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
