"use client";

import { useState } from "react";
import { useAuth } from "@/hooks/use-auth";
import { updateProfile, type ProfileField } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";
import { Plus, Trash2 } from "lucide-react";

const MAX_FIELDS = 4;

export function ProfileFieldsEditor({
  initial,
}: {
  initial: ProfileField[];
}) {
  const { token } = useAuth();
  const [fields, setFields] = useState<ProfileField[]>(initial);
  const [saving, setSaving] = useState(false);

  const update = (i: number, key: "name" | "value", val: string) => {
    setFields((prev) => prev.map((f, j) => (j === i ? { ...f, [key]: val } : f)));
  };

  const add = () => {
    if (fields.length >= MAX_FIELDS) return;
    setFields((prev) => [...prev, { name: "", value: "" }]);
  };

  const remove = (i: number) => {
    setFields((prev) => prev.filter((_, j) => j !== i));
  };

  const save = async () => {
    if (!token) return;
    const clean = fields.filter((f) => f.name.trim() || f.value.trim());
    setSaving(true);
    try {
      await updateProfile({ profileFields: clean }, token);
      setFields(clean);
      toast.success("Profile fields saved.");
    } catch {
      toast.error("Failed to save profile fields.");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="glass-effect glossy-effect bottom rounded-md shadow-fa-lg p-4 space-y-3">
      <div>
        <h3 className="text-lg font-medium">Profile fields</h3>
        <p className="text-sm text-muted-foreground">
          Add up to {MAX_FIELDS} custom fields visible on your profile and
          across the fediverse (e.g. Website, Pronouns).
        </p>
      </div>

      <div className="space-y-2">
        {fields.map((f, i) => (
          <div key={i} className="flex gap-2 items-center">
            <Input
              value={f.name}
              onChange={(e) => update(i, "name", e.target.value)}
              placeholder="Label"
              className="max-w-[8rem] text-sm"
            />
            <Input
              value={f.value}
              onChange={(e) => update(i, "value", e.target.value)}
              placeholder="Value"
              className="text-sm"
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
        {fields.length < MAX_FIELDS && (
          <Button variant="outline" size="sm" onClick={add}>
            <Plus className="h-4 w-4 mr-1" /> Add field
          </Button>
        )}
        <Button size="sm" onClick={save} disabled={saving}>
          {saving ? "Saving…" : "Save"}
        </Button>
      </div>
    </div>
  );
}
