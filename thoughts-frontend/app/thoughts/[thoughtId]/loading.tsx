import { ThoughtSkeleton } from "@/components/loading-skeleton";

export default function ThoughtLoading() {
  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6 space-y-4">
      <ThoughtSkeleton />
      <div className="pl-6 border-l-2 border-primary border-dashed space-y-4">
        <ThoughtSkeleton />
        <ThoughtSkeleton />
      </div>
    </div>
  );
}
