"use client";

import { useState } from "react";
import { useAuth } from "@/hooks/use-auth";
import { updateProfile, type ProfileField } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";
import { Plus, Trash2 } from "lucide-react";

const MAX_MOODS = 8;

export function CustomMoodsEditor({
  initial,
}: {
  initial: ProfileField[];
}) {
  const { token } = useAuth();
  const [moods, setMoods] = useState<ProfileField[]>(initial);
  const [saving, setSaving] = useState(false);

  const update = (i: number, key: "name" | "value", val: string) => {
    setMoods((prev) => prev.map((f, j) => (j === i ? { ...f, [key]: val } : f)));
  };

  const add = () => {
    if (moods.length >= MAX_MOODS) return;
    setMoods((prev) => [...prev, { name: "", value: "" }]);
  };

  const remove = (i: number) => {
    setMoods((prev) => prev.filter((_, j) => j !== i));
  };

  const save = async () => {
    if (!token) return;
    const clean = moods.filter((f) => f.name.trim() || f.value.trim());
    setSaving(true);
    try {
      await updateProfile({ customMoods: clean }, token);
      setMoods(clean);
      toast.success("Custom moods saved.");
    } catch {
      toast.error("Failed to save custom moods.");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="glass-effect glossy-effect bottom rounded-md shadow-fa-lg p-4 space-y-3">
      <div>
        <h3 className="text-lg font-medium">Custom moods</h3>
        <p className="text-sm text-muted-foreground">
          Add up to {MAX_MOODS} custom moods. These appear alongside the
          predefined moods when composing a thought.
        </p>
      </div>

      <div className="space-y-2">
        {moods.map((f, i) => (
          <div key={i} className="flex gap-2 items-center">
            <Input
              value={f.name}
              onChange={(e) => update(i, "name", e.target.value)}
              placeholder="Mood name"
              className="max-w-[10rem] text-sm"
            />
            <Input
              value={f.value}
              onChange={(e) => update(i, "value", e.target.value)}
              placeholder="Emoji"
              className="max-w-[5rem] text-sm"
            />
            <Button
              variant="ghost"
              size="icon"
              onClick={() => remove(i)}
              className="shrink-0"
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        ))}
      </div>

      <div className="flex gap-2">
        {moods.length < MAX_MOODS && (
          <Button variant="outline" size="sm" onClick={add}>
            <Plus className="h-4 w-4 mr-1" /> Add mood
          </Button>
        )}
        <Button size="sm" onClick={save} disabled={saving}>
          {saving ? "Saving…" : "Save"}
        </Button>
      </div>
    </div>
  );
}
