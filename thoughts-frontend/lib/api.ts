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

export const MeSchema = z.object({
  id: z.uuid(),
  username: z.string(),
  displayName: z.string().nullable(),
  bio: z.string().nullable(),
  avatarUrl: z.url().nullable(),
  headerUrl: z.url().nullable(),
  customCss: z.string().nullable(),
  topFriends: z.array(z.string()),
  joinedAt: z.coerce.date(),
  following: z.array(UserSchema),
});

export const ThoughtSchema = z.object({
  id: z.uuid(),
  authorUsername: z.string(),
  content: z.string(),
  visibility: z.enum(["Public", "FriendsOnly", "Private"]),
  replyToId: z.uuid().nullable(),
  createdAt: z.coerce.date(),
});

export const RegisterSchema = z.object({
  username: z.string().min(3),
  email: z.email(),
  password: z.string().min(6),
});

export const LoginSchema = z.object({
  username: z.string().min(3),
  password: z.string().min(6),
});

export const CreateThoughtSchema = z.object({
  content: z.string().min(1).max(128),
  visibility: z.enum(["Public", "FriendsOnly", "Private"]).optional(),
  replyToId: z.string().uuid().optional(),
});

export const UpdateProfileSchema = z.object({
  displayName: z.string().max(50).optional(),
  bio: z.string().max(160).optional(),
  avatarUrl: z.url().or(z.literal("")).optional(),
  headerUrl: z.url().or(z.literal("")).optional(),
  customCss: z.string().optional(),
  topFriends: z.array(z.string()).max(8).optional(),
});

export type User = z.infer<typeof UserSchema>;
export type Me = z.infer<typeof MeSchema>;
export type Thought = z.infer<typeof ThoughtSchema>;
export type Register = z.infer<typeof RegisterSchema>;
export type Login = z.infer<typeof LoginSchema>;

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8000";

async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {},
  schema: z.ZodType<T>,
  token?: string | null
): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options.headers as Record<string, string>),
  };

  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    throw new Error(`API request failed with status ${response.status}`);
  }
  
  if (response.status === 204) {
    return null as T;
  }

  const data = await response.json();
  return schema.parse(data);
}

export const registerUser = (data: z.infer<typeof RegisterSchema>) =>
  apiFetch("/auth/register", {
    method: "POST",
    body: JSON.stringify(data),
  }, UserSchema);

export const loginUser = (data: z.infer<typeof LoginSchema>) =>
  apiFetch("/auth/login", {
    method: "POST",
    body: JSON.stringify(data),
  }, z.object({ token: z.string() }));

  export const getFeed = (token: string) =>
  apiFetch(
    "/feed",
    {},
    z.object({ thoughts: z.array(ThoughtSchema) }),
    token
  );

// --- User API Functions ---
export const getUserProfile = (username: string, token: string | null) =>
  apiFetch(`/users/${username}`, {}, UserSchema, token);

export const getUserThoughts = (username: string, token: string | null) =>
  apiFetch(
    `/users/${username}/thoughts`,
    {},
    z.object({ thoughts: z.array(ThoughtSchema) }),
    token
  );

export const createThought = (
  data: z.infer<typeof CreateThoughtSchema>,
  token: string
) =>
  apiFetch(
    "/thoughts",
    {
      method: "POST",
      body: JSON.stringify(data),
    },
    ThoughtSchema,
    token
  );

  export const followUser = (username: string, token: string) =>
  apiFetch(
    `/users/${username}/follow`,
    { method: "POST" },
    z.null(), // Expect a 204 No Content response, which we treat as null
    token
  );

export const unfollowUser = (username: string, token: string) =>
  apiFetch(
    `/users/${username}/follow`,
    { method: "DELETE" },
    z.null(), // Expect a 204 No Content response
    token
  );

  export const getMe = (token: string) =>
  apiFetch("/users/me", {}, MeSchema, token);

  export const getPopularTags = () =>
  apiFetch(
    "/tags/popular",
    {},
    z.array(z.string()) // Expect an array of strings
  );

  export const deleteThought = (thoughtId: string, token: string) =>
  apiFetch(
    `/thoughts/${thoughtId}`,
    { method: "DELETE" },
    z.null(), // Expect a 204 No Content response
    token
  );

export const updateProfile = (
  data: z.infer<typeof UpdateProfileSchema>,
  token: string
) =>
  apiFetch(
    "/users/me",
    {
      method: "PUT",
      body: JSON.stringify(data),
    },
    UserSchema, // Expect the updated user object back
    token
  );

  export const getThoughtsByTag = (tagName: string, token: string | null) =>
  apiFetch(
    `/tags/${tagName}`,
    {},
    z.object({ thoughts: z.array(ThoughtSchema) }),
    token
  );

export const getThoughtById = (thoughtId: string, token: string | null) =>
  apiFetch(
    `/thoughts/${thoughtId}`,
    {},
    ThoughtSchema, // Expect a single thought object
    token
  );

  export const getFollowingList = (username: string, token: string | null) =>
  apiFetch(
    `/users/${username}/following`,
    {},
    z.object({ users: z.array(UserSchema) }),
    token
  );

export const getFollowersList = (username: string, token: string | null) =>
  apiFetch(
    `/users/${username}/followers`,
    {},
    z.object({ users: z.array(UserSchema) }),
    token
  );