"use client";

import { useState } from "react";
import { setAlsoKnownAs } from "@/lib/api";
import { useAuth } from "@/hooks/use-auth";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";

export function MigrationSettings() {
  const { token } = useAuth();
  const [value, setValue] = useState("");
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    if (!token) return;
    setSaving(true);
    try {
      await setAlsoKnownAs(value.trim() || null, token);
      toast.success(value.trim() ? "Also known as saved." : "Also known as cleared.");
    } catch {
      toast.error("Failed to save.");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="glass-effect glossy-effect bottom rounded-md shadow-fa-lg p-4 space-y-3">
      <div>
        <h3 className="text-lg font-medium">Account migration</h3>
        <p className="text-sm text-muted-foreground">
          Set your new actor URL before broadcasting a Move. This lets remote
          servers verify the migration is legitimate.
        </p>
      </div>
      <div className="space-y-1">
        <label className="text-sm font-medium">Also known as</label>
        <p className="text-xs text-muted-foreground">
          The full actor URL on your new server, e.g.{" "}
          <code className="font-mono">https://newdomain.com/users/&lt;uuid&gt;</code>
        </p>
        <div className="flex gap-2">
          <Input
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder="https://newdomain.com/users/…"
            className="font-mono text-sm"
          />
          <Button onClick={handleSave} disabled={saving} size="sm">
            {saving ? "Saving…" : "Save"}
          </Button>
        </div>
      </div>
    </div>
  );
}
