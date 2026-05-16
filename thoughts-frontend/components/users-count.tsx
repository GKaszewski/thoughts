import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
} from "@/components/ui/card";
import { getAllUsersCount } from "@/lib/api";

export async function UsersCount() {
  const usersCount = await getAllUsersCount().catch(() => null);

  const count = usersCount?.count ?? null;

  return (
    <Card className="p-4">
      <CardHeader className="p-0 pb-3">
        <CardTitle className="text-lg flex items-center gap-2">
          <span className="widget-icon widget-icon-purple">✨</span>
          Community
        </CardTitle>
      </CardHeader>
      <CardContent className="p-0">
        {count === null ? (
          <p className="text-sm text-muted-foreground text-center py-2">
            Could not load member count.
          </p>
        ) : count === 0 ? (
          <p className="text-sm text-muted-foreground text-center py-2">
            Be the first to join!
          </p>
        ) : (
          <div
            className="rounded-xl p-3 text-center glossy-effect relative overflow-hidden"
            style={{
              background: "rgba(255,255,255,0.4)",
              border: "1px solid rgba(255,255,255,0.6)",
            }}
          >
            <div
              className="text-3xl font-extrabold leading-none"
              style={{
                background: "linear-gradient(135deg, #2563eb, #06b6d4)",
                WebkitBackgroundClip: "text",
                WebkitTextFillColor: "transparent",
                backgroundClip: "text",
              }}
            >
              {count}
            </div>
            <div className="text-[10px] uppercase tracking-widest text-muted-foreground mt-1 font-semibold">
              members
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
