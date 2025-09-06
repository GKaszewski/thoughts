import { z } from "zod";

export const UserSchema = z.object({
  id: z.uuid(),
  username: z.string(),
  displayName: z.string().nullable(),
  bio: z.string().nullable(),
  avatarUrl: z.url().nullable(),
  headerUrl: z.url().nullable(),
  customCss: z.string().nullable(),
  topFriends: z.array(z.string()),
  joinedAt: z.coerce.date(),
});

export const ThoughtSchema = z.object({
  id: z.uuid(),
  authorUsername: z.string(),
  content: z.string(),
  visibility: z.enum(["Public", "FriendsOnly", "Private"]),
  replyToId: z.uuid().nullable(),
  createdAt: z.string().datetime(),
});

export type User = z.infer<typeof UserSchema>;
export type Thought = z.infer<typeof ThoughtSchema>;

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8000";

async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {},
  schema: z.ZodType<T>
): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`API request failed with status ${response.status}`);
  }

  const data = await response.json();
  return schema.parse(data);
}

// --- User API Functions ---
export const getUserProfile = (username: string) =>
  apiFetch(`/users/${username}`, {}, UserSchema);

export const getUserThoughts = (username: string) =>
  apiFetch(
    `/users/${username}/thoughts`,
    {},
    z.object({ thoughts: z.array(ThoughtSchema) })
  );