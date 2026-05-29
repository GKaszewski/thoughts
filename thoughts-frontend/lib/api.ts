import { cache } from "react";
import { z } from "zod";

export const UserSchema = z.object({
  id: z.string().uuid(),
  username: z.string(),
  displayName: z.string().nullable(),
  bio: z.string().nullable(),
  avatarUrl: z.string().nullable(),
  headerUrl: z.string().nullable(),
  customCss: z.string().nullable(),
  local: z.boolean(),
  isFollowedByViewer: z.boolean(),
  joinedAt: z.coerce.date().nullable(),
});

export const MeSchema = UserSchema;

export const ProfileFieldSchema = z.object({
  name: z.string(),
  value: z.string(),
});
export type ProfileField = z.infer<typeof ProfileFieldSchema>;

export const RemoteActorSchema = z.object({
  handle: z.string(),
  displayName: z.string().nullable(),
  avatarUrl: z.string().nullable(),
  url: z.string(),
  bio: z.string().nullable(),
  bannerUrl: z.string().nullable(),
  alsoKnownAs: z.array(z.string()),
  outboxUrl: z.string().nullable(),
  followersUrl: z.string().nullable(),
  followingUrl: z.string().nullable(),
  attachment: z.array(ProfileFieldSchema),
});
export type RemoteActor = z.infer<typeof RemoteActorSchema>;

export const ThoughtSchema = z.object({
  id: z.string().uuid(),
  content: z.string(),
  author: UserSchema,
  replyToId: z.string().uuid().nullable(),
  replyToUrl: z.string().url().nullable().optional(),
  visibility: z.string(),
  contentWarning: z.string().nullable(),
  sensitive: z.boolean(),
  likeCount: z.number(),
  boostCount: z.number(),
  replyCount: z.number(),
  likedByViewer: z.boolean(),
  boostedByViewer: z.boolean(),
  createdAt: z.coerce.date(),
  updatedAt: z.coerce.date().nullable(),
  noteExtensions: z.record(z.string(), z.unknown()).nullish(),
});

export const RegisterSchema = z.object({
  username: z.string().min(3),
  email: z.string().email(),
  password: z.string().min(6),
});

export const LoginSchema = z.object({
  email: z.string().email(),
  password: z.string().min(6),
});

export const CreateThoughtSchema = z.object({
  content: z.string().min(1).max(128),
  visibility: z.enum(["public", "followers", "unlisted", "direct"]).optional(),
  inReplyToId: z.string().uuid().optional(),
});

export const UpdateProfileSchema = z.object({
  displayName: z.string().max(50).optional(),
  bio: z.string().max(4000).optional(),
  customCss: z.string().optional(),
});

export const SearchResultsSchema = z.object({
  query: z.string(),
  thoughts: z.array(ThoughtSchema),
  users: z.array(UserSchema),
});

export const ApiKeySchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  createdAt: z.coerce.date(),
});

export const ApiKeyResponseSchema = ApiKeySchema.extend({
  key: z.string().optional(),
});

export const ApiKeyListSchema = z.object({
  keys: z.array(ApiKeySchema),
});

export const CreateApiKeySchema = z.object({
  name: z.string().min(1, "Key name cannot be empty."),
});

export const ThoughtThreadSchema: z.ZodType<{
  id: string;
  content: string;
  author: z.infer<typeof UserSchema>;
  replyToId: string | null;
  visibility: string;
  contentWarning: string | null;
  sensitive: boolean;
  likeCount: number;
  boostCount: number;
  replyCount: number;
  likedByViewer: boolean;
  boostedByViewer: boolean;
  createdAt: Date;
  updatedAt: Date | null;
  noteExtensions?: Record<string, unknown> | null;
  replies: ThoughtThread[];
}> = z.object({
  id: z.string().uuid(),
  content: z.string(),
  author: UserSchema,
  replyToId: z.string().uuid().nullable(),
  visibility: z.string(),
  contentWarning: z.string().nullable(),
  sensitive: z.boolean(),
  likeCount: z.number(),
  boostCount: z.number(),
  replyCount: z.number(),
  likedByViewer: z.boolean(),
  boostedByViewer: z.boolean(),
  createdAt: z.coerce.date(),
  updatedAt: z.coerce.date().nullable(),
  noteExtensions: z.record(z.string(), z.unknown()).nullish(),
  replies: z.lazy(() => z.array(ThoughtThreadSchema)),
});

