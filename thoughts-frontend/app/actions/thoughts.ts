"use server";

import { revalidateTag } from "next/cache";
import { cookies } from "next/headers";
import {
  createThought as apiCreateThought,
  deleteThought as apiDeleteThought,
  CreateThoughtSchema,
} from "@/lib/api";
import { z } from "zod";

async function getToken(): Promise<string> {
  const token = (await cookies()).get("auth_token")?.value;
  if (!token) throw new Error("Not authenticated");
  return token;
}

export async function createThought(data: z.infer<typeof CreateThoughtSchema>) {
  const token = await getToken();
  const thought = await apiCreateThought(data, token);
  revalidateTag("feed");
  return thought;
}

export async function deleteThought(thoughtId: string) {
  const token = await getToken();
  await apiDeleteThought(thoughtId, token);
  revalidateTag("feed");
  revalidateTag(`thought:${thoughtId}`);
}
