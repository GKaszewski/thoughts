// components/top-friends-editor.tsx
"use client";

import { useState } from "react";
import { User, setTopFriends } from "@/lib/api";
import { UserAvatar } from "./user-avatar";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Card, CardContent } from "./ui/card";

interface TopFriendsEditorProps {
  token: string;
  initialTopFriends: User[];
  localFriends: User[];
}

export function TopFriendsEditor({
  token,
  initialTopFriends,
  localFriends,
}: TopFriendsEditorProps) {
  const [topFriends, setTopFriendsState] = useState<User[]>(initialTopFriends);
  const [search, setSearch] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  const topIds = new Set(topFriends.map((f) => f.id));
  const suggestions = localFriends.filter(
    (f) =>
      !topIds.has(f.id) &&
      (f.username.toLowerCase().includes(search.toLowerCase()) ||
        (f.displayName ?? "").toLowerCase().includes(search.toLowerCase()))
  );

  function remove(id: string) {
    setTopFriendsState((prev) => prev.filter((f) => f.id !== id));
    setSaved(false);
  }

  function add(user: User) {
    if (topFriends.length >= 8) return;
    setTopFriendsState((prev) => [...prev, user]);
    setSearch("");
    setSaved(false);
  }

  async function save() {
    setSaving(true);
    setError(null);
    setSaved(false);
    try {
      await setTopFriends(token, topFriends.map((f) => f.id));
      setSaved(true);
    } catch {
      setError("Failed to save. Please try again.");
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardContent className="p-4 space-y-2">
          {topFriends.length === 0 && (
            <p className="text-sm text-muted-foreground text-center py-4">
              No top friends yet. Add up to 8 from the search below.
            </p>
          )}
          {topFriends.map((friend, i) => (
            <div
              key={friend.id}
              className="flex items-center gap-3 p-2 rounded-lg bg-accent/30"
            >
              <span className="text-xs text-muted-foreground w-4 text-center">
                {i + 1}
              </span>
              <UserAvatar
                src={friend.avatarUrl}
                alt={friend.displayName ?? friend.username}
                className="h-8 w-8"
              />
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium truncate">
                  {friend.displayName || friend.username}
                </p>
                <p className="text-xs text-muted-foreground">
                  @{friend.username}
                </p>
              </div>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => remove(friend.id)}
                className="h-6 w-6 p-0 text-muted-foreground hover:text-destructive"
              >
                ×
              </Button>
            </div>
          ))}
        </CardContent>
      </Card>

      {topFriends.length < 8 && (
        <div className="space-y-2">
          <Input
            placeholder="Search local friends to add…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          {search.length > 0 && (
            <Card>
              <CardContent className="p-2 space-y-1">
                {suggestions.length === 0 ? (
                  <p className="text-sm text-muted-foreground px-2 py-1">
                    No matches.
                  </p>
                ) : (
                  suggestions.slice(0, 5).map((user) => (
                    <button
                      key={user.id}
                      onClick={() => add(user)}
                      className="flex items-center gap-3 w-full p-2 rounded-lg hover:bg-accent text-left"
                    >
                      <UserAvatar
                        src={user.avatarUrl}
                        alt={user.displayName ?? user.username}
                        className="h-7 w-7"
                      />
                      <div>
                        <p className="text-sm font-medium">
                          {user.displayName || user.username}
                        </p>
                        <p className="text-xs text-muted-foreground">
                          @{user.username}
                        </p>
                      </div>
                    </button>
                  ))
                )}
              </CardContent>
            </Card>
          )}
        </div>
      )}

      <div className="flex items-center gap-3">
        <Button onClick={save} disabled={saving}>
          {saving ? "Saving…" : "Save"}
        </Button>
        {saved && (
          <p className="text-sm text-emerald-500">Saved!</p>
        )}
        {error && (
          <p className="text-sm text-destructive">{error}</p>
        )}
      </div>
    </div>
  );
}
