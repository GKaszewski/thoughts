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
    <Card>
      <CardHeader>
        <CardTitle>Popular Tags</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-wrap gap-2">
        {tags.map((tag) => (
          <Link href={`/tags/${tag}`} key={tag}>
            <Badge
              variant="secondary"
              className="hover:bg-accent cursor-pointer"
            >
              <Hash className="mr-1 h-3 w-3" />
              {tag}
            </Badge>
          </Link>
        ))}
      </CardContent>
    </Card>
  );
}