export type User = z.infer<typeof UserSchema>;
export type Me = z.infer<typeof MeSchema>;
export type Thought = z.infer<typeof ThoughtSchema>;
export type ThoughtThread = z.infer<typeof ThoughtThreadSchema>;
export type Register = z.infer<typeof RegisterSchema>;
export type Login = z.infer<typeof LoginSchema>;
export type ApiKey = z.infer<typeof ApiKeySchema>;
export type ApiKeyResponse = z.infer<typeof ApiKeyResponseSchema>;

const API_BASE_URL =
  typeof window === "undefined"
    ? process.env.NEXT_PUBLIC_SERVER_SIDE_API_URL
    : process.env.NEXT_PUBLIC_API_URL;

type ApiFetchOptions = Omit<RequestInit, 'next'> & {
  next?: { tags?: string[]; revalidate?: number | false }
}

async function apiFetch<T>(
  endpoint: string,
  options: ApiFetchOptions = {},
  schema: z.ZodType<T>,
  token?: string | null
): Promise<T> {
  if (!API_BASE_URL) {
    throw new Error("API_BASE_URL is not defined");
  }

  const { next, ...restOptions } = options;

  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(restOptions.headers as Record<string, string>),
  };

  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    ...restOptions,
    headers,
    ...(next ? { next } : {}),
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

// ── Auth ──────────────────────────────────────────────────────────────────

export const registerUser = (data: z.infer<typeof RegisterSchema>) =>
  apiFetch(
    "/auth/register",
    { method: "POST", body: JSON.stringify(data) },
    z.object({ token: z.string(), user: UserSchema })
  );

export const loginUser = (data: z.infer<typeof LoginSchema>) =>
  apiFetch("/auth/login", { method: "POST", body: JSON.stringify(data) }, z.object({ token: z.string() }));

// ── Current user ──────────────────────────────────────────────────────────

export const getMe = (token: string) =>
  apiFetch("/users/me", { next: { tags: ['me'] } }, MeSchema, token);

export const updateProfile = (data: z.infer<typeof UpdateProfileSchema>, token: string) =>
  apiFetch("/users/me", { method: "PATCH", body: JSON.stringify(data) }, UserSchema, token);

async function uploadImage(endpoint: string, file: File, token: string): Promise<User> {
  const base = process.env.NEXT_PUBLIC_API_URL;
  const body = new FormData();
  body.append("file", file);
  const res = await fetch(`${base}${endpoint}`, {
    method: "PUT",
    headers: { Authorization: `Bearer ${token}` },
    body,
  });
  if (!res.ok) throw new Error(`Upload failed: ${res.status}`);
  return UserSchema.parse(await res.json());
}

export const uploadAvatar = (file: File, token: string) =>
  uploadImage("/users/me/avatar", file, token);

export const uploadBanner = (file: File, token: string) =>
  uploadImage("/users/me/banner", file, token);

export const getMeFollowingList = (token: string) =>
  apiFetch("/users/me/following", { next: { tags: ['me'] } }, z.object({ total: z.number(), items: z.array(UserSchema) }), token);

// ── Users ─────────────────────────────────────────────────────────────────

export const getUserProfile = (username: string, token: string | null) =>
  apiFetch(`/users/${username}`, { next: { tags: [`profile:${username}`] } }, UserSchema, token);

export const getFollowersList = (username: string, token: string | null) =>
  apiFetch(`/users/${username}/followers`, { next: { tags: [`profile:${username}`] } }, z.object({ total: z.number(), items: z.array(UserSchema) }), token);

export const getFollowingList = (username: string, token: string | null) =>
  apiFetch(`/users/${username}/following`, { next: { tags: [`profile:${username}`] } }, z.object({ total: z.number(), items: z.array(UserSchema) }), token);

export const getTopFriends = (username: string, token: string | null) =>
  apiFetch(
    `/users/${username}/top-friends`,
    { next: { tags: [`profile:${username}`] } },
    z.object({ topFriends: z.array(UserSchema) }),
    token
  );

export const followUser = (username: string, token: string) =>
  apiFetch(`/users/${username}/follow`, { method: "POST" }, z.null(), token);

export const unfollowUser = (username: string, token: string) =>
  apiFetch(`/users/${username}/follow`, { method: "DELETE" }, z.null(), token);

export const markNotificationRead = (id: string, token: string) =>
  apiFetch(
    `/notifications/${id}`,
    { method: "PATCH", body: JSON.stringify({ read: true }) },
    z.null(),
    token
  );

export const markAllNotificationsRead = (token: string) =>
  apiFetch(
    "/notifications",
    { method: "PATCH", body: JSON.stringify({ read: true }) },
    z.null(),
    token
  );

export const lookupRemoteActor = cache((handle: string, token: string | null) =>
  apiFetch(
    `/users/lookup?handle=${encodeURIComponent(handle)}`,
    { next: { tags: [`remote-actor:${handle}`] } },
    RemoteActorSchema,
    token
  )
);

