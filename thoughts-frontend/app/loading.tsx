import { ThoughtSkeleton } from "@/components/loading-skeleton";

export default function FeedLoading() {
  return (
    <div className="container mx-auto max-w-6xl p-4 sm:p-6">
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
        <aside className="hidden lg:block lg:col-span-1" />
        <main className="col-span-1 lg:col-span-2 space-y-6">
          <div className="h-10 w-32 bg-muted rounded animate-pulse mb-6" />
          <div className="space-y-4">
            <ThoughtSkeleton />
            <ThoughtSkeleton />
            <ThoughtSkeleton />
          </div>
        </main>
        <aside className="hidden lg:block lg:col-span-1" />
      </div>
    </div>
  );
}
