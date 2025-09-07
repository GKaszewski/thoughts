import Link from "next/link";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { getPopularTags } from "@/lib/api";
import { Hash } from "lucide-react";

export async function PopularTags() {
  const tags = await getPopularTags().catch(() => []);

  if (tags.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Popular Tags</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-center text-muted-foreground">
            No popular tags to display.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="p-4">
      <CardHeader className="p-0 pb-2">
        <CardTitle className="text-lg">Popular Tags</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-wrap gap-2 p-0">
        {tags.map((tag) => (
          <Link href={`/tags/${tag}`} key={tag}>
            <Badge
              variant="secondary"
              className="hover:shadow-lg transition-shadow text-shadow-sm cursor-pointer"
            >
              <Hash className="mr-1 h-3 w-3" />
              {tag}
            </Badge>
          </Link>
        ))}
        {tags.length === 0 && (
          <p className="text-sm text-muted-foreground">No popular tags yet.</p>
        )}
      </CardContent>
    </Card>
  );
}
