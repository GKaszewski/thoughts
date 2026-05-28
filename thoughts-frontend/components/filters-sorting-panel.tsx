"use client";

import { useRouter, useSearchParams } from "next/navigation";
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

  const currentSort = (searchParams.get("sort") ?? "newest") as FeedSortOption;
  const originalsOnly = searchParams.get("originals_only") === "true";
  const repliesOnly   = searchParams.get("replies_only")   === "true";
  const localOnly     = searchParams.get("local_only")     === "true";
  const hideSensitive = searchParams.get("hide_sensitive") === "true";

  function update(key: string, value: string | null) {
    const params = new URLSearchParams(searchParams.toString());
    params.delete("page");
    if (value === null) {
      params.delete(key);
    } else {
      params.set(key, value);
    }
    router.push(`/?${params.toString()}`);
  }

  function setSort(value: FeedSortOption) {
    update("sort", value === "newest" ? null : value);
  }

  function toggleFilter(key: string, current: boolean) {
    update(key, current ? null : "true");
  }

  return (
    <div className="space-y-4">
      <div>
        <h3 className="text-sm font-semibold mb-2">Sort by</h3>
        <div className="space-y-1">
          {SORT_OPTIONS.map((opt) => (
            <label key={opt.value} className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="feed-sort"
                value={opt.value}
                checked={currentSort === opt.value}
                onChange={() => setSort(opt.value)}
                className="accent-primary"
              />
              <span className="text-sm">{opt.label}</span>
            </label>
          ))}
        </div>
      </div>

      <div>
        <h3 className="text-sm font-semibold mb-2">Filter</h3>
        <div className="space-y-1">
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={originalsOnly}
              onChange={() => {
                const params = new URLSearchParams(searchParams.toString());
                params.delete("page");
                if (!originalsOnly) {
                  params.delete("replies_only");
                  params.set("originals_only", "true");
                } else {
                  params.delete("originals_only");
                }
                router.push(`/?${params.toString()}`);
              }}
              className="accent-primary"
            />
            <span className="text-sm">Originals only</span>
          </label>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={repliesOnly}
              onChange={() => {
                const params = new URLSearchParams(searchParams.toString());
                params.delete("page");
                if (!repliesOnly) {
                  params.delete("originals_only");
                  params.set("replies_only", "true");
                } else {
                  params.delete("replies_only");
                }
                router.push(`/?${params.toString()}`);
              }}
              className="accent-primary"
            />
            <span className="text-sm">Replies only</span>
          </label>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={localOnly}
              onChange={() => toggleFilter("local_only", localOnly)}
              className="accent-primary"
            />
            <span className="text-sm">Local only</span>
          </label>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={hideSensitive}
              onChange={() => toggleFilter("hide_sensitive", hideSensitive)}
              className="accent-primary"
            />
            <span className="text-sm">Hide sensitive</span>
          </label>
        </div>
      </div>
    </div>
  );
}
