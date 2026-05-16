import { ThoughtSkeleton } from "@/components/loading-skeleton";

export default function SearchLoading() {
  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6 space-y-4">
      <div className="h-8 w-48 bg-muted rounded animate-pulse" />
      <ThoughtSkeleton />
      <ThoughtSkeleton />
      <ThoughtSkeleton />
    </div>
  );
}
