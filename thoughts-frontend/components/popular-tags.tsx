import Link from "next/link";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { getPopularTags } from "@/lib/api";

export async function PopularTags() {
  const tags = await getPopularTags().catch(() => []);

  if (tags.length === 0) {
    return (
      <Card className="p-4">
        <CardHeader className="p-0 pb-2">
          <CardTitle className="text-lg flex items-center gap-2">
            <span className="widget-icon widget-icon-blue">🏷</span>
            Popular Tags
          </CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <p className="text-center text-sm text-muted-foreground py-4">No tags yet.</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="p-4">
      <CardHeader className="p-0 pb-3">
        <CardTitle className="text-lg flex items-center gap-2">
          <span className="widget-icon widget-icon-blue">🏷</span>
          Popular Tags
        </CardTitle>
      </CardHeader>
      <CardContent className="flex flex-wrap gap-2 p-0">
        {tags.map((tag, i) => (
          <Link href={`/tags/${tag}`} key={tag}>
            <Badge variant={i < 2 ? "trending" : "branded"}>
              {i < 2 ? "🔥 " : "#"}{tag}
            </Badge>
          </Link>
        ))}
      </CardContent>
    </Card>
  );
}
