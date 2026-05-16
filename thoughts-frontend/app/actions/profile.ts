"use server";

import { revalidateTag } from "next/cache";
import { cookies } from "next/headers";
import { updateProfile as apiUpdateProfile, UpdateProfileSchema } from "@/lib/api";
import { z } from "zod";

async function getToken(): Promise<string> {
  const token = (await cookies()).get("auth_token")?.value;
  if (!token) throw new Error("Not authenticated");
  return token;
}

export async function updateProfile(
  username: string,
  data: z.infer<typeof UpdateProfileSchema>
) {
  const token = await getToken();
  const updated = await apiUpdateProfile(data, token);
  revalidateTag(`profile:${username}`);
  revalidateTag("me");
  return updated;
}
