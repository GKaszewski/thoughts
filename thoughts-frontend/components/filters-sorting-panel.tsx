"use client";

import { useState, useTransition } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { FeedSortOption } from "@/lib/api";

const SORT_OPTIONS: { value: FeedSortOption; label: string }[] = [
  { value: "newest",         label: "Newest first" },
  { value: "oldest",         label: "Oldest first" },
  { value: "most_liked",     label: "Most liked" },
  { value: "most_boosted",   label: "Most boosted" },
  { value: "most_discussed", label: "Most discussed" },
];

export function FiltersSortingPanel() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [isPending, startTransition] = useTransition();

  const [sort, setSort] = useState<FeedSortOption>(
    (searchParams.get("sort") as FeedSortOption | null) ?? "newest"
  );
  const [originalsOnly, setOriginalsOnly] = useState(
    searchParams.get("originals_only") === "true"
  );
  const [repliesOnly, setRepliesOnly] = useState(
    searchParams.get("replies_only") === "true"
  );
  const [localOnly, setLocalOnly] = useState(
    searchParams.get("local_only") === "true"
  );
  const [hideSensitive, setHideSensitive] = useState(
    searchParams.get("hide_sensitive") === "true"
  );

  function pushParams(updates: Record<string, string | null>) {
    const params = new URLSearchParams(searchParams.toString());
    params.delete("page");
    for (const [key, value] of Object.entries(updates)) {
      if (value === null) {
        params.delete(key);
      } else {
        params.set(key, value);
      }
    }
    startTransition(() => router.replace(`/?${params.toString()}`));
  }

  function handleSort(value: FeedSortOption) {
    setSort(value);
    pushParams({ sort: value === "newest" ? null : value });
  }

  function handleOriginalsOnly(checked: boolean) {
    setOriginalsOnly(checked);
    if (checked) setRepliesOnly(false);
    const updates: Record<string, string | null> = {
      originals_only: checked ? "true" : null,
    };
    if (checked) updates.replies_only = null;
    pushParams(updates);
  }

  function handleRepliesOnly(checked: boolean) {
    setRepliesOnly(checked);
    if (checked) setOriginalsOnly(false);
    const updates: Record<string, string | null> = {
      replies_only: checked ? "true" : null,
    };
    if (checked) updates.originals_only = null;
    pushParams(updates);
  }

  function handleLocalOnly(checked: boolean) {
    setLocalOnly(checked);
    pushParams({ local_only: checked ? "true" : null });
  }

  function handleHideSensitive(checked: boolean) {
    setHideSensitive(checked);
    pushParams({ hide_sensitive: checked ? "true" : null });
  }

  return (
    <div
      className={`space-y-3 transition-opacity duration-150 ${
        isPending ? "opacity-50 pointer-events-none" : ""
      }`}
    >
      <div>
        <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
          Sort by
        </p>
        <RadioGroup
          value={sort}
          onValueChange={(v) => handleSort(v as FeedSortOption)}
          className="space-y-1"
        >
          {SORT_OPTIONS.map((opt) => (
            <div key={opt.value} className="flex items-center gap-2">
              <RadioGroupItem value={opt.value} id={`sort-${opt.value}`} />
              <Label
                htmlFor={`sort-${opt.value}`}
                className="text-xs font-normal cursor-pointer"
              >
                {opt.label}
              </Label>
            </div>
          ))}
        </RadioGroup>
      </div>

      <Separator />

      <div>
        <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
          Filter
        </p>
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <Checkbox
              id="originals-only"
              checked={originalsOnly}
              onCheckedChange={(c) => handleOriginalsOnly(c === true)}
            />
            <Label htmlFor="originals-only" className="text-xs font-normal cursor-pointer">
              Originals only
            </Label>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox
              id="replies-only"
              checked={repliesOnly}
              onCheckedChange={(c) => handleRepliesOnly(c === true)}
            />
            <Label htmlFor="replies-only" className="text-xs font-normal cursor-pointer">
              Replies only
            </Label>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox
              id="local-only"
              checked={localOnly}
              onCheckedChange={(c) => handleLocalOnly(c === true)}
            />
            <Label htmlFor="local-only" className="text-xs font-normal cursor-pointer">
              Local only
            </Label>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox
              id="hide-sensitive"
              checked={hideSensitive}
              onCheckedChange={(c) => handleHideSensitive(c === true)}
            />
            <Label htmlFor="hide-sensitive" className="text-xs font-normal cursor-pointer">
              Hide sensitive
            </Label>
          </div>
        </div>
      </div>
    </div>
  );
}
