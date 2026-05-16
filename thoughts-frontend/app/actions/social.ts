"use server";

import { revalidateTag } from "next/cache";
import { cookies } from "next/headers";
import {
  followUser as apiFollowUser,
  unfollowUser as apiUnfollowUser,
} from "@/lib/api";

async function getToken(): Promise<string> {
  const token = (await cookies()).get("auth_token")?.value;
  if (!token) throw new Error("Not authenticated");
  return token;
}

export async function followUser(username: string) {
  const token = await getToken();
  await apiFollowUser(username, token);
  revalidateTag(`profile:${username}`);
  revalidateTag("feed");
}

export async function unfollowUser(username: string) {
  const token = await getToken();
  await apiUnfollowUser(username, token);
  revalidateTag(`profile:${username}`);
  revalidateTag("feed");
}