export const getRemoteActorPosts = (
  handle: string,
  page: number,
  token: string | null
) =>
  apiFetch(
    `/federation/actors/${encodeURIComponent(handle)}/posts?page=${page}&per_page=20`,
    { next: { tags: [`remote-actor:${handle}`] } },
    z.object({
      total: z.number(),
      page: z.number(),
      per_page: z.number(),
      items: z.array(ThoughtSchema),
    }),
    token
  );

export const ActorConnectionSchema = z.object({
  handle: z.string(),
  displayName: z.string().nullable(),
  avatarUrl: z.string().nullable(),
  url: z.string(),
});
export type ActorConnection = z.infer<typeof ActorConnectionSchema>;

const ActorConnectionPageSchema = z.object({
  items: z.array(ActorConnectionSchema),
  page: z.number(),
  hasMore: z.boolean(),
});

export const getActorFollowers = (
  handle: string,
  page: number,
  token: string | null
) =>
  apiFetch(
    `/federation/actors/${encodeURIComponent(handle)}/followers-list?page=${page}`,
    { next: { tags: [`remote-actor:${handle}`] } },
    ActorConnectionPageSchema,
    token
  );

export const getActorFollowing = (
  handle: string,
  page: number,
  token: string | null
) =>
  apiFetch(
    `/federation/actors/${encodeURIComponent(handle)}/following-list?page=${page}`,
    { next: { tags: [`remote-actor:${handle}`] } },
    ActorConnectionPageSchema,
    token
  );

export const getAllUsers = (page: number = 1, pageSize: number = 20) =>
  apiFetch(
    `/users?page=${page}&per_page=${pageSize}`,
    { next: { tags: ['users'] } },
    z.object({ items: z.array(UserSchema), total: z.number(), page: z.number(), per_page: z.number() })
      .transform((d) => ({ ...d, totalPages: Math.ceil(d.total / d.per_page) }))
  );

export const getAllUsersCount = () =>
  apiFetch("/users/count", { next: { tags: ['users'] } }, z.object({ count: z.number() }));

// ── Thoughts ──────────────────────────────────────────────────────────────

export type FeedSortOption =
  | "newest"
  | "oldest"
  | "most_liked"
  | "most_boosted"
  | "most_discussed";

export type FeedOptions = {
  sort?: FeedSortOption;
  originals_only?: boolean;
  replies_only?: boolean;
  local_only?: boolean;
  hide_sensitive?: boolean;
};

export const getFeed = (token: string, page = 1, pageSize = 20, opts: FeedOptions = {}) => {
  const params = new URLSearchParams({ page: String(page), per_page: String(pageSize) });
  if (opts.sort)           params.set("sort", opts.sort);
  if (opts.originals_only) params.set("originals_only", "true");
  if (opts.replies_only)   params.set("replies_only", "true");
  if (opts.local_only)     params.set("local_only", "true");
  if (opts.hide_sensitive) params.set("hide_sensitive", "true");
  return apiFetch(
    `/feed?${params.toString()}`,
    { next: { tags: ["feed"] } },
    z.object({ items: z.array(ThoughtSchema), total: z.number(), page: z.number(), per_page: z.number() })
      .transform((d) => ({ ...d, totalPages: Math.ceil(d.total / d.per_page) })),
    token
  );
};

export const getUserThoughts = (username: string, token: string | null, page = 1) =>
  apiFetch(
    `/users/${username}/thoughts?page=${page}`,
    { next: { tags: [`profile:${username}`] } },
    z.object({ items: z.array(ThoughtSchema), total: z.number(), page: z.number(), per_page: z.number() }),
    token
  );

export const createThought = (data: z.infer<typeof CreateThoughtSchema>, token: string) =>
  apiFetch("/thoughts", { method: "POST", body: JSON.stringify(data) }, ThoughtSchema, token);

export const deleteThought = (thoughtId: string, token: string) =>
  apiFetch(`/thoughts/${thoughtId}`, { method: "DELETE" }, z.null(), token);

export const getThoughtById = (thoughtId: string, token: string | null) =>
  apiFetch(`/thoughts/${thoughtId}`, { next: { tags: [`thought:${thoughtId}`] } }, ThoughtSchema, token);

