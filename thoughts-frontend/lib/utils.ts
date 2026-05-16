import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"
import { Thought, ThoughtThread as ThoughtThreadType } from "./api";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** Construct a full fediverse handle like `user@instance.social`.
 *  Falls back gracefully for existing DB rows that only stored the username. */
export function fullFediverseHandle(handle: string, actorUrl: string): string {
  if (handle.includes("@")) return handle;
  try {
    return `${handle}@${new URL(actorUrl).hostname}`;
  } catch {
    return handle;
  }
}

export function buildThoughtThreads(thoughts: Thought[]): ThoughtThreadType[] {
  const thoughtMap = new Map<string, Thought>();
  thoughts.forEach((t) => thoughtMap.set(t.id, t));

  const threads: ThoughtThreadType[] = [];
  const repliesMap: Record<string, Thought[]> = {};

  thoughts.forEach((thought) => {
    if (thought.replyToId) {
      if (!repliesMap[thought.replyToId]) {
        repliesMap[thought.replyToId] = [];
      }
      repliesMap[thought.replyToId].push(thought);
    }
  });

  function buildThread(thought: Thought): ThoughtThreadType {
    return {
      ...thought,
      replies: (repliesMap[thought.id] || []).map(buildThread),
    };
  }

  thoughts.forEach((thought) => {
    if (!thought.replyToId) {
      threads.push(buildThread(thought));
    }
  });

  return threads;
}