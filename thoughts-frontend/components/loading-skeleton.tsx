import { Card, CardContent, CardHeader } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"

export function ThoughtSkeleton() {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center gap-4">
        <Skeleton className="h-10 w-10 rounded-full" />
        <div className="space-y-2">
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-3 w-20" />
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <Skeleton className="h-4 w-full" />
        <Skeleton className="h-4 w-4/5" />
      </CardContent>
    </Card>
  )
}

export function ProfileSkeleton() {
  return (
    <Card>
      <CardContent className="pt-6 flex items-center gap-4">
        <Skeleton className="h-16 w-16 rounded-full" />
        <div className="space-y-2">
          <Skeleton className="h-5 w-40" />
          <Skeleton className="h-4 w-24" />
        </div>
      </CardContent>
    </Card>
  )
}

export function TagsSkeleton() {
  return (
    <Card>
      <CardContent className="pt-4 space-y-2">
        <Skeleton className="h-4 w-24 mb-3" />
        {[...Array(5)].map((_, i) => (
          <Skeleton key={i} className="h-6 w-full rounded-full" />
        ))}
      </CardContent>
    </Card>
  )
}

export function CountSkeleton() {
  return (
    <Card>
      <CardContent className="pt-4 pb-4">
        <Skeleton className="h-6 w-32" />
      </CardContent>
    </Card>
  )
}
