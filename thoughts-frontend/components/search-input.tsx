"use client";

import { useRouter } from "next/navigation";
import { Input } from "./ui/input";
import { Search as SearchIcon } from "lucide-react";

export function SearchInput() {
  const router = useRouter();

  const handleSearch = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const query = formData.get("q") as string;
    if (query) {
      router.push(`/search?q=${encodeURIComponent(query)}`);
    }
  };

  return (
    <form onSubmit={handleSearch} className="relative w-full md:max-w-sm">
      <SearchIcon className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
      <Input
        name="q"
        placeholder="Search for users or thoughts..."
        className="pl-9 md:min-w-[250px]"
      />
    </form>
  );
}
