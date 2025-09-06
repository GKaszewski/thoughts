import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"
import { Thought } from "./api";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function buildThoughtThreads(allThoughts: Thought[]) {
  const repliesByParentId = new Map<string, Thought[]>();
  const topLevelThoughts: Thought[] = [];

  // 1. Group all thoughts into top-level posts or replies
  for (const thought of allThoughts) {
    if (thought.replyToId) {
      // It's a reply, group it with its parent
      const replies = repliesByParentId.get(thought.replyToId) || [];
      replies.push(thought);
      repliesByParentId.set(thought.replyToId, replies);
    } else {
      // It's a top-level thought
      topLevelThoughts.push(thought);
    }
  }

  // 2. Sort top-level thoughts by date, newest first
  topLevelThoughts.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());

  // 3. Sort replies within each thread by date, oldest first for conversational flow
  for (const replies of repliesByParentId.values()) {
    replies.sort((a, b) => new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime());
  }

  return { topLevelThoughts, repliesByParentId };
}