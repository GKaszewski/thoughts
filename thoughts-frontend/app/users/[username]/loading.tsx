// app/users/[username]/loading.tsx
import { Card } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";

// This is the ProfileSkeleton component from the previous step.
// Next.js will automatically render this while page.tsx is loading.
export default function ProfileLoading() {
  return (
    <div>
      <Skeleton className="h-48 w-full" />
      <main className="container mx-auto max-w-3xl p-4 -mt-16">
        <Card className="p-6">
          <div className="flex items-end gap-4">
            <Skeleton className="h-24 w-24 rounded-full" />
            <div className="space-y-2">
              <Skeleton className="h-8 w-40" />
              <Skeleton className="h-4 w-24" />
            </div>
          </div>
          <Skeleton className="h-6 w-full mt-4" />
          <Skeleton className="h-6 w-3/4 mt-2" />
        </Card>
        <div className="mt-8 space-y-4">
          <Skeleton className="h-32 w-full" />
          <Skeleton className="h-32 w-full" />
        </div>
      </main>
    </div>
  );
}