export const getThoughtThread = async (thoughtId: string, token: string | null): Promise<ThoughtThread> => {
  const thoughts = await apiFetch(`/thoughts/${thoughtId}/thread`, { next: { tags: [`thought:${thoughtId}`] } }, z.array(ThoughtSchema), token);
  type T = z.infer<typeof ThoughtSchema>;
  const repliesMap: Record<string, T[]> = {};
  for (const t of thoughts) {
    if (t.replyToId) {
      (repliesMap[t.replyToId] ??= []).push(t);
    }
  }
  function build(t: T): ThoughtThread {
    return { ...t, replies: (repliesMap[t.id] ?? []).map(build) };
  }
  const root = thoughts.find((t) => t.id === thoughtId) ?? thoughts[0];
  if (!root) throw new Error("Thread not found");
  return build(root);
};

// ── Tags ──────────────────────────────────────────────────────────────────

export const getThoughtsByTag = (tagName: string, token: string | null) =>
  apiFetch(
    `/tags/${tagName}`,
    { next: { tags: [`tag:${tagName}`, 'feed'] } },
    z.object({ tag: z.string(), items: z.array(ThoughtSchema), total: z.number(), page: z.number(), per_page: z.number() }),
    token
  );

export const getPopularTags = () =>
  apiFetch(
    "/tags/popular",
    { next: { tags: ['tags:popular'] } },
    z.object({ tags: z.array(z.object({ name: z.string(), thought_count: z.number() })) })
      .transform((d) => d.tags.map((t) => t.name))
  );

// ── Search ────────────────────────────────────────────────────────────────

export const search = (query: string, token: string | null) =>
  apiFetch(`/search?q=${encodeURIComponent(query)}`, { next: { tags: ['search'] } }, SearchResultsSchema, token);

// ── API Keys ──────────────────────────────────────────────────────────────

export const getApiKeys = (token: string) =>
  apiFetch("/api-keys", { next: { tags: ['api-keys'] } }, z.object({ keys: z.array(ApiKeySchema) }), token);

export const createApiKey = (data: z.infer<typeof CreateApiKeySchema>, token: string) =>
  apiFetch("/api-keys", { method: "POST", body: JSON.stringify(data) }, ApiKeyResponseSchema, token);

export const deleteApiKey = (keyId: string, token: string) =>
  apiFetch(`/api-keys/${keyId}`, { method: "DELETE" }, z.null(), token);

// ── Legacy alias used by top-friends-combobox ─────────────────────────────

export const getFriends = (token: string) =>
  getMeFollowingList(token).then((r) => ({ users: r.items }));

// ── Federation management ─────────────────────────────────────────────────

export const getPendingFollowRequests = (token: string) =>
  apiFetch(
    "/federation/me/followers/pending",
    { next: { tags: ['federation:pending'] } },
    z.array(RemoteActorSchema),
    token
  );

export const acceptFollowRequest = (actorUrl: string, token: string) =>
  apiFetch(
    "/federation/me/followers/accept",
    { method: "POST", body: JSON.stringify({ actor_url: actorUrl }) },
    z.null(),
    token
  );

export const rejectFollowRequest = (actorUrl: string, token: string) =>
  apiFetch(
    "/federation/me/followers",
    { method: "DELETE", body: JSON.stringify({ actor_url: actorUrl }) },
    z.null(),
    token
  );

export const getRemoteFollowers = (token: string) =>
  apiFetch(
    "/federation/me/followers",
    { next: { tags: ['federation:followers'] } },
    z.array(RemoteActorSchema),
    token
  );

export const getRemoteFollowing = (token: string) =>
  apiFetch(
    "/federation/me/following",
    { next: { tags: ['federation:following'] } },
    z.array(RemoteActorSchema),
    token
  );

// ── Friends ───────────────────────────────────────────────────────────────

export const getMyFriends = (token: string, page = 1, perPage = 50) =>
  apiFetch(
    `/users/me/friends?page=${page}&per_page=${perPage}`,
    { cache: "no-store" },
    z.object({
      items: z.array(UserSchema),
      total: z.number(),
      page: z.number(),
      perPage: z.number(),
    }),
    token
  );

export const getMyRemoteFriends = (token: string) =>
  apiFetch(
    "/federation/me/friends",
    { cache: "no-store" },
    z.array(RemoteActorSchema),
    token
  );

export const setTopFriends = (token: string, friendIds: string[]) =>
  apiFetch(
    "/users/me/top-friends",
    { method: "PUT", body: JSON.stringify({ friendIds }) },
    z.null(),
    token
  );

export const unfollowRemoteActor = (handle: string, token: string) =>
  apiFetch(
    "/federation/me/following",
    { method: "DELETE", body: JSON.stringify({ handle }) },
    z.null(),
    token
  );

export const setAlsoKnownAs = (value: string | null, token: string) =>
  apiFetch(
    "/federation/me/also-known-as",
    { method: "PATCH", body: JSON.stringify({ also_known_as: value || null }) },
    z.null(),
    token
  );
