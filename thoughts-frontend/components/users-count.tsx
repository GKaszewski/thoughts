import { Link } from "lucide-react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  CardDescription,
} from "@/components/ui/card";
import { getAllUsersCount } from "@/lib/api";

export async function UsersCount() {
  const usersCount = await getAllUsersCount().catch(() => null);

  if (usersCount === null) {
    return (
      <Card className="p-4">
        <CardHeader className="p-0 pb-2">
          <CardTitle className="text-lg text-shadow-md">Users Count</CardTitle>
          <CardDescription>
            Total number of registered users on Thoughts.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="text-muted-foreground text-sm text-center py-4">
            Could not load users count.
          </div>
        </CardContent>
      </Card>
    );
  }

  if (usersCount.count === 0) {
    return (
      <Card className="p-4">
        <CardHeader className="p-0 pb-2">
          <CardTitle className="text-lg text-shadow-md">Users Count</CardTitle>
          <CardDescription>
            Total number of registered users on Thoughts.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="text-muted-foreground text-sm text-center py-4">
            No registered users yet. Be the first to{" "}
            <Link href="/signup" className="text-primary hover:underline">
              sign up
            </Link>
            !
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="p-4">
      <CardHeader className="p-0 pb-2">
        <CardTitle className="text-lg text-shadow-md">Users Count</CardTitle>
        <CardDescription>
          Total number of registered users on Thoughts.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="text-muted-foreground text-sm text-center py-4">
          {usersCount.count} registered users.
        </div>
      </CardContent>
    </Card>
  );
}
